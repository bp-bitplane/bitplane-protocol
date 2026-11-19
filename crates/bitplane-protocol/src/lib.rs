//! Protocol v1 of the bitplane stack (ADR 0021, ADR 0024): wire types and
//! frame codecs, no logic.

pub mod agent;
pub mod error;
pub mod frame;
pub mod fs;
pub mod git;
pub mod hello;
mod open_enum;
pub mod plane;
pub mod rpc;
pub mod session;

pub use error::{ErrorCode, ErrorData, RpcError};
pub use frame::{Frame, FrameError, Kind, decode_envelope};
pub use hello::{
    Capability, ClientInfo, DaemonInfo, DaemonShutdownParams, HelloParams, HelloResult,
    MIN_PROTOCOL_VERSION, PROTOCOL_VERSION,
};
pub use rpc::{Id, Message, Notification, Request, Response};

pub mod method {
    pub const HELLO: &str = "hello";
    /// Replies once the sessions are saved or hung up, then the daemon exits.
    pub const DAEMON_SHUTDOWN: &str = "daemon.shutdown";

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

    pub const AGENT_LIST: &str = "agent.list";

    /// Events. `changed` carries the agent's `session` always, `null` once
    /// that session is closed; `kind.changed` follows a kind's availability.
    pub const AGENT_CHANGED: &str = "agent.changed";
    pub const AGENT_REMOVED: &str = "agent.removed";
    pub const AGENT_KIND_CHANGED: &str = "agent.kind.changed";

    pub const FS_LIST: &str = "fs.list";
    pub const FS_READ: &str = "fs.read";
    pub const FS_WATCH: &str = "fs.watch";

    /// Event, scoped to the connections watching the path.
    pub const FS_CHANGED: &str = "fs.changed";

    pub const GIT_STATUS: &str = "git.status";
    pub const GIT_SHOW: &str = "git.show";
    pub const GIT_WATCH: &str = "git.watch";

    /// Event, scoped to the connections watching the repo.
    pub const GIT_CHANGED: &str = "git.changed";

    pub const PLANE_LIST: &str = "plane.list";
    /// Re-runs `bp list`; the answer arrives as `plane.changed`.
    pub const PLANE_REFRESH: &str = "plane.refresh";
    pub const PLANE_PROJECTS: &str = "plane.projects";
    pub const PLANE_CREATE_DRAFT: &str = "plane.createDraft";
    pub const PLANE_DISPATCH: &str = "plane.dispatch";

    /// Event, broadcast: the whole model.
    pub const PLANE_CHANGED: &str = "plane.changed";
}
