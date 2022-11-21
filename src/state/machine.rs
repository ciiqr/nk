use crate::traits::FromWithName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Machine {
    pub name: String,
    pub roles: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RawMachine {
    roles: Vec<String>,
}

impl FromWithName<RawMachine> for Machine {
    fn from_with_name(name: String, from: RawMachine) -> Self {
        let RawMachine { roles } = from;
        Machine { name, roles }
    }
}
