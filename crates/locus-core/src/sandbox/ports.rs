//! Per-run port allocation and per-project network naming.

use super::*;

#[derive(Default)]
pub struct PortAllocator {
    /// Binding keeps the host port reserved until the run releases it.
    allocated: Mutex<BTreeMap<u16, TcpListener>>,
}

impl PortAllocator {
    pub fn allocate(&self) -> Result<u16> {
        let mut allocated = self.allocated.lock().expect("port allocator lock");
        for port in PORT_START..=PORT_END {
            if allocated.contains_key(&port) {
                continue;
            }
            if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) {
                allocated.insert(port, listener);
                return Ok(port);
            }
        }
        bail!("no Locus ports remain")
    }

    pub fn release(&self, port: u16) {
        self.allocated
            .lock()
            .expect("port allocator lock")
            .remove(&port);
    }
}

/// The agent-only network. Docker marks it `internal`, so it has no NAT route to the Internet.
pub fn project_internal_network(project_id: &str) -> String {
    format!("locus-{project_id}-internal")
}

/// The sidecar-only outward network. Agent containers are never attached to this network.
pub fn project_egress_network(project_id: &str) -> String {
    format!("locus-{project_id}-egress")
}

/// Compatibility name for the project network used by agents.
pub fn project_network(project_id: &str) -> String {
    project_internal_network(project_id)
}

pub fn same_project_network(left: &str, right: &str) -> bool {
    project_internal_network(left) == project_internal_network(right)
}

#[cfg(test)]
mod allocation {
    use super::*;

    #[test]
    fn allocates_unique() {
        let ports = PortAllocator::default();
        let first = ports.allocate().unwrap();
        let second = ports.allocate().unwrap();
        assert_ne!(first, second);
        assert!((PORT_START..=PORT_END).contains(&first));
        assert!(TcpListener::bind(("127.0.0.1", first)).is_err());
        ports.release(first);
        assert!(TcpListener::bind(("127.0.0.1", first)).is_ok());
    }
}

#[cfg(test)]
mod net {
    use super::*;

    #[test]
    fn project_network() {
        assert_eq!(
            super::project_network("project-a"),
            "locus-project-a-internal"
        );
        assert_eq!(
            super::project_egress_network("project-a"),
            "locus-project-a-egress"
        );
    }

    #[test]
    fn project_isolation() {
        assert!(!same_project_network("project-a", "project-b"));
    }

    #[test]
    fn project_isolation_keeps_none_runs_off_the_egress_network() {
        let agent_network = project_internal_network("project-a");
        let sidecar_egress = project_egress_network("project-a");
        assert_ne!(agent_network, sidecar_egress);
    }
}
