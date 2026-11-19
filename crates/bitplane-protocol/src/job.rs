//! The root module's `job.*` methods and events (sessiond ADR 0034): work
//! that outlives the client that started it, with a phase, a log and
//! cancel. Modules contribute job kinds; `plane.create` is the only one.
//! Job state is broadcast, logs are scoped per connection by `job.watch`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Opaque and unique on the host.
pub type JobId = String;

/// The reply's entries to `job.list`, and the `job.changed` event's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: JobId,
    pub kind: JobKind,
    pub status: JobStatus,
    /// The furthest step the work has started; absent until it reports one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<JobPhase>,
    /// The `hello` client of the connection that started it, for display.
    pub started_by: JobClient,
    /// Unix milliseconds.
    #[cfg_attr(feature = "specta", specta(type = specta_typescript::Number))]
    pub started_at: u64,
    /// Unix milliseconds; set once `status` is no longer `running`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "specta", specta(type = Option<specta_typescript::Number>))]
    pub finished_at: Option<u64>,
    /// The kind's own: `plane.create`'s is a
    /// [`PlaneCreateDetail`](crate::plane::PlaneCreateDetail).
    #[cfg_attr(feature = "specta", specta(type = specta_typescript::Unknown))]
    pub detail: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum JobKind {
    /// Started by `plane.create`.
    #[serde(rename = "plane.create")]
    PlaneCreate,
    /// Any kind this build does not know, from a newer peer.
    #[serde(rename = "unknown")]
    Unknown,
}

crate::open_enum::deserialize_as_unknown!(JobKind, [PlaneCreate]);

/// For `plane.create`, the furthest step any member has started, from
/// `bp`'s progress events: fetch, worktree, script.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum JobPhase {
    Fetching,
    AddingWorktrees,
    RunningScripts,
    /// Any phase this build does not know, from a newer peer.
    Unknown,
}

crate::open_enum::deserialize_as_unknown!(JobPhase, [Fetching, AddingWorktrees, RunningScripts]);

/// Every state but `running` is final. For `plane.create` it is `bp`'s
/// answer, never what the disk looks like.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum JobStatus {
    Running,
    Succeeded,
    /// The plane was built and a script it ran failed.
    ScriptFailed,
    /// `bp` left nothing behind: it refused, or unwound everything.
    NothingCreated,
    /// `bp` could not take back all it made; the plane is a remnant.
    Partial,
    /// The work ended without an answer: `bp` could not be run, died, or
    /// the daemon restarted under it (`reason` is then `daemon restarted`).
    Failed {
        reason: String,
    },
    /// Any state this build does not know, from a newer peer.
    Unknown,
}

/// The `reason` of a job the daemon was restarted under.
pub const DAEMON_RESTARTED: &str = "daemon restarted";

impl JobStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, JobStatus::Running)
    }
}

/// Who started a job: the `hello` client's name and version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct JobClient {
    pub name: String,
    pub version: String,
}

/// The reply to `job.list`: running jobs and finished ones not yet
/// dismissed, oldest first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct JobListResult {
    pub jobs: Vec<Job>,
}

/// The `job.changed` event, broadcast.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct JobChanged {
    pub job: Job,
}

/// Names one job: `job.cancel` and `job.dismiss`, and the `job.removed`
/// event. `UNKNOWN_JOB` for a job the daemon does not have; `job.cancel`
/// on a finished job gets `JOB_FINISHED`, `job.dismiss` on a running one
/// `INVALID_REQUEST`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct JobRef {
    pub job: JobId,
}

/// `job.watch`: replaces this connection's log scope; `[]` stops. Each job
/// new to the scope gets its backlog as one `job.log`, then live lines
/// while it runs. A job the daemon does not have gets `job.removed` at once.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct JobWatchParams {
    pub jobs: Vec<JobId>,
}

/// The `job.log` event, sent only to connections watching the job. For
/// `plane.create` each line is `bp`'s stderr verbatim, without its newline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct JobLog {
    pub job: JobId,
    pub lines: Vec<String>,
    /// On a backlog only: how many earlier lines the capped log evicted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "specta", specta(type = Option<specta_typescript::Number>))]
    pub dropped: Option<u64>,
}

/// A `state` this build does not name decodes as `Unknown`; specta cannot
/// export `#[serde(other)]` on a tagged enum (see `open_enum`).
impl<'de> Deserialize<'de> for JobStatus {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "state", rename_all = "camelCase")]
        enum Known {
            Running,
            Succeeded,
            ScriptFailed,
            NothingCreated,
            Partial,
            Failed { reason: String },
        }
        let value = Value::deserialize(deserializer)?;
        if !matches!(
            value.get("state").and_then(|state| state.as_str()),
            Some(
                "running" | "succeeded" | "scriptFailed" | "nothingCreated" | "partial" | "failed"
            )
        ) {
            return Ok(JobStatus::Unknown);
        }
        Ok(
            match Known::deserialize(value).map_err(serde::de::Error::custom)? {
                Known::Running => JobStatus::Running,
                Known::Succeeded => JobStatus::Succeeded,
                Known::ScriptFailed => JobStatus::ScriptFailed,
                Known::NothingCreated => JobStatus::NothingCreated,
                Known::Partial => JobStatus::Partial,
                Known::Failed { reason } => JobStatus::Failed { reason },
            },
        )
    }
}
