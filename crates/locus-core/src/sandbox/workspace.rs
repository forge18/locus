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

/// Clone a remote and check out the exact run branch. The branch is validated before any
/// container request is made so a verifier cannot accidentally run on a primary branch.
pub fn workspace_clone_branch_command(remote: &str, branch: &str) -> Result<String> {
    if remote.trim().is_empty() {
        bail!("workspace clone remote is required")
    }
    refuse_primary_branch(branch)?;
    if branch.trim().is_empty() {
        bail!("workspace branch is required")
    }
    Ok(format!(
        "git clone {} /workspace && git -C /workspace checkout {}",
        shell_quote(remote),
        shell_quote(branch),
    ))
}

pub fn bot_workspace_clone_command(remote: &str, bot_id: &str) -> Result<String> {
    let branch = crate::repo::bot_branch_name(bot_id)?;
    workspace_clone_branch_command(remote, &branch)
}

pub fn refuse_primary_branch(branch: &str) -> Result<()> {
    if matches!(
        branch,
        "main" | "master" | "refs/heads/main" | "refs/heads/master"
    ) {
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

    #[test]
    fn bot_workspace_uses_the_persistent_branch() {
        let command = bot_workspace_clone_command("git://host/project.git", "bot-1").unwrap();
        assert!(command.contains("checkout 'bots/bot-1'"));
        assert!(!command.contains("agent/"));
    }
}
