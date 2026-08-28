//! ACP planning-client transport.

use crate::{bus::InProcessBus, runtime::backend::RuntimeBackend};
use std::path::PathBuf;

// `SessionId` here is the ACP wire type, not `crate::ids::SessionId`. They are
// different identifiers: one is the harness's session handle, one is ours.
use agent_client_protocol::{
    schema::v1::{
        ContentBlock, NewSessionRequest, PromptRequest, SessionId, SessionNotification, TextContent,
    },
    AcpAgent, AcpAgentConfig,
};
use tokio::sync::broadcast;

pub use super::controls::{
    invoke_panel_subagent, replay_panel, ActivePlan, Checkpoint, CheckpointLedger, ContextView,
    ElicitationAction, ElicitationHistory, ElicitationMode, ElicitationProperty,
    ElicitationRequest, ElicitationResponse, ElicitationResult, ElicitationSchema, PanelReplay,
    PanelSubagentRequest, PermissionPosture, PlanItem, PlanItemStatus, PlanProjection, PlanUpdate,
    RestoreResult, SessionCommand, SteeringBoundary, TurnCompletion, WorkspaceSnapshot,
};

/// ACP transport for a planning conversation. Its SDK client is intentionally not exposed to
/// ordinary agent-session code.
pub struct PlanningAgent {
    agent: AcpAgent,
}

impl PlanningAgent {
    pub fn config(&self) -> &AcpAgentConfig {
        self.agent.config()
    }
}

/// Builds the ACP SDK's subprocess transport for a planning conversation over stdio.
///
/// Kept private so production callers must use `container_stdio_transport`.
fn planning_stdio_transport<I, S>(command: impl Into<PathBuf>, args: I) -> PlanningAgent
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    PlanningAgent {
        agent: AcpAgent::new(AcpAgentConfig::new(command).args(args)),
    }
}

/// Creates the ACP SDK transport through the selected container runtime, so the agent process
/// runs in its container. The runtime command is selected by trusted host configuration, never by
/// agent-provided input.
pub fn container_stdio_transport_for_backend<I, S>(
    backend: RuntimeBackend,
    container: impl Into<String>,
    command: impl Into<String>,
    args: I,
) -> PlanningAgent
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let executable = match backend {
        RuntimeBackend::Docker => "docker",
        RuntimeBackend::Sbx => "sbx",
    };
    let mut runtime_args = vec!["exec".into(), "-i".into(), container.into(), command.into()];
    runtime_args.extend(args.into_iter().map(Into::into));
    planning_stdio_transport(executable, runtime_args)
}

/// Creates the ACP SDK transport through Docker, preserving the historical default.
pub fn container_stdio_transport<I, S>(
    container: impl Into<String>,
    command: impl Into<String>,
    args: I,
) -> PlanningAgent
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    container_stdio_transport_for_backend(RuntimeBackend::Docker, container, command, args)
}

/// Builds the ACP `session/new` request for a planning conversation.
pub fn session_new(cwd: impl Into<PathBuf>) -> NewSessionRequest {
    NewSessionRequest::new(cwd).mcp_servers(vec![])
}

/// Builds the ACP `session/prompt` request with its text content.
pub fn session_prompt(
    session_id: impl Into<SessionId>,
    prompt: impl Into<String>,
) -> PromptRequest {
    PromptRequest::new(
        session_id,
        vec![ContentBlock::Text(TextContent::new(prompt))],
    )
}

/// Broadcasts streamed ACP `session/update` notifications to planning consumers.
#[derive(Clone)]
pub struct UpdateStream(InProcessBus<SessionNotification>);

impl UpdateStream {
    pub fn new(capacity: usize) -> Self {
        Self(InProcessBus::new(capacity))
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SessionNotification> {
        self.0.subscribe()
    }

    pub fn publish(&self, update: SessionNotification) {
        let _ = self.0.publish(update);
    }
}

#[cfg(test)]
mod transport {
    use std::path::Path;

    use super::*;

    #[test]
    fn transport_uses_agent_stdio() {
        let transport = planning_stdio_transport("agent", ["acp"]);

        assert_eq!(transport.config().command(), Path::new("agent"));
        assert_eq!(transport.config().arguments(), ["acp"]);
    }
}

#[cfg(test)]
mod runs_in_container {
    use std::path::Path;

    use super::*;

