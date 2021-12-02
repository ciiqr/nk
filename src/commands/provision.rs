use crate::{config::Config, state};
use std::{
    collections::HashSet,
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
    let machine = get_current_machine(&config)?;
    let roles = find_roles(&machine.roles, &config.sources);
    let files = find_files(&roles)?;

    // TODO: filter based on "when:" conditions (files[].groups[].when)
    // TODO: load plugins
    // TODO: match states to plugins
    // TODO: run plugins

    // TODO: change this to use a propper logger
    println!("TODO: implement provision:");
    println!("{:?}", args);
    println!("{:?}", config);
    println!("{:?}", machine);
    println!("{:#?}", roles);
    for file in files {
        println!("{:#?}", file);
    }

    Ok(())
}

// TODO: move
fn find_files(roles: &[Role]) -> Result<Vec<state::File>, Box<dyn std::error::Error>> {
    let mut files: Vec<state::File> = vec![];

    for role in roles {
        for source in &role.sources {
            let mut source_files: Vec<state::File> = vec![];

            for res in std::fs::read_dir(source)? {
                let dir_entry = res?;
                let metadata = dir_entry.metadata()?;
                let path = dir_entry.path();
                let extension = path
                    .extension()
                    .ok_or("TODO: couldn't get extension for file..")?;

                if metadata.is_file() && extension == "yml" {
                    // TODO: files may want to store a Rc reference to their Role (or something like that...)
                    source_files.push(state::File::from_path(path)?);
                } else {
                    // TODO: likely ignore, but log (debug level)
                    // println!("ignoring: {:?}", path);
                }
            }

            // sort files (within each source), so all files from one source are alphabetical and before any of the files from the next source)
            source_files.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));

            files.append(&mut source_files);
        }
    }

    Ok(files)
}

#[derive(Debug)]
pub struct Role {
    pub name: String,
    pub sources: Vec<PathBuf>,
}

fn find_role_sources(role_name: &String, sources: &[PathBuf]) -> Vec<PathBuf> {
    sources
        .iter()
        .map(|source| source.join(role_name))
        .filter(|role_path| role_path.is_dir())
        .collect()
}

fn find_roles(role_names: &[String], sources: &[PathBuf]) -> Vec<Role> {
    role_names
        .iter()
        .map(|role_name| Role {
            name: role_name.into(),
            sources: find_role_sources(role_name, sources),
        })
        .collect()
}

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

    for machine_file in find_machine_files(&sources) {
        for machine in parse_machines_from_path(&machine_file)? {
            if machine_names.contains(&machine.name) {
                return Err(format!("Machine {} defined more than once", machine.name).into());
            }

            machine_names.insert(machine.name.clone());
            machines.push(machine);
        }
    }

    Ok(machines)
}

fn get_current_machine(config: &Config) -> Result<Machine, Box<dyn std::error::Error>> {
    let machines = find_machines(&config.sources)?;

    Ok(machines
        .into_iter()
        .find(|m| m.name == config.machine)
        .ok_or("Current machine not found")?)
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
