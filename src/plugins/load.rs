use super::{Manifest, ManifestAssets, Plugin};
use crate::{
    config::{Config, PluginSource, Version},
    eval::Evaluator,
    vars::{get_system_vars, SystemVars},
};
use async_compression::futures::bufread::GzipDecoder;
use async_tar::Archive;
use futures::{
    io::{self, BufReader, ErrorKind},
    prelude::*,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
    str::FromStr,
};
use std::{fs, sync::Mutex};

async fn download_github_plugin(
    owner: &str,
    repo: &str,
    version: &Version,
    plugin: &str,
    system_vars: &SystemVars,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: need a way of caching the non-existence of a plugin for the current platform... (something around the plugin_dir so users can detect it and easily fix it)
    // - probably just need to save the manifest...
    let plugin_dir = PathBuf::from_str(&shellexpand::tilde(
        format!("~/.nk/plugins/{}", plugin).as_str(),
    ))?;

    // linked plugins should be left as is
    if plugin_dir.is_symlink() {
        return Ok(());
    }

    // determine expected version
    let expected_version = match version {
        Version::Latest => {
            let manifest =
                get_release_manifest(owner, repo, version)
                    .await
                    // TODO: move error handling...
                    .map_err(|e| {
                        format!("{e}: while downloading manifest for {owner}/{repo}@{version}")
                    })?;

            Version::Version(manifest.version)
        }
        v @ Version::Version(_) => v.clone(),
    };

    // download/update plugin
    let exists = plugin_dir.try_exists()?;
    if !exists || current_version(&plugin_dir)? != expected_version {
        let manifest =
            get_release_manifest(owner, repo, version).await
            // TODO: move error handling...
            .map_err(|e| {
                format!("{e}: while downloading manifest for {owner}/{repo}@{version}")
            })?;

        // determine asset to download
        let manifest_plugin = manifest
            .plugins
            .iter()
            .find(|p| p.name == *plugin)
            .ok_or_else(|| {
                format!(
                    "could not find plugin \"{plugin}\" in {owner}/{repo}@{version}, did you mean:\n\t- {}",
                    manifest.plugins.iter().map(|p| &p.name).join("\n\t- ")
                )
            })?;

        // TODO: filter assets to only those matching
        // TODO: decide if we want to continue with the asset name priority at all
        // - ? should we sort in a similar way, but with the when conditions as the input?
        // - ? should we simply rely on the when condition and asset order (first to match wins. if multiple match consider treating it as an error even...)
        let asset_priority = get_asset_priority(plugin, system_vars);
        let asset = manifest_plugin.assets.iter()
            .filter(|a| asset_priority.iter().any(|ap| *ap == a.file))
            .sorted_by_cached_key(|a| {
                asset_priority
                    .iter()
                    .position(|ap| *ap == a.file)
                    .expect("unreachable: asset position not found in asset priority list")
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
            let asset_url = get_asset_url(&manifest, asset);
            let response = reqwest::get(asset_url).await?;
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
            fs::write(version_file, manifest.version)?;
        }
    }

    Ok(())
}

pub async fn load_plugins(
    config: &Config,
    evaluator: &Evaluator,
) -> Result<Vec<Plugin>, Box<dyn std::error::Error>> {
    let system_vars = get_system_vars()?;

    // download/update remote plugins
    for plugin in &config.plugins {
        match &plugin.source {
            PluginSource::Local { source: _ } => {} // TODO: might want to link local plugins into ~/.nk/plugins/
            PluginSource::Github {
                owner,
                repo,
                version,
                plugin,
            } => {
                if let Some(plugin) = plugin {
                    download_github_plugin(
                        owner,
                        repo,
                        version,
                        plugin,
                        &system_vars,
                    )
                    .await?;
                } else {
                    let manifest =
                            get_release_manifest(owner, repo, version)
                                .await
                                // TODO: move error handling...
                                .map_err(|e| {
                                    format!("{e}: while downloading manifest for {owner}/{repo}@{version}")
                                })?;

                    for p in manifest.plugins {
                        download_github_plugin(
                            owner,
                            repo,
                            version,
                            &p.name,
                            &system_vars,
                        )
                        .await?;
                    }
                }
            }
        }
    }

    // load plugins
    let all_plugins = futures::future::join_all(config
        .plugins
        .iter()
        .enumerate()
        .map(|(i, p)| {
             async move {
                Ok::<_, Box<dyn std::error::Error>>(match &p.source {
                    PluginSource::Local { source } => vec![
                        Some(Plugin::from_path(source.into(), i))
                    ],
                    PluginSource::Github {
                        owner,
                        repo,
                        version,
                        plugin,
                    } => {
                        if let Some(plugin) = plugin {
                            let plugin_dir = PathBuf::from_str(&shellexpand::tilde(
                                format!("~/.nk/plugins/{}", plugin).as_str(),
                            ))?;

                            let exists = plugin_dir.try_exists()?;
                            if !exists {
                                return Ok(vec![None]);
                            }

                            vec![Some(Plugin::from_path(plugin_dir, i))]
                        }else {
                            let manifest =
                                get_release_manifest(owner, repo, version)
                                    .await
                                    // TODO: move error handling...
                                    .map_err(|e| {
                                        format!("{e}: while downloading manifest for {owner}/{repo}@{version}")
                                    })?;

                            manifest.plugins.iter().map(|p| {
                                let plugin_dir = PathBuf::from_str(&shellexpand::tilde(
                                    format!("~/.nk/plugins/{}", p.name).as_str(),
                                ))?;

                                let exists = plugin_dir.try_exists()?;
                                if !exists {
                                    return Ok(None);
                                }

                                Ok::<_, Box<dyn std::error::Error>>(Some(Plugin::from_path(plugin_dir, i)))
                            }).collect::<Result<Vec<_>, _>>()?
                        }
                    }
                })
            }
        })).await.into_iter()
        // TODO: fix this (we're just ignoring errors...)
        .filter_map(Result::ok)
        .flatten()
        .flatten()
        .collect::<Result<_, _>>()?;

    // filter plugins for os
    let plugins = evaluator.filter_plugins(all_plugins)?;

    Ok(plugins)
}

fn current_version(
    plugin_dir: &Path,
) -> Result<Version, Box<dyn std::error::Error>> {
    let version_file = plugin_dir.join(".nk_version");
    Ok(Version::Version(read_to_string(version_file)?))
}

lazy_static! {
    static ref MANIFEST_CACHE: Mutex<HashMap<RepoReleaseReference, Manifest>> =
        Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RepoReleaseReference {
    owner: String,
    repo: String,
    version: Version,
}

pub async fn get_release_manifest(
    owner: &str,
    repo: &str,
    version: &Version,
) -> Result<Manifest, Box<dyn std::error::Error>> {
    let release_ref = RepoReleaseReference {
        owner: owner.into(),
        repo: repo.into(),
        version: version.clone(),
    };

    if let Some(release) = MANIFEST_CACHE
        .lock()
        .expect("manifest cache lock")
        .get(&release_ref)
    {
        return Ok(release.clone());
    }

    let file = "manifest.yml";
    // TODO: proper url handling
    let url = match version {
        Version::Latest => format!(
            "https://github.com/{owner}/{repo}/releases/latest/download/{file}"
        ),
        Version::Version(version) => format!(
            "https://github.com/{owner}/{repo}/releases/download/{version}/{file}"
        ),
    };

    let response = reqwest::get(url).await?;
    let manifest: Manifest = serde_yaml::from_str(&response.text().await?)?;

    match version {
        Version::Latest => {
            // cache as latest release
            MANIFEST_CACHE
                .lock()
                .expect("manifest cache lock")
                .insert(release_ref, manifest.clone());

            // cache as exact version release
            let exact_version_ref = RepoReleaseReference {
                owner: owner.into(),
                repo: repo.into(),
                version: Version::Version(manifest.version.clone()),
            };
            MANIFEST_CACHE
                .lock()
                .expect("manifest cache lock")
                .insert(exact_version_ref, manifest.clone());
        }
        Version::Version(_) => {
            // cache release
            MANIFEST_CACHE
                .lock()
                .expect("manifest cache lock")
                .insert(release_ref, manifest.clone());
        }
    }

    Ok(manifest)
}

fn get_asset_url(manifest: &Manifest, asset: &ManifestAssets) -> String {
    let owner = &manifest.owner;
    let repo = &manifest.repo;
    let version = &manifest.version;
    let file = &asset.file;

    // TODO: proper url handling
    format!(
        "https://github.com/{owner}/{repo}/releases/download/{version}/{file}"
    )
}

fn get_asset_priority(name: &str, vars: &SystemVars) -> Vec<String> {
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

    let SystemVars {
        distro,
        os,
        family,
        arch,
    } = vars;

    vec![
        format!("{name}-{distro}-{arch}.tar.gz"),
        format!("{name}-{os}-{arch}.tar.gz"),
        format!("{name}-{family}-{arch}.tar.gz"),
        format!("{name}-{arch}.tar.gz"),
        format!("{name}-{distro}.tar.gz"),
        format!("{name}-{os}.tar.gz"),
        format!("{name}-{family}.tar.gz"),
        format!("{name}.tar.gz"),
    ]
}
