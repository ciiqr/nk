use crate::{config::Config, state};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    vec,
};

#[derive(Debug)]
pub struct ProvisionArgs {
    pub dry_run: bool,
}

// TODO: wrap most errors in our own, more user friendly error
pub fn provision(args: ProvisionArgs, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // find all state files for this machine
    let role_names = get_current_machines_role_names(&config)?;
    let roles = find_roles(&role_names, &config.sources)?;
    let files = find_files(&roles)?;

    // TODO: filter based on "when:" conditions (files[].groups[].when)
    // TODO: load plugins
    // TODO: match states to plugins
    // TODO: run plugins

    // TODO: change this to use a propper logger
    println!("TODO: implement provision:");
    println!("{:?}", args);
    println!("{:?}", config);
    println!("{:?}", roles);
    for file in files {
        println!("{:#?}", file);
    }

    Ok(())
}

// TODO: move
fn find_files(roles: &[Role]) -> Result<Vec<state::File>, Box<dyn std::error::Error>> {
    let mut files: Vec<state::File> = vec![];

    // TODO: order may matter? probs fine to just have roles ordered tho
    // TODO: maybe not though, consider if we want to explicitly alphabetize files (within roles? or role sources? def not higher)
    for role in roles {
        for source in &role.sources {
            for res in std::fs::read_dir(source)? {
                let dir_entry = res?;
                let metadata = dir_entry.metadata()?;
                let path = dir_entry.path();
                let extension = path
                    .extension()
                    .ok_or("TODO: couldn't get extension for file..")?;

                if metadata.is_file() && extension == "yml" {
                    // TODO: files may want to store a Rc reference to their Role (or something like that...)
                    files.push(state::File::from_path(path)?);
                } else {
                    // TODO: likely ignore, but log (debug level)
                    // println!("ignoring: {:?}", path);
                }
            }
        }
    }

    Ok(files)
}

#[derive(Debug)]
pub struct Role {
    pub name: String,
    pub sources: Vec<PathBuf>,
}

fn find_roles(
    role_names: &[String],
    sources: &[PathBuf],
) -> Result<Vec<Role>, Box<dyn std::error::Error>> {
    let mut roles: Vec<Role> = vec![];

    // TODO: explicitly look for each role name, this way we end up with the roles ordered
    for source in sources {
        for res in std::fs::read_dir(source)? {
            let dir_entry = res?;
            let metadata = dir_entry.metadata()?;
            let path = dir_entry.path();
            let file_name = path
                .file_name()
                .ok_or("TODO: couldn't get filename for file...")?
                .to_str()
                .ok_or("TODO: could not convert os string to utf-8")?;

            // role source
            if metadata.is_dir() && role_names.contains(&file_name.into()) {
                roles.push(Role {
                    name: file_name.into(),
                    // TODO: we want all the sources in one role, not one role per-source...
                    sources: vec![path],
                });
            } else {
                // TODO: likely ignore, but log (debug level)
                // println!("ignoring: {:?}", path);
            }
        }
    }

    Ok(roles)
}

fn get_current_machines_role_names(
    config: &Config,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let machine_files: Vec<PathBuf> = config
        .sources
        .iter()
        .map(|source| source.join("machines.yml"))
        .filter(|machine_file_path| machine_file_path.is_file())
        .collect();

    let mut machines = HashMap::new();

    for machine_file in machine_files {
        for machine in parse_machines_from_path(&machine_file)? {
            if machines.get_key_value(&machine.name).is_some() {
                return Err(format!("Machine {} defined more than once", machine.name).into());
            }

            machines.insert(machine.name.clone(), machine);
        }
    }

    // TODO: above should be separate func
    let current_machine = machines
        .get(&config.machine)
        .ok_or("Current machine not found")?;

    Ok(current_machine.roles.clone())
}

// TODO: move
#[derive(Debug, Clone)]
pub struct Machine {
    pub name: String,
    pub roles: Vec<String>,
}

use yaml_rust::{Yaml, YamlLoader};
impl Machine {
    pub fn from_yaml(name: String, yaml: &Yaml) -> Result<Machine, String> {
        Ok(Machine {
            name,
            // TODO: handle unrecognized options
            roles: match &yaml["roles"] {
                Yaml::Array(yamls) => yamls
                    .iter()
                    .map(|y| y.to_owned().into_string())
                    .map(|s| s.ok_or("Invalid format for roles"))
                    .collect(),
                _ => Err("Invalid format for machine"),
            }?,
        })
    }
}

fn parse_machines_from_path(path: &Path) -> Result<Vec<Machine>, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;
    let yaml_documents = YamlLoader::load_from_str(&contents)?;
    // TODO: make sure only one document in the file
    let yaml = yaml_documents.get(0).ok_or("config file empty")?;

    match parse_machines_from_yaml(yaml) {
        Ok(machines) => Ok(machines),
        Err(err) => Err(err.into()),
    }
}

fn parse_machines_from_yaml(yaml: &Yaml) -> Result<Vec<Machine>, String> {
    match yaml {
        Yaml::Hash(hash) => {
            let mut err: Result<(), String> = Ok(());

            let res: Result<Vec<Machine>, String> = hash
                .iter()
                .filter_map(|(k, v)| match k {
                    Yaml::String(key) => Some((key, v)),
                    _ => {
                        // TODO: there's gotta be a cleaner way...
                        err = Err("Invalid type for top level key".into());
                        None
                    }
                })
                .map(|(k, v)| Machine::from_yaml(k.into(), v))
                .collect();

            // Check for key type errors
            err?;
            res
        }
        Yaml::Null => Ok(vec![]), // allow empty files
        _ => Err("Invalid format for machines".into()),
    }
}
