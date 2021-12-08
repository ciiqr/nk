use super::{Condition, Declaration};
use crate::{traits::FromWithName, utils::deserialize_map_to_vec_of_named};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

// TODO: figure out how to map back to map of raw declarations for Serialize
#[derive(Deserialize, Debug)]
pub struct Group {
    #[serde(default)]
    pub when: Vec<Condition>,
    // TODO: tags? vars? (if they're not handled via state...)
    #[serde(
        flatten,
        deserialize_with = "deserialize_map_to_vec_of_named::<RawDeclaration, _, _>"
    )]
    pub declarations: Vec<Declaration>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, transparent)]
struct RawDeclaration {
    states: Vec<Value>,
}

impl FromWithName<RawDeclaration> for Declaration {
    fn from_with_name(name: String, from: RawDeclaration) -> Self {
        let RawDeclaration { states } = from;
        Declaration { name, states }
    }
}
