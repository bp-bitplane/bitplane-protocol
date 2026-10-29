//! Protocol v1 of the bitplane stack (ADR 0021, ADR 0024): wire types and
//! frame codecs, no logic.

pub mod error;
pub mod frame;
pub mod hello;
mod open_enum;
pub mod rpc;

#[cfg(all(feature = "specta", feature = "schemars"))]
pub mod export;

pub use error::{ErrorCode, ErrorData, RpcError};
pub use frame::{Frame, FrameError, Kind, decode_envelope};
pub use hello::{
    Capability, ClientInfo, DaemonInfo, HelloParams, HelloResult, MIN_PROTOCOL_VERSION,
    PROTOCOL_VERSION,
};
pub use rpc::{Id, Message, Notification, Request, Response};

pub mod method {
    pub const HELLO: &str = "hello";
}