    #[test]
    fn attaches_stdio_to_the_agent_container() {
        let transport = container_stdio_transport("locus-agent-run-1", "agent", ["acp"]);

        assert_eq!(transport.config().command(), Path::new("docker"));
        assert_eq!(
            transport.config().arguments(),
            ["exec", "-i", "locus-agent-run-1", "agent", "acp"]
        );
    }

    #[test]
    fn attaches_stdio_to_an_sbx_agent_without_a_tty() {
        let transport = container_stdio_transport_for_backend(
            RuntimeBackend::Sbx,
            "locus-agent-run-1",
            "agent",
            ["acp"],
        );

        assert_eq!(transport.config().command(), Path::new("sbx"));
        assert_eq!(
            transport.config().arguments(),
            ["exec", "-i", "locus-agent-run-1", "agent", "acp"]
        );
    }
}

#[cfg(test)]
mod not_on_host {
    use std::path::Path;

    use super::*;

    #[test]
    fn agent_process_is_started_only_through_its_container() {
        let transport = container_stdio_transport("locus-agent-run-1", "agent", ["acp"]);

        assert_eq!(transport.config().command(), Path::new("docker"));
        assert_eq!(
            transport.config().arguments(),
            ["exec", "-i", "locus-agent-run-1", "agent", "acp"]
        );
    }
}

#[cfg(test)]
mod session_new {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn sets_cwd_and_empty_mcp_servers() {
        let request = session_new("/workspace");

        assert_eq!(request.cwd, PathBuf::from("/workspace"));
        assert!(request.mcp_servers.is_empty());
        assert_eq!(
            serde_json::to_value(request).expect("serialize session request")["mcpServers"],
            serde_json::json!([])
        );
    }
}

#[cfg(test)]
mod prompt_streams {
    use agent_client_protocol::schema::v1::{
        ContentChunk, SessionNotification, SessionUpdate, TextContent,
    };

    use super::*;

    #[tokio::test]
    async fn sends_text_prompts_and_broadcasts_streamed_updates() {
        let request = session_prompt("planning-1", "Draft a plan");
        assert_eq!(
            serde_json::to_value(request).expect("serialize prompt request"),
            serde_json::json!({
                "sessionId": "planning-1",
                "prompt": [{"type": "text", "text": "Draft a plan"}],
            })
        );

        let updates = UpdateStream::new(1);
        let mut subscription = updates.subscribe();
        let update = SessionNotification::new(
            "planning-1",
            SessionUpdate::AgentMessageChunk(ContentChunk::new(ContentBlock::Text(
                TextContent::new("First step"),
            ))),
        );
        updates.publish(update.clone());

        assert_eq!(
            subscription.recv().await.expect("receive streamed update"),
            update
        );
    }
}

#[cfg(test)]
mod planning_only {
    use super::*;

    #[test]
    fn exposes_acp_only_as_a_planning_transport() {
        let transport = planning_stdio_transport("agent", ["acp"]);

        assert_eq!(transport.config().command(), std::path::Path::new("agent"));
        assert_eq!(transport.config().arguments(), ["acp"]);
    }
}

#[cfg(test)]
mod update_mapping {
    use serde_json::json;

    use crate::services::telemetry::{AcpAdapter, Adapter, EventVerb};

    #[test]
    fn shared_session_updates_map_to_canonical_verbs() {
        let adapter = AcpAdapter;

        for (session_update, expected) in [
            ("AgentMessageChunk", EventVerb::Assistant),
            ("AgentThoughtChunk", EventVerb::Thinking),
            ("ToolCall", EventVerb::ToolCall),
        ] {
            let events = adapter
                .normalize(json!({
                    "method": "session/update",
                    "params": {"update": {"sessionUpdate": session_update}},
                }))
                .expect("normalize session update");

            assert_eq!(events.len(), 1);
            assert_eq!(events[0].verb, expected);
        }
    }
}

#[cfg(test)]
mod tool_status_split {
    use serde_json::json;

    use crate::services::telemetry::{AcpAdapter, Adapter, EventVerb};

