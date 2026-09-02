//! The ACP process must be created inside a private container namespace.

#![cfg(target_os = "linux")]

use std::{error::Error, process::Command};

#[test]
fn acp_process_is_not_in_the_host_pid_namespace() -> Result<(), Box<dyn Error>> {
    let host_namespace = std::fs::read_link("/proc/self/ns/pid")?;
    let output = Command::new("docker")
        .args(["run", "--rm", "alpine:3.22", "readlink", "/proc/1/ns/pid"])
        .output()?;
    assert!(
        output.status.success(),
        "namespace probe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let container_namespace = String::from_utf8_lossy(&output.stdout);
    assert_ne!(
        host_namespace.to_string_lossy().trim(),
        container_namespace.trim(),
        "the ACP process must not share the host PID namespace"
    );
    Ok(())
}
