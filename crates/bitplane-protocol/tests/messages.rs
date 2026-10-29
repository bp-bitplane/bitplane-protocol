use bitplane_protocol::{
    ClientInfo, ErrorCode, HelloParams, Id, Message, Request, hello::negotiate,
};
use serde_json::json;

#[test]
fn messages_are_told_apart_by_their_keys() {
    let request = json!({"jsonrpc": "2.0", "id": 1, "method": "hello", "params": {}});
    let notification = json!({"jsonrpc": "2.0", "method": "session.changed"});
    let response = json!({"jsonrpc": "2.0", "id": "a", "result": null});
    let error = json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "x"}});

    assert!(matches!(
        serde_json::from_value(request),
        Ok(Message::Request(_))
    ));
    assert!(matches!(
        serde_json::from_value(notification),
        Ok(Message::Notification(_))
    ));
    assert!(matches!(
        serde_json::from_value(response),
        Ok(Message::Response(_))
    ));
    assert!(matches!(
        serde_json::from_value(error),
        Ok(Message::Response(_))
    ));
}

#[test]
fn a_request_round_trips() {
    let request = Request::new(Id::Number(7), "hello", Some(json!({"a": 1})));
    let text = serde_json::to_string(&Message::Request(request.clone())).unwrap();
    assert_eq!(
        serde_json::from_str::<Message>(&text).unwrap(),
        Message::Request(request)
    );
}

#[test]
fn unknown_error_codes_decode_as_unknown() {
    let code: ErrorCode = serde_json::from_value(json!("FROM_THE_FUTURE")).unwrap();
    assert_eq!(code, ErrorCode::Unknown);
}

#[test]
fn negotiation_picks_the_newest_shared_version() {
    let client = |min, max| HelloParams {
        protocol_version: max,
        min_protocol_version: min,
        client: ClientInfo {
            name: "t".into(),
            version: "0".into(),
            via: None,
        },
    };
    assert_eq!(negotiate(&client(1, 3), 1, 2), Some(2));
    assert_eq!(negotiate(&client(1, 1), 1, 2), Some(1));
    assert_eq!(negotiate(&client(3, 4), 1, 2), None);
    assert_eq!(negotiate(&client(1, 1), 2, 2), None);
}
