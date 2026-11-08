use bitplane_protocol::session::{
    AttachmentAckParams, AttachmentError, AttachmentOpenParams, Session, SessionOpenParams,
    SessionStatus,
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
    assert!(params.env.is_none() && params.plane.is_none());
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
