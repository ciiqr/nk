use super::Group;
use serde::Deserialize;
use std::{ffi::OsStr, path::PathBuf};

#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
    pub groups: Vec<Group>,
}

impl File {
    pub fn from_path(
        path: PathBuf,
    ) -> Result<File, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(&path)?;

        let groups = serde_yml::Deserializer::from_str(&contents)
            .map(Group::deserialize)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(File { path, groups })
    }

    fn find_all_in_dir(
        directory: &PathBuf,
    ) -> Result<Vec<File>, Box<dyn std::error::Error>> {
        let mut source_files: Vec<File> = vec![];

        // TODO: may want to make sources optional? (can always log missing directories)
        let dir_results = match std::fs::read_dir(directory) {
            Ok(r) => Ok(r),
            Err(e) => Err(format!("{}: {}", e, directory.display())),
        }?;

        for res in dir_results {
            let dir_entry = res?;
            let metadata = dir_entry.metadata()?;
            let path = dir_entry.path();
            let extension = path.extension().unwrap_or_default();
            let lossy_file_stem = path
                .file_stem()
                .map(OsStr::to_string_lossy)
                .unwrap_or_default();

            if metadata.is_file()
                && extension == "yml"
                && !lossy_file_stem.starts_with('.')
            {
                source_files.push(File::from_path(path)?);
            } else {
                // TODO: likely ignore, but log (debug level)
                // println!("ignoring: {}", path.display());
            }
        }

        // sort files (within each source), so all files from one source are alphabetical and before any of the files from the next source)
        source_files
            .sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));

        Ok(source_files)
    }

    pub fn find_all(
        sources: &[PathBuf],
    ) -> Result<Vec<File>, Box<dyn std::error::Error>> {
        let mut files: Vec<File> = vec![];

        // top level
        for source in sources {
            files.append(&mut File::find_all_in_dir(source)?);
        }

        Ok(files)
    }
}
