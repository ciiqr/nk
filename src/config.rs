use std::{path::PathBuf, str::FromStr};

use crate::args::Arguments;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug)]
pub struct Config {
    pub machine: String,
    pub sources: Vec<PathBuf>,
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
                    .map(|y| y.to_owned().into_string())
                    .map(|s| s.ok_or("Invalid format for source"))
                    .map(|s| match s {
                        Ok(s) => Ok(PathBuf::from(shellexpand::tilde(&s).to_string())),
                        Err(err) => Err(err),
                    })
                    .collect(),
                Yaml::BadValue => Err("Missing required config parameter sources"),
                _ => Err("TODO: can't determine machine".into()),
            }?,
        })
    }
}
