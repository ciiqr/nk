use super::Group;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug)]
pub struct File {
    pub path: String,
    pub groups: Vec<Group>,
}

impl File {
    pub fn from_path(path: String) -> Result<File, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(&path)?;
        let yaml_documents = YamlLoader::load_from_str(&contents)?;

        Ok(File {
            path,
            groups: parse_groups(&yaml_documents)?,
        })
    }
}

fn parse_groups(yamls: &Vec<Yaml>) -> Result<Vec<Group>, String> {
    yamls.iter().map(|c| Group::from_yaml(&c)).collect()
}
