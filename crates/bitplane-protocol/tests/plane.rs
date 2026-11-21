use bitplane_protocol::plane::{
    BpVersion, PlaneDispatchParams, PlaneInfo, PlaneKind, PlaneListError, PlaneMember, PlaneRow,
    PlanesModel, Project,
};
use bitplane_protocol::{ErrorCode, RpcError, method};
use serde_json::json;

#[test]
fn the_model_carries_planes_drafts_and_gone_rows() {
    let model = PlanesModel {
        planes_dir: "/p".into(),
        planes_dir_exists: true,
        refreshed_at: Some(1_700_000_000_000),
        list_error: None,
        planes: vec![
            PlaneRow {
                id: "auth".into(),
                directory: "/p/auth".into(),
                kind: PlaneKind::Plane,
                members: vec![PlaneMember {
                    member: "@api".into(),
                    path: "/p/auth/api".into(),
                }],
                findings: vec!["member_worktree_missing".into()],
            },
            PlaneRow {
                id: "old".into(),
                directory: "/p/old".into(),
                kind: PlaneKind::Gone,
                members: vec![],
                findings: vec![],
            },
        ],
    };

    let value = serde_json::to_value(&model).unwrap();

    assert_eq!(
        value,
        json!({
            "planesDir": "/p",
            "planesDirExists": true,
            "refreshedAt": 1_700_000_000_000u64,
            "planes": [
                {"id": "auth", "directory": "/p/auth", "kind": "plane",
                 "members": [{"member": "@api", "path": "/p/auth/api"}],
                 "findings": ["member_worktree_missing"]},
                {"id": "old", "directory": "/p/old", "kind": "gone", "members": [], "findings": []},
            ],
        })
    );
    assert_eq!(serde_json::from_value::<PlanesModel>(value).unwrap(), model);
}

#[test]
fn a_failed_list_carries_bps_envelope() {
    let envelope = json!({"error": "git_missing", "code": 4, "message": "git was not found", "problems": [], "remedy": null});
    let error = PlaneListError {
        message: "git was not found".into(),
        envelope: Some(envelope.clone()),
    };

    assert_eq!(
        serde_json::to_value(&error).unwrap(),
        json!({"message": "git was not found", "envelope": envelope})
    );
}

#[test]
fn an_unknown_plane_kind_decodes_as_unknown() {
    let kind: PlaneKind = serde_json::from_value(json!("creating")).unwrap();

    assert_eq!(kind, PlaneKind::Unknown);
}

#[test]
fn plane_info_names_bps_version_and_contract() {
    let info = PlaneInfo {
        bp: BpVersion {
            version: "0.1.0".into(),
            contract: 1,
        },
    };

    assert_eq!(
        serde_json::to_value(&info).unwrap(),
        json!({"bp": {"version": "0.1.0", "contract": 1}})
    );
}

#[test]
fn a_project_is_camel_cased() {
    let project = Project {
        name: "api".into(),
        source_repo_present: true,
    };

    assert_eq!(
        serde_json::to_value(&project).unwrap(),
        json!({"name": "api", "sourceRepoPresent": true})
    );
}

#[test]
fn dispatch_carries_bps_request_verbatim() {
    let request = json!({"action": "plane_list", "health": "full", "future": [1]});

    let params: PlaneDispatchParams =
        serde_json::from_value(json!({ "request": request })).unwrap();

    assert_eq!(params.request, request);
}

#[test]
fn a_refusal_names_itself_and_carries_the_envelope() {
    let envelope = json!({"error": "lock_timeout", "code": 5, "message": "busy", "problems": [], "remedy": null});
    let error = RpcError::protocol(ErrorCode::BpRefused, "busy", Some(envelope.clone()));

    let value = serde_json::to_value(&error).unwrap();

    assert_eq!(
        value["data"],
        json!({"code": "BP_REFUSED", "detail": envelope})
    );
    assert_eq!(
        serde_json::from_value::<ErrorCode>(json!("PLANE_EXISTS")).unwrap(),
        ErrorCode::PlaneExists
    );
}

#[test]
fn the_plane_methods_live_in_the_plane_namespace() {
    for name in [
        method::PLANE_LIST,
        method::PLANE_REFRESH,
        method::PLANE_PROJECTS,
        method::PLANE_CREATE_DRAFT,
        method::PLANE_DISPATCH,
        method::PLANE_CHANGED,
    ] {
        assert!(name.starts_with("plane."), "{name}");
    }
}
