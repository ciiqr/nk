use serde::{
    de::{Deserializer, MapAccess, Visitor},
    Deserialize,
};
use std::{collections::HashMap, marker::PhantomData};

use crate::traits::{FromWithName, IntoWithName};

pub fn deserialize_map_to_map_of_named<'de, Raw, Target, D>(
    deserializer: D,
) -> Result<HashMap<String, Target>, D::Error>
where
    Raw: Deserialize<'de>,
    Target: FromWithName<Raw>,
    D: Deserializer<'de>,
{
    struct MapNamedVisitor<Raw, Target>(PhantomData<Raw>, PhantomData<Target>);

    impl<'de, Raw, Target> Visitor<'de> for MapNamedVisitor<Raw, Target>
    where
        Raw: Deserialize<'de>,
        Target: FromWithName<Raw>,
    {
        type Value = HashMap<String, Target>;

        fn expecting(
            &self,
            formatter: &mut core::fmt::Formatter,
        ) -> core::fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map =
                HashMap::with_capacity(access.size_hint().unwrap_or(0));

            while let Some((name, value)) =
                access.next_entry::<String, Raw>()?
            {
                map.insert(name.clone(), value.into_with_name(name));
            }

            Ok(map)
        }
    }

    deserializer.deserialize_map(MapNamedVisitor(PhantomData, PhantomData))
}

// TODO: reduce duplication with above
pub fn deserialize_map_to_map_of_named_optional<'de, Raw, Target, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, Target>>, D::Error>
where
    Raw: Deserialize<'de>,
    Target: FromWithName<Raw>,
    D: Deserializer<'de>,
{
    struct MapNamedVisitor<Raw, Target>(PhantomData<Raw>, PhantomData<Target>);

    impl<'de, Raw, Target> Visitor<'de> for MapNamedVisitor<Raw, Target>
    where
        Raw: Deserialize<'de>,
        Target: FromWithName<Raw>,
    {
        type Value = Option<HashMap<String, Target>>;

        fn expecting(
            &self,
            formatter: &mut core::fmt::Formatter,
        ) -> core::fmt::Result {
            formatter.write_str("an optional map")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map =
                HashMap::with_capacity(access.size_hint().unwrap_or(0));

            while let Some((name, value)) =
                access.next_entry::<String, Raw>()?
            {
                map.insert(name.clone(), value.into_with_name(name));
            }

            Ok(Some(map))
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(self)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_option(MapNamedVisitor(PhantomData, PhantomData))
}
