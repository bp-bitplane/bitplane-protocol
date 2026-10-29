//! Enums decode unknown variants as `Unknown`. `#[serde(other)]` would say
//! so, but specta cannot export it for unit enums, so the decode is written
//! out here against the derived `Serialize` names.

macro_rules! deserialize_as_unknown {
    ($ty:ident, [$($variant:ident),* $(,)?]) => {
        impl<'de> serde::Deserialize<'de> for $ty {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let name = <String as serde::Deserialize>::deserialize(deserializer)?;
                Ok([$($ty::$variant),*]
                    .into_iter()
                    .find(|known| serde_json::to_value(known).ok().as_ref().and_then(|v| v.as_str()) == Some(name.as_str()))
                    .unwrap_or($ty::Unknown))
            }
        }
    };
}

pub(crate) use deserialize_as_unknown;
