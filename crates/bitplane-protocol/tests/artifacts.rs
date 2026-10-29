//! The committed TS types and JSON Schema must match the Rust types.
//! `UPDATE_ARTIFACTS=1 cargo test --test artifacts` rewrites them.

use std::path::Path;

#[test]
fn committed_typescript_is_current() {
    check(
        "ts/src/generated.ts",
        &bitplane_protocol::export::typescript(),
    );
}

#[test]
fn committed_json_schema_is_current() {
    check(
        "schema/protocol.schema.json",
        &bitplane_protocol::export::json_schema(),
    );
}

fn check(relative: &str, fresh: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    if std::env::var_os("UPDATE_ARTIFACTS").is_some() {
        std::fs::write(&path, fresh).unwrap();
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        fresh == committed,
        "{relative} is stale: run `UPDATE_ARTIFACTS=1 cargo test --test artifacts`"
    );
}
