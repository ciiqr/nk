use std::{error::Error, fs::File, io::BufReader, path::Path};

use serde::de::DeserializeOwned;

pub trait SerdeDeserializeFromYamlPath<T> {
    fn from_yaml_file(path: &Path) -> Result<T, Box<dyn Error>>;
}

impl<T> SerdeDeserializeFromYamlPath<T> for T
where
    T: DeserializeOwned,
{
    fn from_yaml_file(path: &Path) -> Result<T, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        match serde_yaml::from_reader(reader) {
            Ok(val) => Ok(val),
            Err(err) => Err(err.into()),
        }
    }
}
