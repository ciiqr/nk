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
        let groups = DOCUMENT_SEPERATOR
            .split(&contents)
            .map(serde_yaml::from_str)
            // TODO: maybe consider partitioning and showing all the errors instead...
            .collect::<Result<Vec<_>, _>>()?;

        Ok(File { path, groups })
    }

    fn find_all_in_dir(directory: &PathBuf) -> Result<Vec<File>, Box<dyn std::error::Error>> {
        let mut source_files: Vec<File> = vec![];

        // TODO: may want to make sources optional? (def at least for roles, can always log missing directories)
        let dir_results = match std::fs::read_dir(directory) {
            Ok(r) => Ok(r),
            Err(e) => Err(format!("{}: {}", e, directory.display())),
        }?;

        for res in dir_results {
            let dir_entry = res?;
            let metadata = dir_entry.metadata()?;
            let path = dir_entry.path();
            let extension = path
                .extension()
                .map(|x| x.to_owned())
                .unwrap_or_else(|| "".into());
            let lossy_file_stem = path
                .file_stem()
                .map(|x| x.to_string_lossy())
                .unwrap_or_else(|| "".into());

            if metadata.is_file() && extension == "yml" && !lossy_file_stem.starts_with('.') {
                // TODO: files may want to store a Rc reference to their Role (or something like that...)
                source_files.push(File::from_path(path)?);
            } else {
                // TODO: likely ignore, but log (debug level)
                // println!("ignoring: {}", path.display());
            }
        }

        // sort files (within each source), so all files from one source are alphabetical and before any of the files from the next source)
        source_files.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));

        Ok(source_files)
    }

    pub fn find_all(
        sources: &[PathBuf],
        roles: &[Role],
    ) -> Result<Vec<File>, Box<dyn std::error::Error>> {
        let mut files: Vec<File> = vec![];

        // top level
        for source in sources {
            files.append(&mut File::find_all_in_dir(source)?);
        }

        // by role
        for role in roles {
            for source in &role.sources {
                files.append(&mut File::find_all_in_dir(source)?);
            }
        }

        Ok(files)
    }
}
