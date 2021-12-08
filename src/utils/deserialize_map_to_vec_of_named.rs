use serde::{
    de::{Deserializer, MapAccess, Visitor},
    Deserialize,
};
use std::marker::PhantomData;

use crate::traits::{FromWithName, IntoWithName};

// TODO: maybe we can make this into a trait that IntoWithName types implement, then we could likely use it as: RawMachine::deserialize_map_to_vec_of_named
// TODO: OR EVEN BETTER: instead implement Deserialize for FromWithName<Raw> where Raw is Deserialize
// TODO: https://stackoverflow.com/a/46755370
// TODO: Serialize also
pub fn deserialize_map_to_vec_of_named<'de, Raw, Target, D>(
    deserializer: D,
) -> Result<Vec<Target>, D::Error>
where
    Raw: Deserialize<'de>,
    Target: FromWithName<Raw>,
    D: Deserializer<'de>,
{
    struct VecNamedVisitor<Raw, Target>(PhantomData<Raw>, PhantomData<Target>);

    impl<'de, Raw, Target> Visitor<'de> for VecNamedVisitor<Raw, Target>
    where
        Raw: Deserialize<'de>,
        Target: FromWithName<Raw>,
    {
        type Value = Vec<Target>;

        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = Vec::with_capacity(access.size_hint().unwrap_or(0));

            while let Some((name, value)) = access.next_entry::<String, Raw>()? {
                map.push(value.into_with_name(name));
            }

            Ok(map)
        }
    }

    deserializer.deserialize_map(VecNamedVisitor(PhantomData, PhantomData))
}
