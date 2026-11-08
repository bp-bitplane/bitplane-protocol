//! Protocol v1 of the bitplane stack (ADR 0021, ADR 0024): wire types and
//! frame codecs, no logic.

pub mod error;
pub mod frame;
pub mod hello;
mod open_enum;
pub mod rpc;
pub mod session;

pub use error::{ErrorCode, ErrorData, RpcError};
pub use frame::{Frame, FrameError, Kind, decode_envelope};
pub use hello::{
    Capability, ClientInfo, DaemonInfo, HelloParams, HelloResult, MIN_PROTOCOL_VERSION,
    PROTOCOL_VERSION,
};
pub use rpc::{Id, Message, Notification, Request, Response};

pub mod method {
    pub const HELLO: &str = "hello";

    pub const SESSION_LIST: &str = "session.list";
    pub const SESSION_OPEN: &str = "session.open";
    pub const SESSION_CLOSE: &str = "session.close";
    /// A new shell under the same id; the stream's offsets keep counting.
    pub const SESSION_RESTART: &str = "session.restart";
    pub const SESSION_BUSY: &str = "session.busy";
    pub const SESSION_RENAME: &str = "session.rename";
    pub const SESSION_ATTACHMENT_OPEN: &str = "session.attachment.open";
    pub const SESSION_ATTACHMENT_RESIZE: &str = "session.attachment.resize";
    pub const SESSION_ATTACHMENT_CLOSE: &str = "session.attachment.close";
    /// A client notification.
    pub const SESSION_ATTACHMENT_ACK: &str = "session.attachment.ack";

    /// Events.
    pub const SESSION_CHANGED: &str = "session.changed";
    pub const SESSION_REMOVED: &str = "session.removed";
    pub const SESSION_ATTACHMENT_ERROR: &str = "session.attachment.error";
}
