use bitplane_protocol::git::{
    GitChange, GitChangeKind, GitHead, GitRepo, GitRev, GitShowParams, GitStatus, GitStatusResult,
    GitTracking,
};
use bitplane_protocol::method;
use serde_json::json;

#[test]
fn a_status_carries_head_tracking_and_changes() {
    let result = GitStatusResult {
        repos: vec![GitRepo {
            path: "/planes/web".into(),
            status: Some(GitStatus {
                head: GitHead::Branch {
                    name: "main".into(),
                },
                tracking: Some(GitTracking {
                    ahead: 1,
                    behind: 0,
                }),
                staged: vec![GitChange {
                    path: "b.txt".into(),
                    kind: GitChangeKind::Renamed,
                    from: Some("a.txt".into()),
                }],
                unstaged: vec![GitChange {
                    path: "c.txt".into(),
                    kind: GitChangeKind::TypeChanged,
                    from: None,
                }],
                untracked: vec!["d.txt".into()],
            }),
            gone: false,
            error: None,
        }],
    };

    let value = serde_json::to_value(&result).unwrap();

    assert_eq!(
        value,
        json!({"repos": [{
            "path": "/planes/web",
            "status": {
                "head": {"kind": "branch", "name": "main"},
                "tracking": {"ahead": 1, "behind": 0},
                "staged": [{"path": "b.txt", "kind": "renamed", "from": "a.txt"}],
                "unstaged": [{"path": "c.txt", "kind": "typeChanged"}],
                "untracked": ["d.txt"],
            },
        }]})
    );
    assert_eq!(
        serde_json::from_value::<GitStatusResult>(value).unwrap(),
        result
    );
}

#[test]
fn a_gone_repo_sends_only_its_path_and_gone() {
    let repo = GitRepo {
        path: "/planes/web".into(),
        status: None,
        gone: true,
        error: None,
    };

    let value = serde_json::to_value(&repo).unwrap();

    assert_eq!(value, json!({"path": "/planes/web", "gone": true}));
    assert_eq!(serde_json::from_value::<GitRepo>(value).unwrap(), repo);
}

#[test]
fn unknown_heads_kinds_and_revs_decode_as_unknown() {
    let head: GitHead = serde_json::from_value(json!({"kind": "orphan", "x": 1})).unwrap();
    let kind: GitChangeKind = serde_json::from_value(json!("ignored")).unwrap();
    let rev: GitRev = serde_json::from_value(json!("stash")).unwrap();

    assert_eq!(head, GitHead::Unknown);
    assert_eq!(kind, GitChangeKind::Unknown);
    assert_eq!(rev, GitRev::Unknown);
}

#[test]
fn show_names_a_repo_relative_path_at_head_or_the_index() {
    let params = GitShowParams {
        repo: "/planes/web".into(),
        path: "src/a.rs".into(),
        rev: GitRev::Index,
        max_bytes: Some(1000),
    };

    let value = serde_json::to_value(&params).unwrap();

    assert_eq!(
        value,
        json!({"repo": "/planes/web", "path": "src/a.rs", "rev": "index", "maxBytes": 1000})
    );
    assert_eq!(
        serde_json::from_value::<GitShowParams>(value).unwrap(),
        params
    );
}

#[test]
fn the_git_methods_live_in_the_git_namespace() {
    for name in [
        method::GIT_STATUS,
        method::GIT_SHOW,
        method::GIT_WATCH,
        method::GIT_CHANGED,
    ] {
        assert!(name.starts_with("git."), "{name}");
    }
}
