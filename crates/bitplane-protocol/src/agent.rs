//! The `agent` module's method and events: the coding agents running in
//! agent terminals, of every kind (ADR 0029).

use serde::{Deserialize, Serialize};

use crate::session::SessionId;

/// The agent's own session id, as its `SessionStart` hook reports it.
pub type AgentSessionId = String;

/// An agent kind, such as `claude`; open, so a newer daemon's kinds pass through.
pub type AgentKind = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub kind: AgentKind,
    pub agent_session: AgentSessionId,
    /// The session the agent runs in; `null` once that session is closed.
    pub session: Option<SessionId>,
    /// Where the agent session started; `claude --resume` finds it only there.
    pub cwd: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub state: AgentState,
    /// Why the last turn failed; cleared by the next prompt. Only hooks report it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<AgentError>,
    pub tracking: Tracking,
}

/// A turn that ended on an API error (Claude Code's `StopFailure`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AgentError {
    /// Claude Code's error type, such as `rate_limit` or `authentication_failed`.
    pub error: String,
    /// What the agent showed the user, when it said.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AgentState {
    Idle,
    /// A turn is running.
    Busy,
    Waiting {
        reason: WaitingReason,
    },
    /// The agent was launched resuming it; no `SessionStart` has arrived yet.
    Resuming,
    Exited,
    /// Any state this build does not know, from a newer peer.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum WaitingReason {
    Permission,
    Input,
    Other,
    /// Any reason this build does not know, from a newer peer.
    Unknown,
}

crate::open_enum::deserialize_as_unknown!(WaitingReason, [Permission, Input, Other]);

/// Where the agent's state comes from: its hooks, or only the polled
/// `claude agents --json` registry when the hooks did not load.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Tracking {
    Hooks,
    Poll,
    /// Any mode this build does not know, from a newer peer.
    Unknown,
}

crate::open_enum::deserialize_as_unknown!(Tracking, [Hooks, Poll]);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AgentListResult {
    pub kinds: Vec<AgentKindStatus>,
    pub agents: Vec<Agent>,
}

/// Whether an agent terminal of `kind` can be launched on this host, and
/// why not; also the `agent.kind.changed` event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AgentKindStatus {
    pub kind: AgentKind,
    pub available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// The `agent.changed` event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AgentResult {
    pub agent: Agent,
}

/// Names one agent: the `agent.removed` event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AgentRef {
    pub agent_session: AgentSessionId,
}

/// A `kind` this build does not name decodes as `Unknown`; specta cannot
/// export `#[serde(other)]` on a tagged enum (see `open_enum`).
impl<'de> Deserialize<'de> for AgentState {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "camelCase")]
        enum Known {
            Idle,
            Busy,
            Waiting { reason: WaitingReason },
            Resuming,
            Exited,
        }
        let value = serde_json::Value::deserialize(deserializer)?;
        if !matches!(
            value.get("kind").and_then(|kind| kind.as_str()),
            Some("idle" | "busy" | "waiting" | "resuming" | "exited")
        ) {
            return Ok(AgentState::Unknown);
        }
        Ok(
            match Known::deserialize(value).map_err(serde::de::Error::custom)? {
                Known::Idle => AgentState::Idle,
                Known::Busy => AgentState::Busy,
                Known::Waiting { reason } => AgentState::Waiting { reason },
                Known::Resuming => AgentState::Resuming,
                Known::Exited => AgentState::Exited,
            },
        )
    }
}
