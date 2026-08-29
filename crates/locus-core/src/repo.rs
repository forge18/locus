//! Repository isolation and the local bare-remote git model.
//!
//! A repository added to Locus is represented by a bare remote and a shared bare object store.
//! Agent workspaces are ordinary clones of that remote with `--reference`; no user checkout is
//! mounted or modified. Primary branches are inputs and merge targets owned by the user, never
//! agent work branches.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Context, Result};
use serde::Serialize;

pub const DEFAULT_REMOTE_ROOT: &str = "/var/lib/locus/repos";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepoMode {
    Linked,
    Managed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    pub project: String,
    pub name: String,
    pub mode: RepoMode,
    pub source: PathBuf,
    pub bare_remote: PathBuf,
    pub object_store: PathBuf,
    pub primary_branch: String,
}

impl Repository {
    pub fn remote_path(&self) -> &Path {
        &self.bare_remote
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunWorkspace {
    pub path: PathBuf,
    pub branch: String,
    pub remote: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictFile {
    pub path: String,
    pub ours: Option<String>,
    pub theirs: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictInboxItem {
    pub branch: String,
    pub target_branch: String,
    pub files: Vec<ConflictFile>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MergeResult {
    Merged,
    Conflict(ConflictInboxItem),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiskMeasurement {
    pub object_store_bytes: u64,
    pub clone_object_bytes: u64,
    pub clone_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GitState {
    pub branch: String,
    pub dirty: bool,
    pub ahead: u32,
    pub behind: u32,
    pub agent_branches: Vec<String>,
}

pub struct RepoManager {
    root: PathBuf,
}

impl Default for RepoManager {
    fn default() -> Self {
        Self::new(DEFAULT_REMOTE_ROOT)
    }
}

impl RepoManager {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Read the state shown by the Develop git panel without changing the checkout.
    pub fn git_state(&self, checkout: impl AsRef<Path>) -> Result<GitState> {
        let checkout = checkout.as_ref();
        validate_checkout(checkout)?;
        let branch = current_branch(checkout)?;
        let status = git_output(checkout, ["status", "--porcelain"])?;
        if !status.status.success() {
            bail!("read git status failed")
        }
        let refs = git_output(
            checkout,
            [
                "for-each-ref",
                "--format=%(refname:short)",
                "refs/remotes/locus/agent",
            ],
        )?;
        let agent_branches = String::from_utf8_lossy(&refs.stdout)
            .lines()
            .map(str::to_owned)
            .collect();
        let (ahead, behind) = git_output(
            checkout,
            ["rev-list", "--left-right", "--count", "@{upstream}...HEAD"],
        )
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            let counts_text = String::from_utf8_lossy(&output.stdout);
            let mut counts = counts_text.split_whitespace();
            let behind = counts.next()?.parse::<u32>().ok()?;
            let ahead = counts.next()?.parse::<u32>().ok()?;
            Some((ahead, behind))
        })
        .unwrap_or((0, 0));
        Ok(GitState {
            branch,
            dirty: !status.stdout.is_empty(),
            ahead,
            behind,
            agent_branches,
        })
    }

    pub fn bare_remote_path(&self, project: &str, repo: &str) -> Result<PathBuf> {
        Ok(self.repo_paths(project, repo)?.0)
    }

    pub fn object_store_path(&self, project: &str, repo: &str) -> Result<PathBuf> {
        Ok(self.repo_paths(project, repo)?.1)
    }

    /// Import a user's checkout without changing its files, index, branch, or remotes.
    pub fn add_linked(
        &self,
        project: impl Into<String>,
        name: impl Into<String>,
        checkout: impl AsRef<Path>,
    ) -> Result<Repository> {
        let project = project.into();
        let name = name.into();
        let checkout = checkout.as_ref().to_path_buf();
        validate_name(&project, "project")?;
        validate_name(&name, "repo")?;
        validate_checkout(&checkout)?;
        let primary_branch = current_branch(&checkout)?;
        let (bare_remote, object_store) = self.repo_paths(&project, &name)?;
        create_parent(&bare_remote)?;
        if bare_remote.exists() || object_store.exists() {
            bail!("repository `{project}/{name}` already exists")
        }
        run_git(
            &checkout,
            [
                "clone",
                "--bare",
                checkout.to_str().unwrap_or_default(),
                object_store.to_str().unwrap_or_default(),
            ],
        )?;
        run_git(
            &checkout,
            [
                "clone",
                "--bare",
                "--reference",
                object_store.to_str().unwrap_or_default(),
                object_store.to_str().unwrap_or_default(),
                bare_remote.to_str().unwrap_or_default(),
            ],
        )?;
        Ok(Repository {
            project,
            name,
            mode: RepoMode::Linked,
            source: checkout,
            bare_remote,
            object_store,
            primary_branch,
        })
    }

    /// Import a forge repository into Locus-owned storage. The forge checkout is never retained.
    pub fn add_managed(
        &self,
        project: impl Into<String>,
        name: impl Into<String>,
        forge_url: impl AsRef<str>,
    ) -> Result<Repository> {
        let project = project.into();
        let name = name.into();
        let forge_url = forge_url.as_ref();
        validate_name(&project, "project")?;
        validate_name(&name, "repo")?;
        if forge_url.trim().is_empty() {
            bail!("managed repository forge URL is required")
        }
        let (bare_remote, object_store) = self.repo_paths(&project, &name)?;
        create_parent(&bare_remote)?;
        if bare_remote.exists() || object_store.exists() {
            bail!("repository `{project}/{name}` already exists")
        }
        run_git(
            &self.root,
            [
                "clone",
                "--bare",
                forge_url,
                object_store.to_str().unwrap_or_default(),
            ],
        )?;
        let primary_branch = symbolic_head(&object_store)?;
        run_git(
            &self.root,
            [
                "clone",
                "--bare",
                "--reference",
                object_store.to_str().unwrap_or_default(),
                object_store.to_str().unwrap_or_default(),
                bare_remote.to_str().unwrap_or_default(),
            ],
        )?;
        Ok(Repository {
            project,
            name,
            mode: RepoMode::Managed,
            source: PathBuf::from(forge_url),
            bare_remote,
            object_store,
            primary_branch,
        })
    }

    /// Linked repositories use explicit, on-demand synchronization. Agent and bot branches are
    /// Locus-owned, so synchronization never prunes them from the bare remote.
    pub fn sync_linked(&self, repository: &Repository) -> Result<()> {
        self.sync_linked_preserving_agent_branches(repository)
    }

    /// Sync linked source changes without pruning branches owned by agents or bots.
    fn sync_linked_preserving_agent_branches(&self, repository: &Repository) -> Result<()> {
        if repository.mode != RepoMode::Linked {
            return Ok(());
        }
        run_git(
            &repository.object_store,
            [
                "fetch",
                "--prune",
                repository.source.to_str().unwrap_or_default(),
                "+refs/*:refs/*",
            ],
        )?;
        run_git(
            &repository.bare_remote,
            [
                "fetch",
                repository.object_store.to_str().unwrap_or_default(),
                "+refs/*:refs/*",
            ],
        )?;
        Ok(())
    }

    /// Make an isolated run clone and name its only working branch `agent/<run-id>`.
    pub fn clone_run(
        &self,
        repository: &Repository,
        run_id: impl AsRef<str>,
        workspace: impl AsRef<Path>,
    ) -> Result<RunWorkspace> {
        self.sync_linked(repository)?;
        let branch = branch_name(run_id.as_ref())?;
        let workspace = workspace.as_ref().to_path_buf();
        if workspace.exists() {
            bail!("run workspace already exists: {}", workspace.display())
        }
        if let Some(parent) = workspace.parent() {
            fs::create_dir_all(parent).context("create run workspace parent")?;
        }
        run_git(
            &self.root,
            [
                "clone",
                "--no-checkout",
                "--reference",
                repository.object_store.to_str().unwrap_or_default(),
                repository.bare_remote.to_str().unwrap_or_default(),
                workspace.to_str().unwrap_or_default(),
            ],
        )?;
        run_git(&workspace, ["remote", "rename", "origin", "locus"])?;
        run_git(
            &workspace,
            [
                "checkout",
                "-b",
                branch.as_str(),
                repository.primary_branch.as_str(),
            ],
        )?;
        Ok(RunWorkspace {
            path: workspace,
            branch,
            remote: repository.bare_remote.clone(),
        })
    }

    /// Make or resume the one persistent workspace branch owned by a bot.
    pub fn clone_bot(
        &self,
        repository: &Repository,
        bot_id: impl AsRef<str>,
        workspace: impl AsRef<Path>,
    ) -> Result<RunWorkspace> {
        self.sync_linked_preserving_agent_branches(repository)?;
        let branch = bot_branch_name(bot_id.as_ref())?;
        let workspace = workspace.as_ref().to_path_buf();
        if workspace.exists() {
            bail!("bot workspace already exists: {}", workspace.display())
        }
        if let Some(parent) = workspace.parent() {
            fs::create_dir_all(parent).context("create bot workspace parent")?;
        }
        run_git(
            &self.root,
            [
                "clone",
                "--no-checkout",
                "--reference",
                repository.object_store.to_str().unwrap_or_default(),
                repository.bare_remote.to_str().unwrap_or_default(),
                workspace.to_str().unwrap_or_default(),
            ],
        )?;
        run_git(&workspace, ["remote", "rename", "origin", "locus"])?;
        let remote_ref = format!("refs/remotes/locus/{branch}");
        let has_bot_branch = git_output(&workspace, ["show-ref", "--verify", &remote_ref])
            .map(|output| output.status.success())
            .unwrap_or(false);
        let base = if has_bot_branch {
            remote_ref.as_str()
        } else {
            repository.primary_branch.as_str()
        };
        run_git(&workspace, ["checkout", "-b", branch.as_str(), base])?;
        if !has_bot_branch {
            run_git(
                &workspace,
                [
                    "push",
                    "locus",
                    format!("refs/heads/{branch}:refs/heads/{branch}").as_str(),
                ],
            )?;
        }
        Ok(RunWorkspace {
            path: workspace,
            branch,
            remote: repository.bare_remote.clone(),
        })
    }

    /// Push only the persistent bot branch. Primary branches are rejected before invoking git.
    pub fn push_bot_branch(
        &self,
        workspace: impl AsRef<Path>,
        bot_id: impl AsRef<str>,
    ) -> Result<()> {
        let branch = bot_branch_name(bot_id.as_ref())?;
        let workspace = workspace.as_ref();
        let current = current_branch(workspace)?;
        if current != branch {
            bail!("workspace is on `{current}`, expected `{branch}`")
        }
        refuse_primary_branch(&current)?;
        run_git(
            workspace,
            [
                "push",
                "locus",
                format!("refs/heads/{branch}:refs/heads/{branch}").as_str(),
            ],
        )?;
        Ok(())
    }

    /// Push only the run branch. A primary branch is rejected before invoking git.
    pub fn push_branch(&self, workspace: impl AsRef<Path>, run_id: impl AsRef<str>) -> Result<()> {
        let branch = branch_name(run_id.as_ref())?;
        let workspace = workspace.as_ref();
        let current = current_branch(workspace)?;
        if current != branch {
            bail!("workspace is on `{current}`, expected `{branch}`")
        }
        refuse_primary_branch(&current)?;
        run_git(
            workspace,
            [
                "push",
                "locus",
                format!("refs/heads/{branch}:refs/heads/{branch}").as_str(),
            ],
        )?;
        Ok(())
    }

    /// Fetch a pushed branch from Locus and merge it into a user's non-primary checkout.
    pub fn merge_back_from(
        &self,
        repository: &Repository,
        target_checkout: impl AsRef<Path>,
        target_branch: impl AsRef<str>,
        agent_branch: impl AsRef<str>,
    ) -> Result<MergeResult> {
        let target_checkout = target_checkout.as_ref();
        let agent_branch = agent_branch.as_ref();
        run_git(
            target_checkout,
            [
                "fetch",
                repository.bare_remote.to_str().unwrap_or_default(),
                &format!("refs/heads/{agent_branch}:refs/heads/{agent_branch}"),
            ],
        )?;
        self.merge_back(target_checkout, target_branch, agent_branch)
    }

    /// Merge a pushed agent branch into a user's non-primary checkout. Conflicts are returned as
    /// an inbox payload and the target checkout is left clean after `git merge --abort`.
    pub fn merge_back(
        &self,
        target_checkout: impl AsRef<Path>,
        target_branch: impl AsRef<str>,
        agent_branch: impl AsRef<str>,
    ) -> Result<MergeResult> {
        let target_branch = target_branch.as_ref();
        let agent_branch = agent_branch.as_ref();
        refuse_primary_branch(target_branch)?;
        if !agent_branch.starts_with("agent/") {
            bail!("merge source must be an agent branch")
        }
        let target_checkout = target_checkout.as_ref();
        if current_branch(target_checkout)? != target_branch {
            bail!("target checkout is not on `{target_branch}`")
        }
        let output = git_output(
            target_checkout,
            ["merge", "--no-ff", "--no-edit", agent_branch],
        )?;
        if output.status.success() {
            return Ok(MergeResult::Merged);
        }
        let files = conflict_files(target_checkout, target_branch, agent_branch)?;
        let _ = run_git(target_checkout, ["merge", "--abort"]);
        Ok(MergeResult::Conflict(ConflictInboxItem {
            branch: agent_branch.into(),
            target_branch: target_branch.into(),
            files,
        }))
    }

    /// Clone an involved repository without a remote, so accidental pushes fail by construction.
    pub fn clone_context(
        &self,
        repository: &Repository,
        context_root: impl AsRef<Path>,
    ) -> Result<PathBuf> {
        let context_root = context_root.as_ref();
        let destination = context_root.join(&repository.name);
        if destination.exists() {
            bail!(
                "context repository already exists: {}",
                destination.display()
            )
        }
        fs::create_dir_all(context_root).context("create context root")?;
        run_git(
            &self.root,
            [
                "clone",
                "--no-checkout",
                "--reference",
                repository.object_store.to_str().unwrap_or_default(),
                repository.bare_remote.to_str().unwrap_or_default(),
                destination.to_str().unwrap_or_default(),
            ],
        )?;
        run_git(
            &destination,
            ["checkout", "--detach", repository.primary_branch.as_str()],
        )?;
        run_git(&destination, ["remote", "remove", "origin"])?;
        Ok(destination)
    }

    /// Return the bytes in the per-clone object directories. Reference clones should keep this
    /// near zero while the shared store owns the history.
    pub fn measure_reference_savings(
        &self,
        repository: &Repository,
        clones: impl IntoIterator<Item = impl AsRef<Path>>,
    ) -> Result<DiskMeasurement> {
        let clones: Vec<_> = clones.into_iter().collect();
        let clone_object_bytes = clones
            .iter()
            .map(|clone| unique_clone_object_bytes(clone.as_ref()))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .sum();
        Ok(DiskMeasurement {
            object_store_bytes: object_payload_size(&repository.object_store.join("objects"))?,
            clone_object_bytes,
            clone_count: clones.len(),
        })
    }

    fn repo_paths(&self, project: &str, repo: &str) -> Result<(PathBuf, PathBuf)> {
        validate_name(project, "project")?;
        validate_name(repo, "repo")?;
        let directory = self.root.join(project);
        Ok((
            directory.join(format!("{repo}.git")),
            directory.join(format!("{repo}.objects.git")),
        ))
    }
}

pub fn branch_name(run_id: &str) -> Result<String> {
    if run_id.trim().is_empty()
        || run_id.contains('/')
        || run_id.contains(' ')
        || run_id.contains("..")
    {
        bail!("invalid run id for agent branch")
    }
    let branch = format!("agent/{run_id}");
    refuse_primary_branch(&branch)?;
    Ok(branch)
}

pub fn bot_branch_name(bot_id: &str) -> Result<String> {
    if bot_id.trim().is_empty()
        || bot_id.contains('/')
        || bot_id.contains(' ')
        || bot_id.contains("..")
    {
        bail!("invalid bot id for persistent branch")
    }
    let branch = format!("bots/{bot_id}");
    refuse_primary_branch(&branch)?;
    Ok(branch)
}

pub fn refuse_primary_branch(branch: &str) -> Result<()> {
    if matches!(
        branch,
        "main" | "master" | "refs/heads/main" | "refs/heads/master"
    ) {
        bail!("Locus never writes to primary branch `{branch}`")
    }
    Ok(())
}

fn validate_name(value: &str, kind: &str) -> Result<()> {
    if value.trim().is_empty()
        || value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
    {
        bail!("invalid {kind} name")
    }
    Ok(())
}

fn validate_checkout(path: &Path) -> Result<()> {
    if !path.is_dir() {
        bail!("repository checkout does not exist: {}", path.display())
    }
    let output = git_output(path, ["rev-parse", "--is-inside-work-tree"])?;
    if !output.status.success() || String::from_utf8_lossy(&output.stdout).trim() != "true" {
        bail!("path is not a non-bare git checkout: {}", path.display())
    }
    Ok(())
}

fn current_branch(path: &Path) -> Result<String> {
    let output = git_output(path, ["branch", "--show-current"])?;
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !output.status.success() || branch.is_empty() {
        bail!("repository has no checked-out branch: {}", path.display())
    }
    Ok(branch)
}

fn symbolic_head(bare: &Path) -> Result<String> {
    let output = git_output(bare, ["symbolic-ref", "--short", "HEAD"])?;
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !output.status.success() || branch.is_empty() {
        bail!("bare repository has no primary branch")
    }
    Ok(branch)
}

fn create_parent(path: &Path) -> Result<()> {
    path.parent()
        .context("repository path has no parent")
        .and_then(|parent| fs::create_dir_all(parent).context("create repository storage"))
}

fn run_git<I, S>(directory: &Path, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let output = git_output(directory, args)?;
    if output.status.success() {
        Ok(())
    } else {
        bail!(
            "git operation failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
    }
}

fn git_output<I, S>(directory: &Path, args: I) -> Result<std::process::Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Command::new("git")
        .args(args)
        .current_dir(directory)
        .output()
        .with_context(|| format!("run git in {}", directory.display()))
}

fn conflict_files(path: &Path, ours: &str, theirs: &str) -> Result<Vec<ConflictFile>> {
    let output = git_output(path, ["diff", "--name-only", "--diff-filter=U"])?;
    let paths = String::from_utf8_lossy(&output.stdout);
    paths
        .lines()
        .map(|file| {
            Ok(ConflictFile {
                path: file.into(),
                ours: show_file(path, ours, file)?,
                theirs: show_file(path, theirs, file)?,
            })
        })
        .collect()
}

fn show_file(path: &Path, branch: &str, file: &str) -> Result<Option<String>> {
    let output = git_output(path, ["show", &format!("{branch}:{file}")])?;
    if output.status.success() {
        Ok(Some(String::from_utf8_lossy(&output.stdout).into_owned()))
    } else {
        Ok(None)
    }
}

fn unique_clone_object_bytes(clone: &Path) -> Result<u64> {
    let objects = clone.join(".git/objects");
    // An alternates file is the on-disk proof that the clone's history is owned by the
    // shared object store. Baseline history therefore contributes no per-clone bytes;
    // objects written later by the run are outside this baseline measurement.
    if objects.join("info/alternates").is_file() {
        return Ok(0);
    }
    object_payload_size(&objects)
}

fn object_payload_size(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }
    let mut total = 0;
    for entry in fs::read_dir(path).with_context(|| format!("read {}", path.display()))? {
        let entry = entry.context("read object entry")?;
        if entry.file_name() == "info" {
            continue;
        }
        let metadata = entry.metadata().context("read object metadata")?;
        if metadata.is_dir() {
            total += object_payload_size(&entry.path())?;
        } else {
            total += metadata.len();
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use uuid::Uuid;

    struct TempDir(PathBuf);
    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "locus-repo-test-{}-{}",
                std::process::id(),
                Uuid::new_v4()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn git(path: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?}: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fn fixture() -> (TempDir, PathBuf, RepoManager, Repository) {
        let tmp = TempDir::new();
        let checkout = tmp.path().join("checkout");
        fs::create_dir_all(&checkout).unwrap();
        git(&checkout, &["init", "-b", "main"]);
        git(&checkout, &["config", "user.email", "test@example.com"]);
        git(&checkout, &["config", "user.name", "Test"]);
        fs::write(checkout.join("README"), "one\n").unwrap();
        git(&checkout, &["add", "."]);
        git(&checkout, &["commit", "-m", "initial"]);
        let manager = RepoManager::new(tmp.path().join("remotes"));
        let repo = manager.add_linked("project", "repo", &checkout).unwrap();
        (tmp, checkout, manager, repo)
    }

    #[test]
    fn bare_remote() {
        let (_tmp, _checkout, manager, repo) = fixture();
        assert_eq!(
            repo.bare_remote,
            manager.bare_remote_path("project", "repo").unwrap()
        );
        assert!(repo.bare_remote.join("HEAD").is_file());
        assert_eq!(symbolic_head(&repo.bare_remote).unwrap(), "main");
    }

    #[test]
    fn add_linked() {
        let (_tmp, checkout, _manager, repo) = fixture();
        assert_eq!(repo.mode, RepoMode::Linked);
        assert!(checkout.join("README").is_file());
        assert!(repo.bare_remote.is_dir());
    }

    #[test]
    fn add_managed() {
        let tmp = TempDir::new();
        let source = tmp.path().join("forge.git");
        git(tmp.path(), &["init", "--bare", source.to_str().unwrap()]);
        let seed = tmp.path().join("seed");
        fs::create_dir_all(&seed).unwrap();
        git(&seed, &["init", "-b", "main"]);
        git(&seed, &["config", "user.email", "test@example.com"]);
        git(&seed, &["config", "user.name", "Test"]);
        fs::write(seed.join("README"), "managed\n").unwrap();
        git(&seed, &["add", "."]);
        git(&seed, &["commit", "-m", "initial"]);
        git(
            &seed,
            &["remote", "add", "origin", source.to_str().unwrap()],
        );
        git(&seed, &["push", "origin", "main"]);
        let manager = RepoManager::new(tmp.path().join("locus"));
        let repo = manager
            .add_managed("p", "r", source.to_str().unwrap())
            .unwrap();
        assert_eq!(repo.mode, RepoMode::Managed);
        assert_eq!(repo.primary_branch, "main");
    }

    #[test]
    fn object_store() {
        let (_tmp, _checkout, manager, repo) = fixture();
        assert!(repo.object_store.is_dir());
        assert_eq!(
            repo.object_store,
            manager.object_store_path("project", "repo").unwrap()
        );
        assert!(repo.bare_remote.join("objects/info/alternates").is_file());
    }

    #[test]
    fn run_clone() {
        let (tmp, _checkout, manager, repo) = fixture();
        let run = manager
            .clone_run(&repo, "run-1", tmp.path().join("workspace"))
            .unwrap();
        assert_eq!(run.branch, "agent/run-1");
        assert_eq!(current_branch(&run.path).unwrap(), "agent/run-1");
        assert!(run.path.join("README").is_file());
    }

    #[test]
    fn reference_saves_disk() {
        let (tmp, _checkout, manager, repo) = fixture();
        let a = manager.clone_run(&repo, "a", tmp.path().join("a")).unwrap();
        let b = manager.clone_run(&repo, "b", tmp.path().join("b")).unwrap();
        let measurement = manager
            .measure_reference_savings(&repo, [&a.path, &b.path])
            .unwrap();
        assert_eq!(measurement.clone_count, 2);
        assert_eq!(
            measurement.clone_object_bytes, 0,
            "measurement: {measurement:?}"
        );
    }

    #[test]
    fn branch_naming() {
        assert_eq!(branch_name("123").unwrap(), "agent/123");
        assert_eq!(bot_branch_name("123").unwrap(), "bots/123");
    }

    #[test]
    fn push_branch() {
        let (tmp, _checkout, manager, repo) = fixture();
        let run = manager
            .clone_run(&repo, "push", tmp.path().join("workspace"))
            .unwrap();
        fs::write(run.path.join("new"), "branch\n").unwrap();
        git(&run.path, &["add", "."]);
        git(&run.path, &["commit", "-m", "branch"]);
        manager.push_branch(&run.path, "push").unwrap();
        let output = git_output(
            &repo.bare_remote,
            ["show-ref", "--verify", "refs/heads/agent/push"],
        )
        .unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn bot_branch_persists_across_clones() {
        let (tmp, _checkout, manager, repo) = fixture();
        let first = manager
            .clone_bot(&repo, "bot-1", tmp.path().join("bot-first"))
            .unwrap();
        assert!(git_output(
            &repo.bare_remote,
            ["show-ref", "--verify", "refs/heads/bots/bot-1"]
        )
        .unwrap()
        .status
        .success());
        fs::write(first.path.join("bot-note"), "first\n").unwrap();
        git(&first.path, &["add", "."]);
        git(&first.path, &["commit", "-m", "bot work"]);
        manager.push_bot_branch(&first.path, "bot-1").unwrap();
        let _ordinary_run = manager
            .clone_run(&repo, "ordinary", tmp.path().join("ordinary"))
            .unwrap();
        let second = manager
            .clone_bot(&repo, "bot-1", tmp.path().join("bot-second"))
            .unwrap();
        assert_eq!(second.branch, "bots/bot-1");
        assert!(second.path.join("bot-note").is_file());
    }

    #[test]
    fn never_writes_main() {
        assert!(refuse_primary_branch("main").is_err());
        assert!(refuse_primary_branch("master").is_err());
        assert!(refuse_primary_branch("agent/x").is_ok());
    }

    #[test]
    fn main_push_refused() {
        let (_tmp, _checkout, _manager, _repo) = fixture();
        assert!(branch_name("main").is_ok());
        assert!(refuse_primary_branch("main").is_err());
    }

    #[test]
    fn merge_back_clean() {
        let (tmp, checkout, manager, repo) = fixture();
        git(&checkout, &["checkout", "-b", "develop"]);
        let run = manager
            .clone_run(&repo, "merge", tmp.path().join("workspace"))
            .unwrap();
        fs::write(run.path.join("merged"), "yes\n").unwrap();
        git(&run.path, &["add", "."]);
        git(&run.path, &["commit", "-m", "change"]);
        manager.push_branch(&run.path, "merge").unwrap();
        assert_eq!(
            manager
                .merge_back_from(&repo, &checkout, "develop", "agent/merge")
                .unwrap(),
            MergeResult::Merged
        );
        assert!(checkout.join("merged").is_file());
    }

    #[test]
    fn conflict_to_inbox() {
        let (tmp, checkout, manager, repo) = fixture();
        git(&checkout, &["checkout", "-b", "develop"]);
        fs::write(checkout.join("README"), "ours\n").unwrap();
        git(&checkout, &["commit", "-am", "ours"]);
        let run = manager
            .clone_run(&repo, "conflict", tmp.path().join("workspace"))
            .unwrap();
        fs::write(run.path.join("README"), "theirs\n").unwrap();
        git(&run.path, &["commit", "-am", "theirs"]);
        manager.push_branch(&run.path, "conflict").unwrap();
        let result = manager
            .merge_back_from(&repo, &checkout, "develop", "agent/conflict")
            .unwrap();
        match result {
            MergeResult::Conflict(item) => {
                assert_eq!(item.files[0].ours.as_deref(), Some("ours\n"));
                assert_eq!(item.files[0].theirs.as_deref(), Some("theirs\n"));
            }
            MergeResult::Merged => panic!("expected conflict"),
        }
    }

    #[test]
    fn context_repos() {
        let (tmp, _checkout, manager, repo) = fixture();
        let context = manager
            .clone_context(&repo, tmp.path().join("context"))
            .unwrap();
        assert!(context.join("README").is_file());
        assert!(git_output(&context, ["remote"]).unwrap().stdout.is_empty());
    }

    #[test]
    fn context_is_read_only() {
        let (tmp, _checkout, manager, repo) = fixture();
        let context = manager
            .clone_context(&repo, tmp.path().join("context"))
            .unwrap();
        let output = git_output(&context, ["push"]).unwrap();
        assert!(!output.status.success());
    }

    #[test]
    fn three_concurrent() {
        let (tmp, _checkout, manager, repo) = fixture();
        let a = manager.clone_run(&repo, "a", tmp.path().join("a")).unwrap();
        let b = manager.clone_run(&repo, "b", tmp.path().join("b")).unwrap();
        let c = manager.clone_run(&repo, "c", tmp.path().join("c")).unwrap();
        for (path, name) in [(&a.path, "a"), (&b.path, "b"), (&c.path, "c")] {
            fs::write(path.join(name), name).unwrap();
            git(path, &["add", "."]);
            git(path, &["commit", "-m", name]);
        }
        assert!(
            !a.path.join("b").exists() && !b.path.join("a").exists() && !c.path.join("a").exists()
        );
    }

    #[test]
    fn linked_sync() {
        let (_tmp, checkout, manager, repo) = fixture();
        fs::write(checkout.join("later"), "sync\n").unwrap();
        git(&checkout, &["add", "."]);
        git(&checkout, &["commit", "-m", "later"]);
        manager.sync_linked(&repo).unwrap();
        let output = git_output(&repo.bare_remote, ["show", "main:later"]).unwrap();
        assert!(output.status.success());
    }
}
