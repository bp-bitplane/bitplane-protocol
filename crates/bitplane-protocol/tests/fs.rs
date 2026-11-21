use bitplane_protocol::fs::{
    Blob, FsChanged, FsEntry, FsEntryKind, FsListResult, FsReadParams, WatchParams,
};
use serde_json::json;

#[test]
fn a_listing_names_each_entry_with_its_kind_and_ignored_flag() {
    let result = FsListResult {
        entries: vec![
            FsEntry {
                name: ".git".into(),
                kind: FsEntryKind::Directory,
                ignored: true,
            },
            FsEntry {
                name: "a.txt".into(),
                kind: FsEntryKind::File,
                ignored: false,
            },
        ],
    };

    let value = serde_json::to_value(&result).unwrap();

    assert_eq!(
        value,
        json!({"entries": [
            {"name": ".git", "kind": "directory", "ignored": true},
            {"name": "a.txt", "kind": "file", "ignored": false},
        ]})
    );
    assert_eq!(
        serde_json::from_value::<FsListResult>(value).unwrap(),
        result
    );
}

#[test]
fn an_unknown_entry_kind_decodes_as_unknown() {
    let entry: FsEntry =
        serde_json::from_value(json!({"name": "s", "kind": "socket", "ignored": false})).unwrap();

    assert_eq!(entry.kind, FsEntryKind::Unknown);
}

#[test]
fn a_read_sends_max_bytes_only_when_set() {
    let plain = FsReadParams {
        path: "/p/a.txt".into(),
        max_bytes: None,
    };
    let capped = FsReadParams {
        path: "/p/a.txt".into(),
        max_bytes: Some(10),
    };

    assert_eq!(
        serde_json::to_value(&plain).unwrap(),
        json!({"path": "/p/a.txt"})
    );
    assert_eq!(
        serde_json::to_value(&capped).unwrap(),
        json!({"path": "/p/a.txt", "maxBytes": 10})
    );
}

#[test]
fn a_blob_past_the_limit_has_no_data() {
    let blob = Blob {
        size: 20,
        data: None,
    };

    assert_eq!(serde_json::to_value(&blob).unwrap(), json!({"size": 20}));
    assert_eq!(
        serde_json::from_value::<Blob>(json!({"size": 3, "data": "YWJj"})).unwrap(),
        Blob {
            size: 3,
            data: Some("YWJj".into())
        }
    );
}

#[test]
fn a_change_sends_gone_only_when_set() {
    let changed = FsChanged {
        path: "/p".into(),
        gone: false,
    };
    let gone = FsChanged {
        path: "/p".into(),
        gone: true,
    };

    assert_eq!(
        serde_json::to_value(&changed).unwrap(),
        json!({"path": "/p"})
    );
    assert_eq!(
        serde_json::to_value(&gone).unwrap(),
        json!({"path": "/p", "gone": true})
    );
    assert_eq!(
        serde_json::from_value::<FsChanged>(json!({"path": "/p"})).unwrap(),
        changed
    );
}

#[test]
fn an_empty_watch_stops() {
    assert_eq!(
        serde_json::from_value::<WatchParams>(json!({"paths": []})).unwrap(),
        WatchParams { paths: vec![] }
    );
}
