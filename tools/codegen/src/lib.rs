//! Generates the committed artifacts: TS types (ADR 0021) and the JSON Schema
//! record of wire changes.

use bitplane_protocol::{
    Capability, ClientInfo, DaemonInfo, ErrorCode, ErrorData, HelloParams, HelloResult, Id,
    Notification, Request, Response, RpcError,
    rpc::Outcome,
    session::{
        AttachmentAckParams, AttachmentError, AttachmentOpenParams, AttachmentOpenResult,
        AttachmentRef, AttachmentResizeParams, Session, SessionBusyResult, SessionListResult,
        SessionOpenParams, SessionRef, SessionRenameParams, SessionResult, SessionStatus, Size,
    },
};

pub const TYPESCRIPT_PATH: &str = "ts/src/generated.ts";
pub const JSON_SCHEMA_PATH: &str = "schema/protocol.schema.json";

pub fn typescript() -> String {
    let types = specta::Types::default()
        .register::<Id>()
        .register::<Request>()
        .register::<Notification>()
        .register::<Response>()
        .register::<Outcome>()
        .register::<RpcError>()
        .register::<ErrorData>()
        .register::<ErrorCode>()
        .register::<HelloParams>()
        .register::<ClientInfo>()
        .register::<HelloResult>()
        .register::<DaemonInfo>()
        .register::<Capability>()
        .register::<Session>()
        .register::<SessionStatus>()
        .register::<Size>()
        .register::<SessionOpenParams>()
        .register::<SessionResult>()
        .register::<SessionListResult>()
        .register::<SessionRef>()
        .register::<SessionRenameParams>()
        .register::<SessionBusyResult>()
        .register::<AttachmentOpenParams>()
        .register::<AttachmentOpenResult>()
        .register::<AttachmentResizeParams>()
        .register::<AttachmentRef>()
        .register::<AttachmentAckParams>()
        .register::<AttachmentError>();
    specta_typescript::Typescript::default()
        .header("// Generated from the bitplane-protocol crate. Do not edit.")
        .export(&types, specta_serde::PhasesFormat)
        .expect("protocol types export to TypeScript")
}

pub fn json_schema() -> String {
    let mut generator = schemars::SchemaGenerator::default();
    generator.subschema_for::<Request>();
    generator.subschema_for::<Notification>();
    generator.subschema_for::<Response>();
    generator.subschema_for::<HelloParams>();
    generator.subschema_for::<HelloResult>();
    generator.subschema_for::<SessionOpenParams>();
    generator.subschema_for::<SessionResult>();
    generator.subschema_for::<SessionListResult>();
    generator.subschema_for::<SessionRef>();
    generator.subschema_for::<SessionRenameParams>();
    generator.subschema_for::<SessionBusyResult>();
    generator.subschema_for::<AttachmentOpenParams>();
    generator.subschema_for::<AttachmentOpenResult>();
    generator.subschema_for::<AttachmentResizeParams>();
    generator.subschema_for::<AttachmentRef>();
    generator.subschema_for::<AttachmentAckParams>();
    generator.subschema_for::<AttachmentError>();
    let schema = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "bitplane protocol",
        "$defs": generator.take_definitions(true),
    });
    serde_json::to_string_pretty(&schema).expect("schema serializes") + "\n"
}

/// The repo root, where the artifact paths are relative to.
pub fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
