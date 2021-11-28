use std::{path::PathBuf, str::FromStr};

use crate::args::Arguments;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug)]
pub struct Config {
    pub machine: String,
    pub sources: Vec<String>,
}

impl Config {
    pub fn new(arguments: &Arguments) -> Result<Config, Box<dyn std::error::Error>> {
        // TODO: consider having ./nk.yml in the possible default config files
        let contents = std::fs::read_to_string(
            arguments
                .global
                .config
                .as_ref()
                .unwrap_or(&PathBuf::from_str(&expand_user("~/.nk.yml"))?),
        )?;
        let yaml_documents = YamlLoader::load_from_str(&contents)?;
        // TODO: use serde?
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
                    .collect(),
                Yaml::BadValue => Err("Missing required config parameter sources"),
                _ => Err("TODO: can't determine machine".into()),
            }?,
        })
    }
}

// TODO: move
fn expand_user(path: &str) -> String {
    // TODO: maybe want a more specific outcome, atm this will complain about no such file or directory
    if let Ok(home) = std::env::var("HOME") {
        return path.replace("~", &home);
    }

    return path.into();
}
