//! Generates the committed artifacts: TS types (ADR 0021) and the JSON Schema
//! record of wire changes.

use crate::{
    Capability, ClientInfo, DaemonInfo, ErrorCode, ErrorData, HelloParams, HelloResult, Id,
    Notification, Request, Response, RpcError, rpc::Outcome,
};

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
        .register::<Capability>();
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
    let schema = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "bitplane protocol",
        "$defs": generator.take_definitions(true),
    });
    serde_json::to_string_pretty(&schema).expect("schema serializes") + "\n"
}
