use super::Group;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::PathBuf;

#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
    pub groups: Vec<Group>,
}
lazy_static! {
    static ref DOCUMENT_SEPERATOR: Regex = Regex::new(r"(?m)^---$").unwrap();
}

impl File {
    pub fn from_path(path: PathBuf) -> Result<File, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(&path)?;

        Ok(File {
            path,
            groups: DOCUMENT_SEPERATOR
                .split(&contents)
                .map(|doc| serde_yaml::from_str(doc))
                // TODO: we only want to filter end of stream, else we want an error...
                .filter(|res| res.is_ok())
                // TODO: fix proper
                .map(|res| res.unwrap())
                .collect(),
        })
    }
}