    #[test]
    fn maps_only_terminal_tool_statuses_to_results() {
        let adapter = AcpAdapter;

        for (status, expected) in [
            ("completed", Some(EventVerb::ToolResult)),
            ("failed", Some(EventVerb::ToolError)),
            ("pending", None),
            ("in_progress", None),
        ] {
            let events = adapter
                .normalize(json!({
                    "method": "session/update",
                    "params": {"update": {"sessionUpdate": "ToolCallUpdate", "status": status}},
                }))
                .expect("normalize tool call update");

            assert_eq!(events.first().map(|event| event.verb), expected);
        }
    }
}

#[cfg(test)]
mod permission_request {
    use crate::ids::RunId;
    use serde_json::json;

    use crate::services::telemetry::{
        AcpAdapter, Adapter, EventCollector, EventVerb, PermissionAlarm,
    };

    #[test]
    fn request_permission_normalizes_and_raises_an_alarm() {
        let raw = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "session/request_permission",
            "params": {
                "sessionId": "planning-1",
                "toolCall": {"toolCallId": "call-1"},
                "options": [{"optionId": "allow-once", "name": "Allow once", "kind": "allow_once"}],
            },
        });
        let captured = AcpAdapter
            .normalize(raw.clone())
            .expect("normalize permission request");
        let collector = EventCollector::new(1);
        let mut alarms = collector.subscribe_alarms();

        let run = RunId::generate();
        let event = collector.capture(run, captured.into_iter().next().expect("event"));

        assert_eq!(event.verb, EventVerb::PermissionRequest);
        assert_eq!(event.raw, raw);
        // The alarm names the run that raised it.
        assert_eq!(
            alarms.try_recv().expect("permission alarm"),
            PermissionAlarm {
                run_id: run,
                seq: 0
            }
        );
    }
}

#[cfg(test)]
mod plan_update {
    use super::*;
    use crate::services::telemetry::EventVerb;
    use serde_json::json;

    #[test]
    fn projects_one_active_plan_without_a_new_event_verb() {
        let update = PlanUpdate::from_value(&json!({
            "planId": "plan-1",
            "items": [
                {"id": "inspect", "title": "Inspect", "status": "in_progress"},
                {"id": "test", "title": "Test", "status": "pending"}
            ]
        }))
        .expect("decode ACP plan update");
        let mut projection = PlanProjection::default();
        let active = projection.apply(update);

        assert_eq!(active.plan_id, "plan-1");
        assert_eq!(active.items.len(), 2);
        assert_eq!(active.items[0].status, PlanItemStatus::InProgress);
        assert_eq!(EventVerb::ALL.len(), 12);
    }
}

#[cfg(test)]
mod elicitation {
    use super::*;
    use serde_json::{json, Map, Value};

    #[test]
    fn accepts_defaults_and_rejects_invalid_values() {
        let schema = ElicitationSchema::from_value(&json!({
            "type": "object",
            "properties": {
                "mode": {"type": "string", "enum": ["safe", "fast"], "default": "safe"},
                "retries": {"type": "integer"}
            },
            "required": ["mode"]
        }))
        .unwrap();
        let request = ElicitationRequest::form("ask-1", "Choose", schema).unwrap();
        let mut history = ElicitationHistory::default();
        let mut accepted = Map::new();
        accepted.insert("retries".into(), Value::from(2));
        assert!(matches!(
            history
                .respond(
                    &request,
                    ElicitationResponse {
                        request_id: "ask-1".into(),
                        action: ElicitationAction::Accept,
                        values: accepted,
                    }
                )
                .unwrap(),
            ElicitationResult::Accepted(_)
        ));
        assert!(history
            .respond(
                &request,
                ElicitationResponse {
                    request_id: "ask-1".into(),
                    action: ElicitationAction::Accept,
                    values: [("mode".into(), json!("unsafe"))].into_iter().collect(),
                }
            )
            .is_err());
        assert!(matches!(
            history
                .respond(
                    &request,
                    ElicitationResponse {
                        request_id: "ask-1".into(),
                        action: ElicitationAction::Decline,
                        values: Map::new(),
                    }
                )
                .unwrap(),
            ElicitationResult::Declined
        ));
        assert!(matches!(
            history
                .respond(
                    &request,
                    ElicitationResponse {
                        request_id: "ask-1".into(),
                        action: ElicitationAction::Cancel,
                        values: Map::new(),
                    }
                )
                .unwrap(),
            ElicitationResult::Cancelled
        ));
    }
}

#[cfg(test)]
mod session_commands {
    use super::*;

