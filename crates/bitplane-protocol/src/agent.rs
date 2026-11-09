//! The `agent.claude` module's method and events: the Claude Code agents
//! running inside daemon sessions.

use serde::{Deserialize, Serialize};

use crate::session::SessionId;

/// Claude Code's own session id, as its `SessionStart` hook reports it.
pub type AgentSessionId = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub agent_session: AgentSessionId,
    /// The session the agent runs in; `null` once that session is closed.
    pub session: Option<SessionId>,
    /// Where the agent session started; `claude --resume` finds it only there.
    pub cwd: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub state: AgentState,
    pub tracking: Tracking,
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
    /// `claude --resume` was typed; no `SessionStart` has arrived yet.
    Resuming,
    Exited,
    /// Any state this build does not know, from a newer peer.
    Unknown,
}

crate::open_enum::deserialize_tagged_as_unknown!(
    AgentState,
    "kind",
    [
        Idle,
        Busy,
        Waiting {
            reason: WaitingReason
        },
        Resuming,
        Exited
    ]
);

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
    pub agents: Vec<Agent>,
}

/// The `agent.claude.changed` event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AgentResult {
    pub agent: Agent,
}

/// Names one agent: the `agent.claude.removed` event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AgentRef {
    pub agent_session: AgentSessionId,
}
