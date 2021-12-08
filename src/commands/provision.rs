use crate::{
    config::Config,
    extensions::SerdeDeserializeFromYamlPath,
    state::{self},
};
use std::{collections::HashSet, path::PathBuf, vec};

#[derive(Debug)]
pub struct ProvisionArgs {
    pub dry_run: bool,
}

// TODO: move plugins
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
// TODO: maybe bad name...
pub struct PluginDefinition {
    name: String,
    executable: String,
    #[serde(default)]
    when: Vec<String>, // TODO: maybe proper conditions later?
}

// TODO: wrap most errors in our own, more user friendly error
pub fn provision(args: ProvisionArgs, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // find all state files for this machine
    let machine = get_current_machine(&config)?;
    let roles = find_roles(&machine.roles, &config.sources);
    let files = find_files(&roles)?;

    // TODO: initialize base vars (machine, roles, ?sources)

    // TODO: filter based on "when:" conditions (files[].groups[].when)
    // TODO: load plugins
    // TODO: call setup command on all plugins to determine how to interface with them? maybe only once required?
    // TODO: maybe move plugin config to sources... likely with "when:" conditions OR:
    // TODO: maybe NEED a "plugin.yml" to add basic "when:" conditions

    // TODO: match states to plugins

    // TODO: run plugins:
    for plugin in &config.plugins {
        // TODO: create a set of proper plugin objects with below process logic
        match &plugin.source {
            crate::config::PluginSource::Local { source } => {
                // TODO: support other state formats...
                let plugin_yml = source.join("plugin.yml");
                let plugin_definition = PluginDefinition::from_yaml_file(&plugin_yml)?;
                let plugin_executable = source.join(&plugin_definition.executable);
                println!("plugin_definition: {:?}", plugin_definition);

                let child = std::process::Command::new(plugin_executable)
                    .arg("provision")
                    .arg("--state")
                    .arg("example state")
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .expect("failed to execute child"); // TODO: handle errors smoother
                let output = child.wait_with_output().expect("failed to wait on child"); // TODO: handle errors smoother
                assert!(output.status.success()); // TODO: handle errors smoother

                // TODO: print stdout/stderr? as applicable
                // let mut me = output.stdout.as_mut().unwrap();
                println!("output: {:#?}", output);
                // println!("stdout: {:?}", String::from_utf8(output.stdout));
            }
        }
    }

    // TODO: change this to use a propper logger
    println!("TODO: implement provision:");
    println!("{:?}", args);
    println!("{:#?}", config);
    println!("{:?}", machine);
    println!("{:#?}", roles);
    for file in files {
        println!("{:#?}", file);
    }

    Ok(())
}

// TODO: move files
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

// TODO: move roles
#[derive(Debug)]
pub struct Role {
    pub name: String,
    pub sources: Vec<PathBuf>,
}

fn find_role_sources(role_name: &str, sources: &[PathBuf]) -> Vec<PathBuf> {
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

// TODO: move machines
fn find_machine_files(sources: &[PathBuf]) -> Vec<PathBuf> {
    sources
        .iter()
        .map(|source| source.join("machines.yml"))
        .filter(|machine_file_path| machine_file_path.is_file())
        .collect()
}

fn find_machines(sources: &[PathBuf]) -> Result<Vec<state::Machine>, Box<dyn std::error::Error>> {
    let mut machine_names = HashSet::new();
    let mut machines = vec![];

    for machine_file in find_machine_files(sources) {
        for machine in state::Machine::all_from_path(&machine_file)? {
            if machine_names.contains(&machine.name) {
                return Err(format!("Machine {} defined more than once", machine.name).into());
            }

            machine_names.insert(machine.name.clone());
            machines.push(machine);
        }
    }

    Ok(machines)
}

fn get_current_machine(config: &Config) -> Result<state::Machine, Box<dyn std::error::Error>> {
    let machines = find_machines(&config.sources)?;
    println!("machines: {:#?}", machines);
    Ok(machines
        .into_iter()
        .find(|m| m.name == config.machine)
        .ok_or("Current machine not found")?)
}
