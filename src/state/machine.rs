use crate::{
    config::Config, extensions::SerdeDeserializeFromYamlPath, traits::FromWithName,
    utils::deserialize_map_to_vec_of_named,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

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

    pub fn get_current(config: &Config) -> Result<Machine, Box<dyn std::error::Error>> {
        let machines = find_machines(&config.sources)?;
        println!("machines: {:#?}", machines); // TODO: remove
        Ok(machines
            .into_iter()
            .find(|m| m.name == config.machine)
            .ok_or("Current machine not found")?)
    }
}

// find

fn find_machine_files(sources: &[PathBuf]) -> Vec<PathBuf> {
    sources
        .iter()
        .map(|source| source.join("machines.yml"))
        .filter(|machine_file_path| machine_file_path.is_file())
        .collect()
}

fn find_machines(sources: &[PathBuf]) -> Result<Vec<Machine>, Box<dyn std::error::Error>> {
    let mut machine_names = HashSet::new();
    let mut machines = vec![];

    for machine_file in find_machine_files(sources) {
        for machine in Machine::all_from_path(&machine_file)? {
            if machine_names.contains(&machine.name) {
                return Err(format!("Machine {} defined more than once", machine.name).into());
            }

            machine_names.insert(machine.name.clone());
            machines.push(machine);
        }
    }

    Ok(machines)
}

// serde

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
