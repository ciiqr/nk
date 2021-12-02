use yaml_rust::Yaml;

#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub states: Vec<Yaml>,
}

impl Declaration {
    pub fn from_yaml(name: String, yaml: Yaml) -> Result<Declaration, String> {
        Ok(Declaration {
            name,
            states: match yaml {
                Yaml::Array(arr) => Ok(arr),

                // singletons are converted to an array of 1
                Yaml::Null
                | Yaml::Integer(_)
                | Yaml::String(_)
                | Yaml::Boolean(_)
                | Yaml::Hash(_)
                | Yaml::Real(_) => Ok(vec![yaml]),

                _ => Err("Invalid format for declaration"),
            }?,
        })
    }
}
