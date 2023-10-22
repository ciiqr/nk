use crate::args::Arguments;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{de::Error, Deserialize, Deserializer};
use std::{path::PathBuf, str::FromStr};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(deserialize_with = "expand_paths")]
    pub sources: Vec<PathBuf>,
    pub plugins: Vec<ConfigPlugin>,
}

impl Config {
    pub fn new(
        arguments: &Arguments,
    ) -> Result<Config, Box<dyn std::error::Error>> {
        // TODO: provide a better error when config file doesn't exist
        let local_path = &PathBuf::from_str(".nk.yml")?;
        let path = arguments
            .config
            .as_ref()
            // TODO: .nk.yml OR ~/.nk.yml? (or merge both?)
            // TODO: - resolve relative paths with the parent dir of the config as the base: https://crates.io/crates/relative-path
            .unwrap_or(local_path);

        let contents = match std::fs::read_to_string(path) {
            Ok(val) => Ok(val),
            Err(e) => Err(format!("{}: {}", e, path.display())),
        }?;

        let mut conf: Self = serde_yaml::from_str(&contents)?;

        // TODO: maybe there should we a way to flag sources as optional?
        conf.sources.retain(|s| s.exists());
        if conf.sources.is_empty() {
            Err("at least one source must exist".into())
        } else {
            Ok(conf)
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, transparent)]
pub struct ConfigPlugin {
    pub source: PluginSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Version {
    Latest,
    Version(String),
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Version::Latest => "latest",
            Version::Version(v) => v,
        })
    }
}

#[derive(Debug)]
pub enum PluginSource {
    Local {
        source: PathBuf,
    },
    Github {
        owner: String,
        repo: String,
        version: Version,
        plugin: Option<String>,
    },
}

lazy_static! {
    static ref GITHUB_PLUGIN_REGEX: Regex = Regex::new(
        r"^(?<owner>.+?)/(?<repo>.+?)(@(?<version>.+?))?(#(?<plugin>.*))?$"
    )
    .unwrap();
}

impl<'de> Deserialize<'de> for PluginSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let source: String = Deserialize::deserialize(deserializer)?;

        match source.chars().nth(1) {
            Some('~' | '.' | '/') => Ok(PluginSource::Local {
                source: PathBuf::from_str(&shellexpand::tilde(&source))
                    .map_err(D::Error::custom)?,
            }),
            Some(_) | None => {
                let captures =
                    GITHUB_PLUGIN_REGEX.captures(&source).ok_or_else(|| {
                        D::Error::custom(format!(
                            "Unrecognized plugin source: {}",
                            source
                        ))
                    })?;

                match (
                    captures.name("owner"),
                    captures.name("repo"),
                    captures.name("version"),
                    captures.name("plugin"),
                ) {
                    (Some(owner), Some(repo), version, plugin) => {
                        Ok(PluginSource::Github {
                            owner: owner.as_str().to_string(),
                            repo: repo.as_str().to_string(),
                            version: version.map_or(Version::Latest, |v| {
                                Version::Version(v.as_str().to_string())
                            }),
                            plugin: plugin.map(|p| p.as_str().to_string()),
                        })
                    }
                    _ => Err(D::Error::custom(format!(
                        "Unrecognized plugin source: {}",
                        source
                    ))),
                }
            }
        }
    }
}

fn expand_paths<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let paths: Vec<String> = Deserialize::deserialize(deserializer)?;

    paths
        .iter()
        .map(|s| {
            PathBuf::from_str(&shellexpand::tilde(&s))
                .map(|e| std::fs::canonicalize(e).map_err(D::Error::custom))
                .map_err(D::Error::custom)
        })
        // TODO: maybe consider partitioning and showing all the errors instead...
        .collect::<Result<Result<Vec<_>, _>, _>>()?
}
