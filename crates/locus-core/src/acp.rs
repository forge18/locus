//! ACP planning-client transport.

use std::path::PathBuf;

use agent_client_protocol::{AcpAgent, AcpAgentConfig};

/// Creates the ACP SDK's subprocess transport, which communicates with the agent over stdio.
pub fn stdio_transport<I, S>(command: impl Into<PathBuf>, args: I) -> AcpAgent
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    AcpAgent::new(AcpAgentConfig::new(command).args(args))
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
