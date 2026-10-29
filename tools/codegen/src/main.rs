//! Rewrites the committed TS types and JSON Schema from the protocol crate.

use codegen::{JSON_SCHEMA_PATH, TYPESCRIPT_PATH, json_schema, repo_root, typescript};

fn main() -> std::io::Result<()> {
    let root = repo_root();
    std::fs::write(root.join(TYPESCRIPT_PATH), typescript())?;
    std::fs::write(root.join(JSON_SCHEMA_PATH), json_schema())?;
    Ok(())
}
