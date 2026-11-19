use bitplane_protocol::job::{
    DAEMON_RESTARTED, Job, JobClient, JobKind, JobLog, JobPhase, JobStatus, JobWatchParams,
};
use bitplane_protocol::plane::{PlaneCreateDetail, PlaneCreateParams};
use bitplane_protocol::{ErrorCode, method};
use serde_json::json;

fn job(status: JobStatus) -> Job {
    Job {
        id: "j-1".into(),
        kind: JobKind::PlaneCreate,
        status,
        phase: Some(JobPhase::AddingWorktrees),
        started_by: JobClient {
            name: "bitplane-cc".into(),
            version: "0.4.0".into(),
        },
        started_at: 1_700_000_000_000,
        finished_at: None,
        detail: json!(PlaneCreateDetail {
            plane: Some("auth".into()),
            ..Default::default()
        }),
    }
}

#[test]
fn a_job_is_camel_cased_with_a_tagged_status() {
    let job = job(JobStatus::Running);

    let value = serde_json::to_value(&job).unwrap();

    assert_eq!(
        value,
        json!({
            "id": "j-1",
            "kind": "plane.create",
            "status": {"state": "running"},
            "phase": "addingWorktrees",
            "startedBy": {"name": "bitplane-cc", "version": "0.4.0"},
            "startedAt": 1_700_000_000_000u64,
            "detail": {"plane": "auth"},
        })
    );
    assert_eq!(serde_json::from_value::<Job>(value).unwrap(), job);
}

#[test]
fn a_failed_job_names_its_reason() {
    let status = JobStatus::Failed {
        reason: DAEMON_RESTARTED.into(),
    };

    let value = serde_json::to_value(&status).unwrap();

    assert_eq!(
        value,
        json!({"state": "failed", "reason": "daemon restarted"})
    );
    assert_eq!(serde_json::from_value::<JobStatus>(value).unwrap(), status);
    for (state, status) in [
        ("succeeded", JobStatus::Succeeded),
        ("scriptFailed", JobStatus::ScriptFailed),
        ("nothingCreated", JobStatus::NothingCreated),
        ("partial", JobStatus::Partial),
    ] {
        assert_eq!(
            serde_json::to_value(&status).unwrap(),
            json!({ "state": state })
        );
    }
}

#[test]
fn unknown_kinds_phases_and_states_decode_as_unknown() {
    assert_eq!(
        serde_json::from_value::<JobKind>(json!("plane.add")).unwrap(),
        JobKind::Unknown
    );
    assert_eq!(
        serde_json::from_value::<JobPhase>(json!("verifying")).unwrap(),
        JobPhase::Unknown
    );
    assert_eq!(
        serde_json::from_value::<JobStatus>(json!({"state": "paused", "until": 1})).unwrap(),
        JobStatus::Unknown
    );
}

#[test]
fn a_backlog_counts_dropped_lines_and_a_live_line_omits_it() {
    let backlog = JobLog {
        job: "j-1".into(),
        lines: vec!["fetching @api".into()],
        dropped: Some(12),
    };
    let live = JobLog {
        dropped: None,
        ..backlog.clone()
    };

    assert_eq!(
        serde_json::to_value(&backlog).unwrap(),
        json!({"job": "j-1", "lines": ["fetching @api"], "dropped": 12})
    );
    assert_eq!(
        serde_json::to_value(&live).unwrap(),
        json!({"job": "j-1", "lines": ["fetching @api"]})
    );
}

#[test]
fn create_carries_bps_request_verbatim() {
    let args = json!({"members": ["@api"], "branch": "feat", "id": null, "future": true});

    let params: PlaneCreateParams = serde_json::from_value(json!({ "args": args })).unwrap();
    let watch: JobWatchParams = serde_json::from_value(json!({"jobs": ["a", "b"]})).unwrap();

    assert_eq!(params.args, args);
    assert_eq!(watch.jobs, ["a", "b"]);
}

#[test]
fn the_job_errors_name_themselves() {
    for (name, code) in [
        ("UNKNOWN_JOB", ErrorCode::UnknownJob),
        ("JOB_FINISHED", ErrorCode::JobFinished),
    ] {
        assert_eq!(serde_json::to_value(code).unwrap(), json!(name));
        assert_eq!(
            serde_json::from_value::<ErrorCode>(json!(name)).unwrap(),
            code
        );
    }
}

#[test]
fn the_job_methods_live_in_the_job_namespace() {
    for name in [
        method::JOB_LIST,
        method::JOB_WATCH,
        method::JOB_CANCEL,
        method::JOB_DISMISS,
        method::JOB_CHANGED,
        method::JOB_REMOVED,
        method::JOB_LOG,
    ] {
        assert!(name.starts_with("job."), "{name}");
    }
    assert_eq!(method::PLANE_CREATE, "plane.create");
}
