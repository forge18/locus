//! What a container may see: two mounts, and the PTY attachment.

use super::*;
#[cfg(test)]
use crate::sandbox::workspace::refuse_primary_branch;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mount {
    pub source: String,
    pub destination: String,
    pub read_only: bool,
}

/// The source config tree is immutable. The entrypoint copies it to writable config for harnesses
/// such as Claude that persist transcripts in their config home.
pub fn agent_mounts(
    socket_source: impl Into<String>,
    config_source: impl Into<String>,
) -> [Mount; 2] {
    [
        Mount {
            source: socket_source.into(),
            destination: LOCUS_SOCKET.into(),
            read_only: false,
        },
        Mount {
            source: config_source.into(),
            destination: CONFIG_SOURCE.into(),
            read_only: true,
        },
    ]
}

pub fn entrypoint_setup() -> &'static str {
    "mkdir -p /locus/config && cp -a /locus/config-ro/. /locus/config/"
}

pub fn validate_agent_mounts(mounts: &[Mount]) -> Result<()> {
    if mounts.len() != 2 {
        bail!("agent containers may have exactly two mounts")
    }
    let destinations = mounts
        .iter()
        .map(|mount| mount.destination.as_str())
        .collect::<BTreeSet<_>>();
    if destinations != BTreeSet::from([LOCUS_SOCKET, CONFIG_SOURCE]) {
        bail!("agent mounts must be the locus socket and read-only config source")
    }
    if mounts
        .iter()
        .any(|mount| mount.destination.contains("docker.sock"))
    {
        bail!("agent containers may not receive a Docker socket")
    }
    if mounts
        .iter()
        .find(|mount| mount.destination == CONFIG_SOURCE)
        .is_none_or(|mount| !mount.read_only)
    {
        bail!("materialized config source must be read-only")
    }
    Ok(())
}

/// The Docker exec attachment required to stream one terminal session through a host PTY.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PtyAttachment {
    pub tty: bool,
    pub stdout: bool,
    pub stderr: bool,
}

pub const AGENT_PTY: PtyAttachment = PtyAttachment {
    tty: true,
    stdout: true,
    stderr: true,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_mounts_only() {
        let mounts = agent_mounts("/tmp/socket", "/tmp/config");
        validate_agent_mounts(&mounts).unwrap();
        assert_eq!(mounts[1].destination, CONFIG_SOURCE);
        assert!(mounts[1].read_only);
        assert!(entrypoint_setup().contains(CONFIG_DESTINATION));
    }

    #[test]
    fn no_docker_socket() {
        let mut mounts = agent_mounts("/tmp/socket", "/tmp/config").to_vec();
        mounts.push(Mount {
            source: "/var/run/docker.sock".into(),
            destination: "/var/run/docker.sock".into(),
            read_only: false,
        });
        assert!(validate_agent_mounts(&mounts).is_err());
    }

    #[test]
    fn host_tree_unreachable() {
        let mounts = agent_mounts("/tmp/socket", "/tmp/config");
        assert!(mounts.iter().all(|mount| mount.destination != "/workspace"));
        assert!(refuse_primary_branch("main").is_err());
    }

    #[test]
    fn pty_attaches() {
        assert_eq!(
            AGENT_PTY,
            PtyAttachment {
                tty: true,
                stdout: true,
                stderr: true,
            }
        );
    }
}
