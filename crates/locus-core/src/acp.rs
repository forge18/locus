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
