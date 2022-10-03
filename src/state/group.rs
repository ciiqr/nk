use std::collections::HashMap;

use super::{Condition, Declaration, RawDeclaration};
use crate::utils::deserialize_map_to_map_of_named;
use serde::Deserialize;
use serde_with::{serde_as, OneOrMany};

#[serde_as]
#[derive(Deserialize, Debug, Clone)]
pub struct Group {
    #[serde_as(deserialize_as = "OneOrMany<_>")]
    #[serde(default)]
    pub when: Vec<Condition>,
    // TODO: vars (once they're actually useful)
    #[serde(
        flatten,
        deserialize_with = "deserialize_map_to_map_of_named::<RawDeclaration, _, _>"
    )]
    pub declarations: HashMap<String, Declaration>,
}
