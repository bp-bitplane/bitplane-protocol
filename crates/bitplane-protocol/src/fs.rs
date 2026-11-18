//! The `fs` module's methods and events: listing, reading and watching any
//! path the daemon's OS account can read (sessiond ADR 0032). Paths are
//! absolute.

use serde::{Deserialize, Serialize};

/// `fs.read` and `git.show` past this many bytes reply without `data`.
pub const READ_LIMIT: u64 = 8 * 1024 * 1024;

/// `fs.list`: a missing path gets `NOT_FOUND`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct FsPathParams {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct FsListResult {
    /// Sorted by name.
    pub entries: Vec<FsEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct FsEntry {
    pub name: String,
    /// What the entry is once symlinks are followed.
    pub kind: FsEntryKind,
    /// Matched by the containing repo's ignore rules, or `.git` and
    /// everything in it.
    pub ignored: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum FsEntryKind {
    File,
    Directory,
    /// Any kind this build does not know, from a newer peer.
    Unknown,
}

crate::open_enum::deserialize_as_unknown!(FsEntryKind, [File, Directory]);

/// `fs.read`: a missing path or a folder gets `NOT_FOUND`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FsReadParams {
    pub path: String,
    /// Reply without `data` past this size; capped at [`READ_LIMIT`], its default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "specta", specta(type = Option<specta_typescript::Number>))]
    pub max_bytes: Option<u64>,
}

/// A raw blob: the reply to `fs.read` and `git.show`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Blob {
    #[cfg_attr(feature = "specta", specta(type = specta_typescript::Number))]
    pub size: u64,
    /// The bytes, base64 (standard alphabet, padded); absent past `maxBytes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

/// `fs.watch` and `git.watch`: replaces this connection's scope of the
/// module; `[]` stops. Only paths new to the scope get an initial event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct WatchParams {
    pub paths: Vec<String>,
}

/// The `fs.changed` event: the watched folder `path` changed, or with
/// `gone` it cannot be watched and has left the scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct FsChanged {
    pub path: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub gone: bool,
}
