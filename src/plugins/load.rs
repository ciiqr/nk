use super::Plugin;
use crate::{
    config::{Config, PluginSource, Version},
    eval::Evaluator,
};
use async_compression::futures::bufread::GzipDecoder;
use async_tar::Archive;
use futures::{
    io::{self, BufReader, ErrorKind},
    prelude::*,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use octocrab::{models::repos::Release, Octocrab};
use serde_yaml::Value;
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
    str::FromStr,
};
use std::{fs, sync::Mutex};

pub async fn load_plugins(
    config: &Config,
    builtin_vars: &HashMap<String, Value>,
    evaluator: &Evaluator,
) -> Result<Vec<Plugin>, Box<dyn std::error::Error>> {
    // TODO: stuff like this would be much cleaner if builtin_vars was a struct with specific fields
    let asset_vars = AssetVars {
        distro: builtin_vars
            .get("distro")
            .ok_or("couldn't find builtin var: distro")?
            .as_str()
            .ok_or("invalid type for builtin var: distro")?
            .to_string(),
        os: builtin_vars
            .get("os")
            .ok_or("couldn't find builtin var: os")?
            .as_str()
            .ok_or("invalid type for builtin var: os")?
            .to_string(),
        family: builtin_vars
            .get("family")
            .ok_or("couldn't find builtin var: family")?
            .as_str()
            .ok_or("invalid type for builtin var: family")?
            .to_string(),
        arch: builtin_vars
            .get("arch")
            .ok_or("couldn't find builtin var: arch")?
            .as_str()
            .ok_or("invalid type for builtin var: arch")?
            .to_string(),
    };

    // download/update remote plugins
    for plugin in &config.plugins {
        match &plugin.source {
            PluginSource::Local { source: _ } => {} // TODO: might want to link local plugins into ~/.nk/plugins/
            PluginSource::Github {
                owner,
                repo,
                version,
                name,
            } => {
                // TODO: need a way of caching the non-existence of a plugin for the current platform... (something around the plugin_dir so users can detect it and easily fix it)
                let plugin_dir = PathBuf::from_str(&shellexpand::tilde(
                    format!("~/.nk/plugins/{}", name).as_str(),
                ))?;

                // linked plugins should be left as is
                if plugin_dir.is_symlink() {
                    continue;
                }

                // determine expected version
                let expected_version = match version {
                    Version::Latest => {
                        let latest = get_release(owner, repo, version).await?;

                        Version::Version(latest.tag_name)
                    }
                    v @ Version::Version(_) => v.clone(),
                };

                // download/update plugin
                let exists = plugin_dir.try_exists()?;
                if !exists || current_version(&plugin_dir)? != expected_version {
                    let release = get_release(owner, repo, version).await?;

                    // determine asset to download
                    let asset_priority = get_asset_priority(name, &asset_vars);
                    let asset = release
                        .assets
                        .into_iter()
                        .filter(|a| asset_priority.iter().any(|ap| *ap == a.name))
                        .sorted_by_cached_key(|a| {
                            asset_priority
                                .iter()
                                .position(|ap| *ap == a.name)
                                .unwrap_or_else(|| {
                                    unreachable!("asset position not found in asset priority list")
                                })
                        })
                        .next(); // first

                    // download
                    // NOTE: if no asset is found, plugin is assumed to not support the current platform and is ignored
                    if let Some(asset) = asset {
                        // delete dir if it already exists
                        if exists {
                            fs::remove_dir_all(plugin_dir.clone())?;
                        }

                        // download asset
                        let response = reqwest::get(asset.browser_download_url).await?;
                        let reader = response
                            .bytes_stream()
                            .map_err(|e| io::Error::new(ErrorKind::Other, e))
                            .into_async_read();

                        // extract
                        let decoder = GzipDecoder::new(BufReader::new(reader));
                        let archive = Archive::new(decoder);
                        archive.unpack(plugin_dir.clone()).await?;

                        // TODO: ? ensure name in plugin.yml matches?

                        // write version file
                        let version_file = plugin_dir.join(".nk_version");
                        fs::write(version_file, release.tag_name)?;
                    }
                }
            }
        }
    }

    // load plugins
    let all_plugins = config
        .plugins
        .iter()
        .enumerate()
        .map::<Result<_, Box<dyn std::error::Error>>, _>(|(i, p)| match &p.source {
            PluginSource::Local { source: _ } => Ok((i, p)),
            PluginSource::Github {
                owner: _,
                repo: _,
                version: _,
                name,
            } => {
                let plugin_dir = PathBuf::from_str(&shellexpand::tilde(
                    format!("~/.nk/plugins/{}", name).as_str(),
                ))?;

                let exists = plugin_dir.try_exists()?;

                if exists {
                    Ok((i, p))
                } else {
                    Err(format!("{} doesn't exist", plugin_dir.to_string_lossy()).into())
                }
            }
        })
        // TODO: this filters permission errors too, fix this (we should only ignore the directory not existing...)
        .filter_map(Result::ok)
        .map(|(i, p)| Plugin::from_config(p, i))
        .collect::<Result<_, _>>()?;

    // filter plugins for os
    let plugins = evaluator.filter_plugins(all_plugins)?;

    Ok(plugins)
}

