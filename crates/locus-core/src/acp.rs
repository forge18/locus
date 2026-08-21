//! ACP planning-client transport.

use std::path::PathBuf;

use agent_client_protocol::{schema::v1::NewSessionRequest, AcpAgent, AcpAgentConfig};

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
