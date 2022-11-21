use crate::{
    args::Arguments,
    state::{Machine, RawMachine},
    utils::deserialize_map_to_vec_of_named,
};
use serde::{de::Error, Deserialize, Deserializer};
use std::{path::PathBuf, str::FromStr};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // TODO: either need to support multiple configs or this needs to be configured some other way...
    pub machine: Option<String>,
    #[serde(deserialize_with = "expand_paths")]
    pub sources: Vec<PathBuf>,
    // TODO: maybe move plugin config to sources if current setup is too limiting
    pub plugins: Vec<ConfigPlugin>,
    #[serde(
        default,
        deserialize_with = "deserialize_map_to_vec_of_named::<RawMachine, _, _>"
    )]
    pub machines: Vec<Machine>,
}

impl Config {
    pub fn new(arguments: &Arguments) -> Result<Config, Box<dyn std::error::Error>> {
        // TODO: provide a better error when config file doesn't exist
        let local_path = &PathBuf::from_str(&shellexpand::tilde(".nk.yml"))?;
        let path = arguments
            .global
            .config
            .as_ref()
            // TODO: .nk.yml OR ~/.nk.yml? (or merge both?)
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

#[derive(Debug)]
pub enum PluginSource {
    Local { source: PathBuf },
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
            Some(_) | None => Err(D::Error::custom(format!(
                "Unrecognized plugin source: {}",
                source
            ))),
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
        .map(|s| PathBuf::from_str(&shellexpand::tilde(&s)).map_err(D::Error::custom))
        // TODO: maybe consider partitioning and showing all the errors instead...
        .collect::<Result<Vec<_>, _>>()
}
