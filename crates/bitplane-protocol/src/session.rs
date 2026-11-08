//! The `session` module's methods and events: sessions, and attachments to them.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ErrorCode;

/// A stable opaque string (a UUID), also the shell's `BITPLANE_SESSION_ID`.
pub type SessionId = String;

/// Unique on its connection; SNAPSHOT, OUTPUT and INPUT frames carry it.
pub type AttachmentId = u32;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: SessionId,
    /// Creation order on this host.
    pub seq: u32,
    /// The session root: the directory the shell starts in.
    pub root: String,
    /// Opaque to the daemon: stored, exported as `BITPLANE_PLANE_ID`, returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plane: Option<String>,
    /// The user's name for the session, set by `session.rename`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The title the program set (OSC 0/2), kept apart from `name`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The name of the terminal's foreground process: the shell when idle,
    /// absent while no shell runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foreground: Option<String>,
    pub status: SessionStatus,
    /// The root no longer exists; a running shell keeps running, and
    /// `session.restart` gets `ROOT_GONE` until it is back.
    #[serde(default)]
    pub root_gone: bool,
    pub size: Size,
    /// How many attachments the session has, across all clients.
    pub attachments: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum SessionStatus {
    Running,
    Exited {
        code: i32,
    },
    /// The shell could not be started.
    SpawnFailed {
        error: String,
    },
    /// Any state this build does not know, from a newer peer.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Size {
    pub cols: u16,
    pub rows: u16,
}

/// `session.open`: `root` must be an existing directory. `env` is set on top
/// of the daemon's login-shell environment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SessionOpenParams {
    pub root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plane: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// The result of `session.open`, and the `session.changed` event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SessionResult {
    pub session: Session,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SessionListResult {
    pub sessions: Vec<Session>,
}

/// Names one session: the params of `session.close`, `session.restart` and
/// `session.busy`, and the `session.removed` event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SessionRef {
    pub session: SessionId,
}

/// `session.rename`: no `name` clears it. Replies with [`SessionResult`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SessionRenameParams {
    pub session: SessionId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// The result of `session.busy`: busy when the terminal's foreground process
/// is not the shell, and then that process's name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SessionBusyResult {
    pub busy: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_name: Option<String>,
}

/// `session.attachment.open`: the reply comes first, then exactly one
/// SNAPSHOT frame, then OUTPUT frames from the snapshot's offset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AttachmentOpenParams {
    pub session: SessionId,
    pub cols: u16,
    pub rows: u16,
    /// Its INPUT gets `READ_ONLY` instead of reaching the shell.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub read_only: bool,
    /// It never sets the session's size, even when it is the latest active.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ignore_size: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AttachmentOpenResult {
    pub attachment: AttachmentId,
    pub session: Session,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AttachmentResizeParams {
    pub attachment: AttachmentId,
    pub cols: u16,
    pub rows: u16,
}

/// Names one attachment: the params of `session.attachment.close`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AttachmentRef {
    pub attachment: AttachmentId,
}

/// The client notification `session.attachment.ack`: `bytes` is the
/// cumulative count of SNAPSHOT and OUTPUT data the client has drawn on this
/// attachment. The daemon keeps at most 128 KiB unacknowledged.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AttachmentAckParams {
    pub attachment: AttachmentId,
    #[cfg_attr(feature = "specta", specta(type = specta_typescript::Number))]
    pub bytes: u64,
}

/// The event `session.attachment.error`: INPUT frames have no reply, so
/// their errors come back here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AttachmentError {
    pub attachment: AttachmentId,
    pub code: ErrorCode,
    pub message: String,
}

/// A `state` this build does not name decodes as `Unknown`; specta cannot
/// export `#[serde(other)]` on a tagged enum (see `open_enum`).
impl<'de> Deserialize<'de> for SessionStatus {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "state", rename_all = "camelCase")]
        enum Known {
            Running,
            Exited { code: i32 },
            SpawnFailed { error: String },
        }
        let value = serde_json::Value::deserialize(deserializer)?;
        if !matches!(
            value.get("state").and_then(|state| state.as_str()),
            Some("running" | "exited" | "spawnFailed")
        ) {
            return Ok(SessionStatus::Unknown);
        }
        Ok(
            match Known::deserialize(value).map_err(serde::de::Error::custom)? {
                Known::Running => SessionStatus::Running,
                Known::Exited { code } => SessionStatus::Exited { code },
                Known::SpawnFailed { error } => SessionStatus::SpawnFailed { error },
            },
        )
    }
}
