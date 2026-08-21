//! ACP planning-client transport.

use std::path::PathBuf;

use agent_client_protocol::{
    schema::v1::{
        ContentBlock, NewSessionRequest, PromptRequest, SessionId, SessionNotification, TextContent,
    },
    AcpAgent, AcpAgentConfig,
};
use tokio::sync::broadcast;

/// Creates the ACP SDK's subprocess transport, which communicates with the agent over stdio.
pub fn stdio_transport<I, S>(command: impl Into<PathBuf>, args: I) -> AcpAgent
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    AcpAgent::new(AcpAgentConfig::new(command).args(args))
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
        let transport = stdio_transport("agent", ["acp"]);

        assert_eq!(transport.config().command(), Path::new("agent"));
        assert_eq!(transport.config().arguments(), ["acp"]);
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
