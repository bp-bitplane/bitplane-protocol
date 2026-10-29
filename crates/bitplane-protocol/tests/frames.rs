use bitplane_protocol::{Frame, FrameError, decode_envelope};
use serde_json::Value;

#[test]
fn valid_vectors_encode_and_decode() {
    for vector in vectors()["valid"].as_array().unwrap() {
        let name = vector["name"].as_str().unwrap();
        let frame = frame_of(&vector["frame"]);
        let websocket = hex(&vector["websocket"]);
        let envelope = hex(&vector["envelope"]);

        assert_eq!(frame.encode(), websocket, "{name}: encode");
        assert_eq!(frame.encode_envelope(), envelope, "{name}: encode_envelope");
        assert_eq!(
            Frame::decode(&websocket),
            Ok(frame.clone()),
            "{name}: decode"
        );
        assert_eq!(
            decode_envelope(&envelope),
            Ok(Some((frame, envelope.len()))),
            "{name}: decode_envelope"
        );
    }
}

#[test]
fn invalid_vectors_are_refused() {
    for vector in vectors()["invalid"].as_array().unwrap() {
        let name = vector["name"].as_str().unwrap();
        let error = decode_envelope(&hex(&vector["envelope"])).unwrap_err();
        assert_eq!(
            error_name(&error),
            vector["error"].as_str().unwrap(),
            "{name}"
        );
    }
}

#[test]
fn incomplete_vectors_need_more_bytes() {
    for vector in vectors()["incomplete"].as_array().unwrap() {
        let name = vector["name"].as_str().unwrap();
        assert_eq!(
            decode_envelope(&hex(&vector["envelope"])),
            Ok(None),
            "{name}"
        );
    }
}

#[test]
fn envelopes_decode_back_to_back() {
    let first = Frame::Input {
        attachment: 1,
        data: b"a".to_vec(),
    };
    let second = Frame::Json(b"{}".to_vec());
    let mut buf = first.encode_envelope();
    buf.extend(second.encode_envelope());

    let (frame, used) = decode_envelope(&buf).unwrap().unwrap();
    assert_eq!(frame, first);
    assert_eq!(decode_envelope(&buf[used..]).unwrap().unwrap().0, second);
}

fn vectors() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-vectors/frames.json"
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn frame_of(value: &Value) -> Frame {
    let attachment = || value["attachment"].as_u64().unwrap() as u32;
    let offset = || value["offset"].as_u64().unwrap();
    let data = hex(&value["data"]);
    match value["kind"].as_str().unwrap() {
        "json" => Frame::Json(data),
        "snapshot" => Frame::Snapshot {
            attachment: attachment(),
            offset: offset(),
            data,
        },
        "output" => Frame::Output {
            attachment: attachment(),
            offset: offset(),
            data,
        },
        "input" => Frame::Input {
            attachment: attachment(),
            data,
        },
        other => panic!("unknown kind {other} in vectors"),
    }
}

fn error_name(error: &FrameError) -> &'static str {
    match error {
        FrameError::Empty => "empty",
        FrameError::UnknownKind(_) => "unknown_kind",
        FrameError::ShortBody(_) => "short_body",
        FrameError::TooLarge(_) => "too_large",
    }
}

fn hex(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}
