use bitplane_protocol::agent::{Agent, AgentRef, AgentState, Tracking, WaitingReason};
use bitplane_protocol::method;
use serde_json::json;

#[test]
fn an_agent_always_carries_its_session_and_tags_its_state_by_kind() {
    let agent = Agent {
        agent_session: "a1".into(),
        session: None,
        cwd: "/planes/web".into(),
        name: None,
        state: AgentState::Waiting {
            reason: WaitingReason::Permission,
        },
        tracking: Tracking::Hooks,
    };

    assert_eq!(
        serde_json::to_value(&agent).unwrap(),
        json!({
            "agentSession": "a1",
            "session": null,
            "cwd": "/planes/web",
            "state": {"kind": "waiting", "reason": "permission"},
            "tracking": "hooks",
        })
    );
    let decoded: Agent = serde_json::from_value(serde_json::to_value(&agent).unwrap()).unwrap();
    assert_eq!(decoded, agent);
}

#[test]
fn unknown_agent_states_reasons_and_tracking_decode_as_unknown() {
    let state: AgentState = serde_json::from_value(json!({"kind": "dreaming"})).unwrap();
    assert_eq!(state, AgentState::Unknown);
    let state: AgentState =
        serde_json::from_value(json!({"kind": "waiting", "reason": "coffee"})).unwrap();
    assert_eq!(
        state,
        AgentState::Waiting {
            reason: WaitingReason::Unknown
        }
    );
    let tracking: Tracking = serde_json::from_value(json!("telepathy")).unwrap();
    assert_eq!(tracking, Tracking::Unknown);
}

#[test]
fn agent_methods_and_events_use_the_spec_names() {
    assert_eq!(method::AGENT_CLAUDE_LIST, "agent.claude.list");
    assert_eq!(method::AGENT_CLAUDE_CHANGED, "agent.claude.changed");
    assert_eq!(method::AGENT_CLAUDE_REMOVED, "agent.claude.removed");
    assert_eq!(
        serde_json::to_value(AgentRef {
            agent_session: "a1".into()
        })
        .unwrap(),
        json!({"agentSession": "a1"})
    );
}
