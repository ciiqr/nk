use yaml_rust::Yaml;

#[derive(Debug)]
pub struct Condition {
    // TODO: decide if this is how we'll store this... maybe could parse and store parsed
    pub rule: String,
}

impl Condition {
    pub fn from_yaml(yaml: &Yaml) -> Result<Condition, String> {
        match yaml {
            Yaml::String(rule) => Ok(Condition { rule: rule.into() }),
            _ => Err("Invalid format for when: rule".into()),
        }
    }
}
