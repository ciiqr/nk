use serde_yaml::Value;

#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub states: Vec<Value>,
}
