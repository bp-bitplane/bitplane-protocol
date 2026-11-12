use bitplane_protocol::session::{
    AttachmentAckParams, AttachmentError, AttachmentOpenParams, Session, SessionBusyResult,
    SessionOpenParams, SessionRenameParams, SessionStatus,
};
use bitplane_protocol::{ErrorCode, method};
use serde_json::json;

#[test]
fn a_session_is_camel_case_with_its_status_tagged_by_state() {
    let session: Session = serde_json::from_value(json!({
        "id": "5b0c",
        "seq": 3,
        "root": "/planes/web",
        "plane": "web",
        "status": {"state": "exited", "code": 2},
        "size": {"cols": 120, "rows": 40},
        "attachments": 1,
    }))
    .unwrap();

    assert_eq!(session.status, SessionStatus::Exited { code: 2 });
    assert_eq!(session.plane.as_deref(), Some("web"));
    assert_eq!(
        serde_json::to_value(&session).unwrap()["status"],
        json!({"state": "exited", "code": 2})
    );
}

#[test]
fn an_unknown_session_state_decodes_as_unknown() {
    let status: SessionStatus =
        serde_json::from_value(json!({"state": "hibernating", "since": 1})).unwrap();
    assert_eq!(status, SessionStatus::Unknown);
}

#[test]
fn open_params_need_only_a_root() {
    let params: SessionOpenParams = serde_json::from_value(json!({"root": "/tmp"})).unwrap();
    assert_eq!(params.root, "/tmp");
    assert!(params.env.is_none() && params.plane.is_none() && params.agent.is_none());
}

#[test]
fn open_params_may_open_an_agent_terminal_that_resumes() {
    let params: SessionOpenParams = serde_json::from_value(
        json!({"root": "/r", "agent": {"kind": "claude", "resume": "a1", "cwd": "/r/web"}}),
    )
    .unwrap();
    let agent = params.agent.clone().unwrap();
    assert_eq!(
        (
            agent.kind.as_str(),
            agent.resume.as_deref(),
            agent.cwd.as_deref()
        ),
        ("claude", Some("a1"), Some("/r/web"))
    );
    let fresh: SessionOpenParams =
        serde_json::from_value(json!({"root": "/r", "agent": {"kind": "claude"}})).unwrap();
    assert_eq!(
        serde_json::to_value(&fresh).unwrap(),
        json!({"root": "/r", "agent": {"kind": "claude"}})
    );
}

#[test]
fn attachment_messages_use_the_spec_names() {
    let open: AttachmentOpenParams =
        serde_json::from_value(json!({"session": "s", "cols": 80, "rows": 24})).unwrap();
    assert_eq!((open.cols, open.rows), (80, 24));
    assert!(!open.read_only && !open.ignore_size);
    let ack: AttachmentAckParams =
        serde_json::from_value(json!({"attachment": 7, "bytes": 131072})).unwrap();
    assert_eq!(ack.bytes, 131072);
    let error = AttachmentError {
        attachment: 7,
        code: ErrorCode::ReadOnly,
        message: "attachment is read-only".into(),
    };
    assert_eq!(serde_json::to_value(error).unwrap()["code"], "READ_ONLY");
    assert_eq!(method::SESSION_ATTACHMENT_ACK, "session.attachment.ack");
}

#[test]
fn session_error_codes_round_trip() {
    for code in [
        "UNKNOWN_SESSION",
        "UNKNOWN_ATTACHMENT",
        "READ_ONLY",
        "SESSION_EXITED",
        "ROOT_GONE",
        "NOT_FOUND",
        "AGENT_LIVE",
    ] {
        let decoded: ErrorCode = serde_json::from_value(json!(code)).unwrap();
        assert_ne!(decoded, ErrorCode::Unknown, "{code}");
        assert_eq!(serde_json::to_value(decoded).unwrap(), code);
    }
}

#[test]
fn an_observer_attaches_read_only_and_ignoring_size() {
    let open: AttachmentOpenParams = serde_json::from_value(json!({
        "session": "s", "cols": 80, "rows": 24, "readOnly": true, "ignoreSize": true,
    }))
    .unwrap();
    assert!(open.read_only && open.ignore_size);
    assert_eq!(
        serde_json::to_value(&open).unwrap(),
        json!({"session": "s", "cols": 80, "rows": 24, "readOnly": true, "ignoreSize": true})
    );
}

#[test]
fn a_session_carries_its_name_title_foreground_and_root_gone() {
    let session: Session = serde_json::from_value(json!({
        "id": "5b0c",
        "seq": 1,
        "root": "/planes/web",
        "name": "api",
        "title": "vim main.rs",
        "foreground": "vim",
        "rootGone": true,
        "status": {"state": "running"},
        "size": {"cols": 80, "rows": 24},
        "attachments": 0,
    }))
    .unwrap();
    assert_eq!(session.name.as_deref(), Some("api"));
    assert_eq!(session.title.as_deref(), Some("vim main.rs"));
    assert_eq!(session.foreground.as_deref(), Some("vim"));
    assert!(session.root_gone);

    let older: Session = serde_json::from_value(json!({
        "id": "5b0c", "seq": 1, "root": "/r", "status": {"state": "running"},
        "size": {"cols": 80, "rows": 24}, "attachments": 0,
    }))
    .unwrap();
    assert!(!older.root_gone && older.name.is_none() && older.title.is_none());
    assert_eq!(serde_json::to_value(&older).unwrap()["rootGone"], false);
}

#[test]
fn an_agent_terminal_names_its_kind_and_a_regular_one_has_no_agent() {
    let session = json!({
        "id": "5b0c", "seq": 1, "root": "/r", "agent": {"kind": "claude"},
        "status": {"state": "spawnFailed", "error": "claude not found on PATH"},
        "rootGone": false, "size": {"cols": 80, "rows": 24}, "attachments": 0,
    });
    let decoded: Session = serde_json::from_value(session.clone()).unwrap();
    assert_eq!(decoded.agent.as_ref().unwrap().kind, "claude");
    assert_eq!(serde_json::to_value(&decoded).unwrap(), session);

    let regular: Session = serde_json::from_value(json!({
        "id": "5b0c", "seq": 1, "root": "/r", "status": {"state": "running"},
        "size": {"cols": 80, "rows": 24}, "attachments": 0,
    }))
    .unwrap();
    assert!(regular.agent.is_none());
    assert!(
        serde_json::to_value(&regular)
            .unwrap()
            .get("agent")
            .is_none()
    );
}

#[test]
fn lifecycle_methods_use_the_spec_names() {
    let rename: SessionRenameParams = serde_json::from_value(json!({"session": "s"})).unwrap();
    assert_eq!(rename.name, None);
    let rename: SessionRenameParams =
        serde_json::from_value(json!({"session": "s", "name": "api"})).unwrap();
    assert_eq!(rename.name.as_deref(), Some("api"));
    let open: SessionOpenParams =
        serde_json::from_value(json!({"root": "/r", "name": "api"})).unwrap();
    assert_eq!(open.name.as_deref(), Some("api"));
    let busy = SessionBusyResult {
        busy: true,
        process_name: Some("claude".into()),
    };
    assert_eq!(
        serde_json::to_value(busy).unwrap(),
        json!({"busy": true, "processName": "claude"})
    );
    let idle: SessionBusyResult = serde_json::from_value(json!({"busy": false})).unwrap();
    assert_eq!(idle.process_name, None);
    assert_eq!(method::SESSION_RESTART, "session.restart");
    assert_eq!(method::SESSION_BUSY, "session.busy");
    assert_eq!(method::SESSION_RENAME, "session.rename");
}
