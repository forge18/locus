//! ACP planning-client transport.

use std::path::PathBuf;

use agent_client_protocol::{
    schema::v1::{
        ContentBlock, NewSessionRequest, PromptRequest, SessionId, SessionNotification, TextContent,
    },
    AcpAgent, AcpAgentConfig,
};
use tokio::sync::broadcast;

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

/// Creates the ACP SDK transport through `docker exec`, so the agent process runs in its container.
pub fn container_stdio_transport<I, S>(
    container: impl Into<String>,
    command: impl Into<String>,
    args: I,
) -> PlanningAgent
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut docker_args = vec!["exec".into(), "-i".into(), container.into(), command.into()];
    docker_args.extend(args.into_iter().map(Into::into));
    planning_stdio_transport("docker", docker_args)
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
pub struct UpdateStream {
    sender: broadcast::Sender<SessionNotification>,
}

impl UpdateStream {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SessionNotification> {
        self.sender.subscribe()
    }

    pub fn publish(&self, update: SessionNotification) {
        let _ = self.sender.send(update);
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

    use crate::telemetry::{AcpAdapter, Adapter, EventVerb};

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

    use crate::telemetry::{AcpAdapter, Adapter, EventVerb};

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
    use serde_json::json;

    use crate::telemetry::{AcpAdapter, Adapter, EventCollector, EventVerb, PermissionAlarm};

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

        let event = collector.capture("planning-run", captured.into_iter().next().expect("event"));

        assert_eq!(event.verb, EventVerb::PermissionRequest);
        assert_eq!(event.raw, raw);
        assert_eq!(
            alarms.try_recv().expect("permission alarm"),
            PermissionAlarm {
                run_id: "planning-run".into(),
                seq: 0,
            }
        );
    }
}

#[cfg(test)]
mod indistinguishable {
    use serde_json::json;

    use crate::telemetry::{AcpAdapter, Adapter, EventCollector, EventVerb, HooksAdapter};

    #[test]
    fn acp_and_hook_events_reach_downstream_with_the_same_shape() {
        let acp = AcpAdapter
            .normalize(json!({
                "method": "session/update",
                "params": {
                    "update": {
                        "sessionUpdate": "agent_message_chunk",
                        "content": {"type": "text", "text": "same response"},
                    },
                },
            }))
            .expect("normalize ACP update")
            .pop()
            .expect("ACP emits an assistant event");
        let hooks_adapter = HooksAdapter::with_table(
            "claude",
            [
                ("AssistantMessage", EventVerb::Assistant),
                ("PreToolUse", EventVerb::ToolCall),
            ],
        );
        let hooks = hooks_adapter
            .normalize(json!({
                "harness": "claude",
                "hook": "AssistantMessage",
                "raw": {"text": "same response"},
            }))
            .expect("normalize hook event")
            .pop()
            .expect("hook emits an assistant event");
        let acp_tool = AcpAdapter
            .normalize(json!({
                "method": "session/update",
                "params": {
                    "update": {
                        "sessionUpdate": "tool_call",
                        "title": "read_file",
                        "rawInput": {"path": "PLAN.md"},
                    },
                },
            }))
            .expect("normalize ACP tool call")
            .pop()
            .expect("ACP emits a tool event");
        let hook_tool = hooks_adapter
            .normalize(json!({
                "harness": "claude",
                "hook": "PreToolUse",
                "raw": {
                    "tool_name": "read_file",
                    "tool_input": {"path": "PLAN.md"},
                },
            }))
            .expect("normalize hook tool call")
            .pop()
            .expect("hook emits a tool event");

        let acp_collector = EventCollector::new(2);
        let hook_collector = EventCollector::new(2);
        let acp_events = [
            acp_collector.capture("run-1", acp),
            acp_collector.capture("run-1", acp_tool),
        ];
        let hook_events = [
            hook_collector.capture("run-1", hooks),
            hook_collector.capture("run-1", hook_tool),
        ];

        for (acp_event, hook_event) in acp_events.into_iter().zip(hook_events) {
            assert_eq!(acp_event.run_id, hook_event.run_id);
            assert_eq!(acp_event.seq, hook_event.seq);
            assert_eq!(acp_event.verb, hook_event.verb);
            assert_eq!(acp_event.text, hook_event.text);
            assert_eq!(acp_event.tool, hook_event.tool);
            assert_eq!(acp_event.args, hook_event.args);
            assert_eq!(acp_event.usage, hook_event.usage);
            assert_ne!(
                acp_event.raw, hook_event.raw,
                "raw retains each source record"
            );
        }
    }
}

#[cfg(test)]
mod mapping_is_shared {
    use serde_json::json;

    use crate::telemetry::{AcpAdapter, Adapter, EventVerb};

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