fn current_version(plugin_dir: &Path) -> Result<Version, Box<dyn std::error::Error>> {
    let version_file = plugin_dir.join(".nk_version");
    Ok(Version::Version(read_to_string(version_file)?))
}

lazy_static! {
    static ref RELEASE_CACHE: Mutex<HashMap<RepoReleaseReference, Release>> =
        Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RepoReleaseReference {
    owner: String,
    repo: String,
    version: Version,
}

async fn get_release(
    owner: &String,
    repo: &String,
    version: &Version,
) -> Result<Release, octocrab::Error> {
    let release_ref = RepoReleaseReference {
        owner: owner.clone(),
        repo: repo.clone(),
        version: version.clone(),
    };

    // TODO: unwrap...
    if let Some(release) = RELEASE_CACHE.lock().unwrap().get(&release_ref) {
        return Ok(release.clone());
    }

    // Create github api client
    let mut builder = Octocrab::builder();
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        builder = builder.personal_token(token);
    }
    let octocrab = builder.build()?;

    match version {
        Version::Latest => {
            // TODO: map common errors (http 404, gh rate limiting)
            let release = octocrab
                .repos(owner.clone(), repo.clone())
                .releases()
                .get_latest()
                .await?;

            // cache as latest release
            RELEASE_CACHE
                .lock()
                .unwrap() // TODO: unwrap...
                .insert(release_ref, release.clone());

            // cache as exact version release
            let exact_version_ref = RepoReleaseReference {
                owner: owner.clone(),
                repo: repo.clone(),
                version: Version::Version(release.tag_name.clone()),
            };
            RELEASE_CACHE
                .lock()
                .unwrap() // TODO: unwrap...
                .insert(exact_version_ref, release.clone());

            Ok(release)
        }
        Version::Version(v) => {
            let release = octocrab.repos(owner, repo).releases().get_by_tag(v).await?;

            // cache release
            RELEASE_CACHE
                .lock()
                .unwrap() // TODO: unwrap...
                .insert(release_ref, release.clone());

            Ok(release)
        }
    }
}

struct AssetVars {
    distro: String,
    os: String,
    family: String,
    arch: String,
}

fn get_asset_priority(name: &String, vars: &AssetVars) -> Vec<String> {
    // ie. files plugin on macos
    // - files-ventura-aarch64.tar.gz
    // - files-macos-aarch64.tar.gz
    // - files-unix-aarch64.tar.gz
    // - files-aarch64.tar.gz
    // - files-ventura.tar.gz
    // - files-macos.tar.gz
    // - files-unix.tar.gz
    // - files.tar.gz

    // ie. pacman plugin on arch linux
    // - pacman-arch-x86_64.tar.gz
    // - pacman-linux-x86_64.tar.gz
    // - pacman-unix-x86_64.tar.gz
    // - pacman-x86_64.tar.gz
    // - pacman-arch.tar.gz
    // - pacman-linux.tar.gz
    // - pacman-unix.tar.gz
    // - pacman.tar.gz

    // ie. chocolatey plugin on windows
    // - chocolatey-windows-x86_64.tar.gz
    // - chocolatey-x86_64.tar.gz
    // - chocolatey-windows.tar.gz
    // - chocolatey.tar.gz

    let AssetVars {
        distro,
        os,
        family,
        arch,
    } = vars;

    vec![
        format!(
            "{name}-{distro}-{arch}.tar.gz",
            name = name,
            distro = distro,
            arch = arch
        ),
        format!(
            "{name}-{os}-{arch}.tar.gz",
            name = name,
            os = os,
            arch = arch
        ),
        format!(
            "{name}-{family}-{arch}.tar.gz",
            name = name,
            family = family,
            arch = arch
        ),
        format!("{name}-{arch}.tar.gz", name = name, arch = arch,),
        format!("{name}-{distro}.tar.gz", name = name, distro = distro,),
        format!("{name}-{os}.tar.gz", name = name, os = os,),
        format!("{name}-{family}.tar.gz", name = name, family = family,),
        format!("{name}.tar.gz", name = name,),
    ]
}
