use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::RpcError;

pub const JSONRPC_VERSION: &str = "2.0";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Id {
    Number(#[cfg_attr(feature = "specta", specta(type = specta_typescript::Number))] i64),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Request {
    pub jsonrpc: String,
    pub id: Id,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "specta", specta(type = Option<specta_typescript::Unknown>))]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Notification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "specta", specta(type = Option<specta_typescript::Unknown>))]
    pub params: Option<Value>,
}

/// `id` is `null` only when the request it answers could not be read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Response {
    pub jsonrpc: String,
    pub id: Option<Id>,
    #[serde(flatten)]
    pub outcome: Outcome,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Outcome {
    Result(#[cfg_attr(feature = "specta", specta(type = specta_typescript::Unknown))] Value),
    Error(RpcError),
}

impl Request {
    pub fn new(id: Id, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            id,
            method: method.into(),
            params,
        }
    }
}

impl Notification {
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            method: method.into(),
            params,
        }
    }
}

impl Response {
    pub fn ok(id: Id, result: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            id: Some(id),
            outcome: Outcome::Result(result),
        }
    }

    pub fn err(id: Option<Id>, error: RpcError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            id,
            outcome: Outcome::Error(error),
        }
    }
}

/// Any JSON-RPC message, told apart by its keys: `method` with `id` is a
/// request, `method` alone a notification, anything else a response.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Notification(Notification),
    Response(Response),
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let value = Value::deserialize(deserializer)?;
        let has = |key| value.get(key).is_some();
        let message = if has("method") && has("id") {
            Message::Request(serde_json::from_value(value).map_err(D::Error::custom)?)
        } else if has("method") {
            Message::Notification(serde_json::from_value(value).map_err(D::Error::custom)?)
        } else {
            Message::Response(serde_json::from_value(value).map_err(D::Error::custom)?)
        };
        Ok(message)
    }
}
