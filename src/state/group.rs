use super::{Condition, Declaration, RawDeclaration};
use crate::utils::deserialize_map_to_vec_of_named;
use serde::Deserialize;
use serde_with::{serde_as, OneOrMany};

// TODO: figure out how to map back to map of raw declarations for Serialize
#[serde_as]
#[derive(Deserialize, Debug, Clone)]
pub struct Group {
    #[serde_as(deserialize_as = "OneOrMany<_>")]
    #[serde(default)]
    pub when: Vec<Condition>,
    // TODO: tags? vars? (if they're not handled via state...)
    #[serde(
        flatten,
        deserialize_with = "deserialize_map_to_vec_of_named::<RawDeclaration, _, _>"
    )]
    pub declarations: Vec<Declaration>,
}
