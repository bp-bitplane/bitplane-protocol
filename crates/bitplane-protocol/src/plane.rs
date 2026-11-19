//! The `plane` module's methods and events: the sidebar model of planes and
//! drafts, the projects list, draft creation and any `bp` request passed
//! through (sessiond ADR 0033). `bp`'s own requests, responses and error
//! envelopes cross as opaque JSON (ADR 0021); their types are bitplane's.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::job::JobId;

/// The `plane` capability's `info` in `hello`: what `bp --version --json`
/// answered at startup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PlaneInfo {
    pub bp: BpVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct BpVersion {
    /// The binary's own version.
    pub version: String,
    /// The version of `bp`'s wire: its requests, responses, envelope and
    /// progress events.
    pub contract: u32,
}

/// The reply to `plane.list` and the `plane.changed` event: every plane and
/// draft in the planes directory, sorted by id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PlanesModel {
    pub planes_dir: String,
    pub planes_dir_exists: bool,
    /// When `bp list` last answered, in Unix milliseconds; absent until it
    /// first has, and a `plane.changed` follows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "specta", specta(type = Option<specta_typescript::Number>))]
    pub refreshed_at: Option<u64>,
    /// The last `bp list` failed; `planes` keeps what the one before found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_error: Option<PlaneListError>,
    pub planes: Vec<PlaneRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PlaneListError {
    pub message: String,
    /// `bp`'s error envelope when it refused, as with `BP_REFUSED`; absent
    /// when `bp` could not be run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "specta", specta(type = Option<specta_typescript::Unknown>))]
    pub envelope: Option<Value>,
}

/// One sidebar row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PlaneRow {
    /// The plane directory's name.
    pub id: String,
    /// Absolute.
    pub directory: String,
    pub kind: PlaneKind,
    /// In plane file order; empty unless `kind` is `plane`.
    pub members: Vec<PlaneMember>,
    /// `bp list`'s finding tags, such as `member_worktree_missing`; empty
    /// unless `kind` is `plane`.
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PlaneMember {
    /// `@project` or a repo path, as `bp create` takes it.
    pub member: String,
    /// The member's worktree, absolute: what `git.*` takes.
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum PlaneKind {
    /// A directory holding `plane.toml`, as `bp list` reports it.
    Plane,
    /// A directory in the planes directory without `plane.toml`.
    Draft,
    /// A plane or draft this daemon listed that has since vanished; the only
    /// gone-plane signal. It stays until the daemon restarts or the id is
    /// back.
    Gone,
    /// Any kind this build does not know, from a newer peer.
    Unknown,
}

crate::open_enum::deserialize_as_unknown!(PlaneKind, [Plane, Draft, Gone]);

/// The reply to `plane.projects`: `bp project list`, sorted by name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PlaneProjectsResult {
    pub projects: Vec<Project>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Without the `@`.
    pub name: String,
    /// `false` also for a project whose `project.toml` did not parse.
    pub source_repo_present: bool,
}

/// `plane.createDraft`: makes `<planesDir>/<id>`, and the planes directory
/// if needed. An id `bp` would refuse gets `INVALID_PARAMS`, an existing
/// entry `PLANE_EXISTS`. A `plane.changed` follows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PlaneCreateDraftParams {
    pub id: String,
}

/// `plane.dispatch`: one `bp` `Request`, run as `bp --stdin --json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PlaneDispatchParams {
    #[cfg_attr(feature = "specta", specta(type = specta_typescript::Unknown))]
    pub request: Value,
}

/// `bp`'s `Response`, from an exit of `0` or `3`. Any other exit with an
/// error envelope is `BP_REFUSED`; anything else is an internal error naming
/// the cause.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PlaneDispatchResult {
    #[cfg_attr(feature = "specta", specta(type = specta_typescript::Unknown))]
    pub response: Value,
}

/// `plane.create`: starts a job of kind `plane.create`, which runs
/// `bp --stdin --json --progress-fd 3` with `args` as its request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PlaneCreateParams {
    /// `bp`'s `PlaneCreateRequest`, opaque; the daemon tags it with
    /// `"action": "plane_create"`. Not an object gets `INVALID_PARAMS`.
    #[cfg_attr(feature = "specta", specta(type = specta_typescript::Unknown))]
    pub args: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PlaneCreateResult {
    pub job: JobId,
}

/// A `plane.create` job's `detail`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PlaneCreateDetail {
    /// The plane id: the request's, or once `bp` answers, its own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plane: Option<String>,
    /// Absolute; known once `bp` answers with a `Response`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
    /// `bp`'s `Response`, when it answered with one (an interrupted run too).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "specta", specta(type = Option<specta_typescript::Unknown>))]
    pub response: Option<Value>,
    /// `bp`'s error envelope, when it refused.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "specta", specta(type = Option<specta_typescript::Unknown>))]
    pub envelope: Option<Value>,
}
