//! The `git` module's methods and events: repo status, blobs at HEAD or the
//! index, and watching repos (sessiond ADR 0032). Repo paths are absolute.

use serde::{Deserialize, Serialize};

/// `git.status`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct GitStatusParams {
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct GitStatusResult {
    /// One per requested path, in order.
    pub repos: Vec<GitRepo>,
}

/// One repo path's state: also the `git.changed` event. Exactly one of
/// `status`, `gone` and `error` is set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct GitRepo {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<GitStatus>,
    /// The path is missing or not a repo; in an event, it left the scope.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub gone: bool,
    /// git ran and failed, in its own words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct GitStatus {
    pub head: GitHead,
    /// Absent without an upstream, or when the upstream is gone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracking: Option<GitTracking>,
    pub staged: Vec<GitChange>,
    pub unstaged: Vec<GitChange>,
    /// Repo-relative paths.
    pub untracked: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum GitHead {
    Branch {
        name: String,
    },
    /// `sha` is abbreviated to 7 characters.
    Detached {
        sha: String,
    },
    /// Any head this build does not know, from a newer peer.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct GitTracking {
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct GitChange {
    /// Repo-relative.
    pub path: String,
    pub kind: GitChangeKind,
    /// The original path of a rename or copy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum GitChangeKind {
    Modified,
    TypeChanged,
    Added,
    Deleted,
    Renamed,
    Copied,
    Conflicted,
    /// Any kind this build does not know, from a newer peer.
    Unknown,
}

crate::open_enum::deserialize_as_unknown!(
    GitChangeKind,
    [
        Modified,
        TypeChanged,
        Added,
        Deleted,
        Renamed,
        Copied,
        Conflicted
    ]
);

/// `git.show`: `path` is relative to `repo`. A blob missing at `rev`, or a
/// `repo` that is not a repo, gets `NOT_FOUND`. Replies with a [`Blob`](crate::fs::Blob).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GitShowParams {
    pub repo: String,
    pub path: String,
    pub rev: GitRev,
    /// Reply without `data` past this size; capped at [`READ_LIMIT`](crate::fs::READ_LIMIT), its default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "specta", specta(type = Option<specta_typescript::Number>))]
    pub max_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum GitRev {
    Head,
    Index,
    /// Any rev this build does not know, from a newer peer.
    Unknown,
}

crate::open_enum::deserialize_as_unknown!(GitRev, [Head, Index]);

/// A `kind` this build does not name decodes as `Unknown` (see `open_enum`).
impl<'de> Deserialize<'de> for GitHead {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "camelCase")]
        enum Known {
            Branch { name: String },
            Detached { sha: String },
        }
        let value = serde_json::Value::deserialize(deserializer)?;
        if !matches!(
            value.get("kind").and_then(|kind| kind.as_str()),
            Some("branch" | "detached")
        ) {
            return Ok(GitHead::Unknown);
        }
        Ok(
            match Known::deserialize(value).map_err(serde::de::Error::custom)? {
                Known::Branch { name } => GitHead::Branch { name },
                Known::Detached { sha } => GitHead::Detached { sha },
            },
        )
    }
}
