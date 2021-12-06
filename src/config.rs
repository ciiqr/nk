use std::{path::PathBuf, str::FromStr, vec};

use crate::args::Arguments;
use yaml_rust::{Yaml, YamlLoader};

// TODO: move
#[derive(Debug)]
pub enum PluginSource {
    Local { source: PathBuf },
}
#[derive(Debug)]
pub struct ConfigPlugin {
    pub source: PluginSource,
}

impl ConfigPlugin {
    pub fn from_yaml(yaml: &Yaml) -> Result<ConfigPlugin, Box<dyn std::error::Error>> {
        match yaml {
            Yaml::String(source) => match source.chars().nth(1) {
                Some('~' | '.' | '/') => Ok(ConfigPlugin {
                    source: PluginSource::Local {
                        source: PathBuf::from_str(&shellexpand::tilde(source))?,
                    },
                }),
                Some(_) | None => Err(format!("Unrecognized plugin source: {}", source).into()),
            },
            _ => Err("Invalid format for plugin".into()),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub machine: String,
    pub sources: Vec<PathBuf>,
    pub plugins: Vec<ConfigPlugin>,
}

impl Config {
    pub fn new(arguments: &Arguments) -> Result<Config, Box<dyn std::error::Error>> {
        // TODO: consider having ./nk.yml in the possible default config files
        let contents = std::fs::read_to_string(
            arguments
                .global
                .config
                .as_ref()
                .unwrap_or(&PathBuf::from_str(&shellexpand::tilde("~/.nk.yml"))?),
        )?;
        let yaml_documents = YamlLoader::load_from_str(&contents)?;
        // TODO: make sure only one document in the file
        let yaml = yaml_documents.get(0).ok_or("config file empty")?;

        // TODO: MAYBE: make sure no unrecognized options? at least warn
        // TODO: refactor
        Ok(Config {
            machine: match &yaml["machine"] {
                Yaml::String(me) => Ok(me),
                Yaml::BadValue => Err("Missing required config parameter machine"),
                _ => Err("Invalid format for machine"),
            }?
            .into(),
            // TODO: make sure not empty
            sources: match &yaml["sources"] {
                Yaml::Array(yamls) => yamls
                    .iter()
                    .map(|y| y.as_str())
                    .map(|s| s.ok_or("Invalid format for source"))
                    .map(|s| match s {
                        Ok(s) => Ok(PathBuf::from(shellexpand::tilde(&s).to_string())),
                        Err(err) => Err(err),
                    })
                    .collect(),
                Yaml::BadValue => Err("Missing required config parameter sources"),
                _ => Err("Invalid format for sources"),
            }?,
            plugins: match &yaml["plugins"] {
                Yaml::Array(yamls) => parse_plugins(yamls),
                Yaml::BadValue => Ok(vec![]),
                _ => Err("Invalid format for plugins".into()),
            }?,
        })
    }
}

fn parse_plugins(yamls: &[Yaml]) -> Result<Vec<ConfigPlugin>, Box<dyn std::error::Error>> {
    yamls.iter().map(ConfigPlugin::from_yaml).collect()
}
