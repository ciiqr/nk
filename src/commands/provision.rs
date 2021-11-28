use std::{path::PathBuf, vec};

use crate::{config::Config, state};

#[derive(Debug)]
pub struct ProvisionArgs {
    pub dry_run: bool,
}

// TODO: wrap most errors in our own, more user friendly error
pub fn provision(args: ProvisionArgs, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // machine definitions
    let machine_files: Vec<PathBuf> = config
        .sources
        .iter()
        .map(|source| source.join("machines.yml"))
        .filter(|machine_file_path| machine_file_path.is_file())
        .collect();

    // TODO: parse machine_files
    for machine_file in machine_files {
        println!("machine_file: {:?}", machine_file);
    }
    // TODO: make sure config.machine defined in files (only once)
    // TODO: pull roles from parsed machine_files
    let role_names = vec![
        "base",
        "ssh",
        "git",
        "restic",
        "gpg",
        "frontend",
        "sublime",
        "syncthing",
        "development",
    ];

    // find all state files for this machine
    let roles = find_roles(role_names, &config.sources)?;
    let files = find_files(roles)?;
    // TODO: filter based on "when:" conditions (files[].groups[].when)

    // TODO: change this to use a propper logger
    for file in files {
        println!("{:#?}", file);
    }
    println!("TODO: implement provision: {:?}", args);
    println!("with: {:?}", config);

    Ok(())
}

// TODO: move
fn find_files(roles: Vec<Role>) -> Result<Vec<state::File>, Box<dyn std::error::Error>> {
    let mut files: Vec<state::File> = vec![];

    for role in roles {
        for source in role.sources {
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

    return Ok(files);
}

#[derive(Debug)]
pub struct Role {
    pub name: String,
    pub sources: Vec<PathBuf>,
}

fn find_roles(
    role_names: Vec<&str>,
    sources: &Vec<PathBuf>,
) -> Result<Vec<Role>, Box<dyn std::error::Error>> {
    let mut roles: Vec<Role> = vec![];

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
            if metadata.is_dir() && role_names.contains(&file_name) {
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

    return Ok(roles);
}
