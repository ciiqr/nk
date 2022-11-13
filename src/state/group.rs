use super::{Condition, Declaration, RawDeclaration};
use crate::utils::deserialize_map_to_map_of_named;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};
use serde_yaml::Mapping;
use std::collections::HashMap;

#[serde_as]
#[derive(Deserialize, Debug, Clone)]
pub struct Group {
    #[serde_as(deserialize_as = "OneOrMany<_>")]
    #[serde(default)]
    pub when: Vec<Condition>,
    #[serde(default)]
    pub vars: Mapping,
    #[serde(
        flatten,
        deserialize_with = "deserialize_map_to_map_of_named::<RawDeclaration, _, _>"
    )]
    pub declarations: HashMap<String, Declaration>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResolvedGroup {
    #[serde(default)]
    pub vars: Mapping,
    #[serde(
        flatten,
        deserialize_with = "deserialize_map_to_map_of_named::<RawDeclaration, _, _>"
    )]
    pub declarations: HashMap<String, Declaration>,
}

impl ResolvedGroup {
    pub fn new() -> Self {
        Self {
            vars: Mapping::new(),
            declarations: HashMap::new(),
        }
    }
}