    #[test]
    fn exposes_the_four_session_commands() {
        assert_eq!(
            SessionCommand::parse("/new-session"),
            Some(SessionCommand::NewSession)
        );
        assert_eq!(
            SessionCommand::parse("/compact"),
            Some(SessionCommand::Compact)
        );
        assert_eq!(
            SessionCommand::parse("/clear-context"),
            Some(SessionCommand::ClearContext)
        );
        assert_eq!(
            SessionCommand::parse("/context"),
            Some(SessionCommand::ViewContext)
        );
        assert!(SessionCommand::parse("ordinary prompt").is_none());
    }
}

#[cfg(test)]
mod steering_boundary {
    use super::*;
    use crate::ids::TurnId;

    #[test]
    fn queues_steer_at_boundary_and_stops_only_active_turn() {
        let mut boundary = SteeringBoundary::default();
        let active = TurnId::generate();
        boundary.begin_turn(active);
        boundary.queue_steer("continue").unwrap();
        assert!(boundary.stop_active_turn());
        let completed = boundary.finish_turn().unwrap();
        assert_eq!(completed.turn_id, active);
        assert!(completed.cancelled);
        assert_eq!(completed.next_steer.as_deref(), Some("continue"));
    }
}

#[cfg(test)]
mod panel_subagent {
    use super::*;
    use crate::runtime::invoke::{
        AgentRef, InvocationContext, InvocationLimits, InvocationSupervisor, NestedRunLauncher,
    };
    use anyhow::Result;

    #[derive(Default)]
    struct Launcher {
        plans: std::sync::Mutex<Vec<crate::runtime::invoke::NestedRunPlan>>,
    }

    impl NestedRunLauncher for Launcher {
        fn start(&self, plan: &crate::runtime::invoke::NestedRunPlan) -> Result<()> {
            self.plans.lock().unwrap().push(plan.clone());
            Ok(())
        }
    }

    #[test]
    fn panel_subagent_uses_the_bounded_invocation_path() {
        let launcher = Launcher::default();
        let supervisor = InvocationSupervisor::new(&launcher);
        let request = PanelSubagentRequest::new(
            crate::ids::RunId::generate(),
            "reviewer",
            1,
            "/var/lib/locus/repos/project.git",
            InvocationContext {
                ancestry: vec![AgentRef {
                    name: "root".into(),
                    version: 1,
                }],
                children_started: 0,
            },
        )
        .with_limits(InvocationLimits::workflow(2, 1).unwrap());
        let plan = invoke_panel_subagent(&supervisor, request).unwrap();
        assert_eq!(plan.agent, "reviewer");
        assert_eq!(launcher.plans.lock().unwrap().len(), 1);
    }
}

#[cfg(test)]
mod one_mapping_every_harness {
    use serde_json::json;

    use crate::services::telemetry::{AcpAdapter, Adapter, EventVerb};

    /// ACP is the only harness interface, so the mapping is shared rather than per harness.
    /// Two different harnesses must produce identical verbs from identical updates.
    #[test]
    fn second_acp_harness_reuses_the_protocol_mapping() {
        let updates = [
            ("AgentMessageChunk", None, EventVerb::Assistant),
            ("AgentThoughtChunk", None, EventVerb::Thinking),
            ("ToolCall", None, EventVerb::ToolCall),
            ("ToolCallUpdate", Some("completed"), EventVerb::ToolResult),
            ("ToolCallUpdate", Some("failed"), EventVerb::ToolError),
        ];

        for harness in ["cursor", "second-acp-harness"] {
            for (session_update, status, expected) in updates {
                let mut update = json!({"sessionUpdate": session_update});
                if let Some(status) = status {
                    update["status"] = json!(status);
                }
                let events = AcpAdapter
                    .normalize(json!({
                        "harness": harness,
                        "method": "session/update",
                        "params": {"update": update},
                    }))
                    .expect("normalize ACP update");

                assert_eq!(events.len(), 1, "{harness} {session_update}");
                assert_eq!(events[0].verb, expected, "{harness} {session_update}");
            }
        }
    }
}

#[cfg(test)]
mod mcp_always_empty {
    use super::*;

    #[test]
    fn every_planning_request_serializes_empty_mcp_servers() {
        for cwd in ["/workspace", "/workspace/another-project"] {
            let request = session_new(cwd);

            assert_eq!(
                serde_json::to_value(request).expect("serialize session request")["mcpServers"],
                serde_json::json!([])
            );
        }
    }
}
