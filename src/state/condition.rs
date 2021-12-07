use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, transparent)]
pub struct Condition {
    // TODO: decide if this is how we'll store this... maybe could parse and store parsed
    pub rule: String,
}
