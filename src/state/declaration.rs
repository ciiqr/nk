use crate::traits::FromWithName;
use serde::{Deserialize, Deserializer, Serialize};
use serde_yaml::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Declaration {
    pub name: String,
    pub states: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, transparent)]
pub struct RawDeclaration {
    #[serde(deserialize_with = "one_or_many")]
    states: Vec<Value>,
}

impl FromWithName<RawDeclaration> for Declaration {
    fn from_with_name(name: String, from: RawDeclaration) -> Self {
        let RawDeclaration { states } = from;
        Declaration { name, states }
    }
}

impl From<Declaration> for RawDeclaration {
    fn from(d: Declaration) -> Self {
        Self { states: d.states }
    }
}

impl Serialize for Declaration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let raw = RawDeclaration::from(self.clone());
        raw.serialize(serializer)
    }
}

// NOTE: serde_with::OneOrMany doesn't work here (presumably because we're working with raw values...)
fn one_or_many<'de, D>(deserializer: D) -> Result<Vec<serde_yaml::Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let res: serde_yaml::Value = Deserialize::deserialize(deserializer)?;
    match res {
        Value::Sequence(many) => Ok(many),
        one => Ok(vec![one]),
    }
}
