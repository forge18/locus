//! The workspace is a clone, never a mount, and it never touches a primary branch.

use super::*;
use crate::sandbox::image::shell_quote;

pub fn workspace_clone_command(remote: &str, run_id: &str) -> Result<String> {
    if remote.trim().is_empty() {
        bail!("workspace clone remote is required")
    }
    Ok(format!(
        "git clone {} /workspace && git -C /workspace checkout -b agent/{}",
        shell_quote(remote),
        shell_quote(run_id),
    ))
}

pub fn refuse_primary_branch(branch: &str) -> Result<()> {
    if matches!(branch, "main" | "master") {
        bail!("agent containers may not run on `{branch}`")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_is_a_clone() {
        let command = workspace_clone_command("git://host/project.git", "run-1").unwrap();
        assert!(command.contains("git clone") && command.contains("agent/'run-1'"));
        refuse_primary_branch("agent/run-1").unwrap();
    }
}
