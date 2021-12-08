use crate::traits::FromWithName;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub states: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, transparent)]
pub struct RawDeclaration {
    states: Vec<Value>,
}

impl FromWithName<RawDeclaration> for Declaration {
    fn from_with_name(name: String, from: RawDeclaration) -> Self {
        let RawDeclaration { states } = from;
        Declaration { name, states }
    }
}
