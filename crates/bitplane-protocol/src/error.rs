use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A JSON-RPC error. Protocol errors use [`RpcError::SERVER_ERROR`] and name
/// themselves in `data.code`; the numeric codes stay JSON-RPC's own.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<ErrorData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ErrorData {
    pub code: ErrorCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "specta", specta(type = Option<specta_typescript::Unknown>))]
    pub detail: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// The first request on a connection was not `hello`.
    HelloRequired,
    /// `hello` was already answered on this connection.
    HelloRepeated,
    /// No protocol version both ends speak; `detail` carries the daemon's range.
    VersionMismatch,
    ModuleUnavailable,
    /// Any code this build does not know, from a newer peer.
    Unknown,
}

crate::open_enum::deserialize_as_unknown!(
    ErrorCode,
    [
        HelloRequired,
        HelloRepeated,
        VersionMismatch,
        ModuleUnavailable
    ]
);

impl RpcError {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    pub const SERVER_ERROR: i32 = -32000;

    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn protocol(code: ErrorCode, message: impl Into<String>, detail: Option<Value>) -> Self {
        Self {
            code: Self::SERVER_ERROR,
            message: message.into(),
            data: Some(ErrorData { code, detail }),
        }
    }

    pub fn protocol_code(&self) -> Option<ErrorCode> {
        self.data.as_ref().map(|data| data.code)
    }
}
