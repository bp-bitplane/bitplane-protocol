//! The committed TS types and JSON Schema must match the Rust types.

use codegen::{JSON_SCHEMA_PATH, TYPESCRIPT_PATH, json_schema, repo_root, typescript};

#[test]
fn committed_typescript_is_current() {
    check(TYPESCRIPT_PATH, &typescript());
}

#[test]
fn committed_json_schema_is_current() {
    check(JSON_SCHEMA_PATH, &json_schema());
}

fn check(relative: &str, fresh: &str) {
    let committed = std::fs::read_to_string(repo_root().join(relative)).unwrap_or_default();
    assert!(
        fresh == committed,
        "{relative} is stale: run `cargo run -p codegen`"
    );
}
