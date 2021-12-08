use super::{Group, Role};
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

    pub fn find_by_roles(roles: &[Role]) -> Result<Vec<File>, Box<dyn std::error::Error>> {
        let mut files: Vec<File> = vec![];

        for role in roles {
            for source in &role.sources {
                let mut source_files: Vec<File> = vec![];

                for res in std::fs::read_dir(source)? {
                    let dir_entry = res?;
                    let metadata = dir_entry.metadata()?;
                    let path = dir_entry.path();
                    let extension = path
                        .extension()
                        .ok_or("TODO: couldn't get extension for file..")?;

                    if metadata.is_file() && extension == "yml" {
                        // TODO: files may want to store a Rc reference to their Role (or something like that...)
                        source_files.push(File::from_path(path)?);
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
}
