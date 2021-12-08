use crate::{
    extensions::SerdeDeserializeFromYamlPath, traits::FromWithName,
    utils::deserialize_map_to_vec_of_named,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug)]
pub struct Machine {
    pub name: String,
    pub roles: Vec<String>,
}

impl Machine {
    pub fn all_from_path(path: &Path) -> Result<Vec<Machine>, Box<dyn std::error::Error>> {
        let root = Root::from_yaml_file(path)?;

        Ok(root.machines)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct RawMachine {
    roles: Vec<String>,
}

impl FromWithName<RawMachine> for Machine {
    fn from_with_name(name: String, from: RawMachine) -> Self {
        let RawMachine { roles } = from;
        Machine { name, roles }
    }
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
struct Root {
    // TODO: is there a way to make this support empty files?
    #[serde(deserialize_with = "deserialize_map_to_vec_of_named::<RawMachine, _, _>")]
    machines: Vec<Machine>,
}
