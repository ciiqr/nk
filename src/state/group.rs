use super::{Condition, Declaration};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;
// TODO: figure out how to map back to map of raw declarations for Serialize
#[derive(Deserialize, Debug)]
pub struct Group {
    #[serde(default)]
    pub when: Vec<Condition>,
    // TODO: tags? vars? (if they're not handled via state...)
    #[serde(flatten, deserialize_with = "deserialize_maps")]
    pub declarations: Vec<Declaration>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, transparent)]
struct RawDeclaration {
    states: Vec<Value>,
}

// TODO: generalize
fn deserialize_maps<'de, D>(deserializer: D) -> Result<Vec<Declaration>, D::Error>
where
    D: ::serde::de::Deserializer<'de>,
{
    use ::serde::de::*;

    type DeclarationVec = Vec<Declaration>;

    struct DeclarationVecVisitor;

    impl<'de> Visitor<'de> for DeclarationVecVisitor {
        // The type that our Visitor is going to produce.
        type Value = DeclarationVec;

        // Format a message stating what data this Visitor expects to receive.
        fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            formatter.write_str("a map")
        }

        // Deserialize DeclarationVec from an abstract "map" provided by the
        // Deserializer. The MapAccess input is a callback provided by
        // the Deserializer to let us see each entry in the map.
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = DeclarationVec::with_capacity(access.size_hint().unwrap_or(0));

            // While there are entries remaining in the input, add them into our map.
            while let Some((name, value)) = access.next_entry()? {
                let RawDeclaration { states } = value;

                let value = Declaration {
                    name,
                    states: states,
                };
                map.push(value);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_map(DeclarationVecVisitor)
}
