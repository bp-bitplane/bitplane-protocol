//! Enums decode unknown variants as `Unknown`. `#[serde(other)]` would say
//! so, but specta cannot export it for unit or tagged enums, so the decode is
//! written out here.

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

/// For internally tagged enums with `rename_all = "camelCase"`: an unknown
/// or missing tag decodes as `Unknown`, a known tag with bad fields fails.
macro_rules! deserialize_tagged_as_unknown {
    ($ty:ident, $tag:literal, [$($variant:ident $({ $($field:ident: $field_ty:ty),* $(,)? })?),* $(,)?]) => {
        impl<'de> serde::Deserialize<'de> for $ty {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "camelCase")]
                enum Tag {
                    $($variant),*
                }
                #[derive(serde::Deserialize)]
                #[serde(tag = $tag, rename_all = "camelCase")]
                enum Known {
                    $($variant $({ $($field: $field_ty),* })?),*
                }
                let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
                let known_tag = value
                    .get($tag)
                    .is_some_and(|tag| tag.is_string() && <Tag as serde::Deserialize>::deserialize(tag).is_ok());
                if !known_tag {
                    return Ok($ty::Unknown);
                }
                Ok(
                    match <Known as serde::Deserialize>::deserialize(value).map_err(serde::de::Error::custom)? {
                        $(Known::$variant $({ $($field),* })? => $ty::$variant $({ $($field),* })?),*
                    },
                )
            }
        }
    };
}

pub(crate) use {deserialize_as_unknown, deserialize_tagged_as_unknown};
