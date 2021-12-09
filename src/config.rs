use crate::args::Arguments;
use serde::{de::Error, Deserialize, Deserializer};
use std::{path::PathBuf, str::FromStr};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub machine: String,
    #[serde(deserialize_with = "expand_paths")]
    pub sources: Vec<PathBuf>,
    // TODO: maybe move plugin config to sources if current setup is too limiting
    pub plugins: Vec<ConfigPlugin>,
}

impl Config {
    pub fn new(arguments: &Arguments) -> Result<Config, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(
            arguments
                .global
                .config
                .as_ref()
                .unwrap_or(&PathBuf::from_str(&shellexpand::tilde("~/.nk.yml"))?),
        )?;

        Ok(serde_yaml::from_str(&contents)?)
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

// TODO: move?
fn expand_paths<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let paths: Vec<String> = Deserialize::deserialize(deserializer)?;
    Ok(paths
        .iter()
        .map(|s| PathBuf::from_str(&shellexpand::tilde(&s)).map_err(D::Error::custom))
        // TODO: need to handle errors properly
        .filter(|res| res.is_ok())
        .map(|res| res.unwrap())
        .collect())
}
