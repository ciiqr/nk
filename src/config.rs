use crate::{
    args::Arguments,
    state::{Machine, RawMachine},
    utils::deserialize_map_to_vec_of_named,
};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{de::Error, Deserialize, Deserializer};
use std::{path::PathBuf, str::FromStr};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // TODO: either need to support multiple configs or this needs to be configured some other way...
    pub machine: Option<String>,
    #[serde(deserialize_with = "expand_paths")]
    pub sources: Vec<PathBuf>,
    pub plugins: Vec<ConfigPlugin>,
    #[serde(
        default,
        deserialize_with = "deserialize_map_to_vec_of_named::<RawMachine, _, _>"
    )]
    pub machines: Vec<Machine>,
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
        name: String,
    },
}

lazy_static! {
    static ref GITHUB_PLUGIN_REGEX: Regex =
        Regex::new(r"^(.+?)/(.+?)(@(.+))?#(.*)$").unwrap();
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
                    captures.get(1),
                    captures.get(2),
                    captures.get(4),
                    captures.get(5),
                ) {
                    // TODO: name is going to become optional... but then, we need to be sure whatever uses PluginSource::Github handles that this represents multiple plugins...
                    // TODO: regex also needs to mark the name group as optional
                    (Some(owner), Some(repo), version, Some(name)) => {
                        Ok(PluginSource::Github {
                            owner: owner.as_str().to_string(),
                            repo: repo.as_str().to_string(),
                            version: version.map_or(Version::Latest, |v| {
                                Version::Version(v.as_str().to_string())
                            }),
                            name: name.as_str().to_string(),
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
    // TODO: maybe std::path::absolute once stable?
    paths
        .iter()
        .map(|s| {
            PathBuf::from_str(&shellexpand::tilde(&s)).map_err(D::Error::custom)
        })
        // TODO: maybe consider partitioning and showing all the errors instead...
        .collect::<Result<Vec<_>, _>>()
}
