//! Ordinary clone resolution for the editor.
//!
//! The editor never creates or opens a git worktree. Linked repositories use the user's
//! existing checkout; managed repositories use Locus's normal clone beside its bare remote.

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RepositoryKind {
    Linked { checkout: PathBuf },
    Managed { clone: PathBuf },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorRepository {
    pub name: String,
    pub kind: RepositoryKind,
}

impl EditorRepository {
    pub fn linked(name: impl Into<String>, checkout: impl Into<PathBuf>) -> Result<Self> {
        let checkout = checkout.into();
        validate_path(&checkout, "linked checkout")?;
        Ok(Self {
            name: name.into(),
            kind: RepositoryKind::Linked { checkout },
        })
    }

    pub fn managed(name: impl Into<String>, clone: impl Into<PathBuf>) -> Result<Self> {
        let clone = clone.into();
        validate_path(&clone, "managed clone")?;
        Ok(Self {
            name: name.into(),
            kind: RepositoryKind::Managed { clone },
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCheckout {
    pub path: PathBuf,
    pub managed: bool,
}

pub fn open_checkout(repository: &EditorRepository) -> Result<EditorCheckout> {
    let (path, managed) = match &repository.kind {
        RepositoryKind::Linked { checkout } => (checkout.clone(), false),
        RepositoryKind::Managed { clone } => (clone.clone(), true),
    };
    if path
        .components()
        .any(|component| component.as_os_str() == "worktrees")
    {
        bail!("editor checkouts may not use a worktree path")
    }
    Ok(EditorCheckout { path, managed })
}

/// The host command used to provision a managed checkout. It is intentionally a clone.
pub fn managed_clone_command(remote: &str, destination: &Path) -> Result<String> {
    if remote.trim().is_empty() {
        bail!("managed repository remote is required")
    }
    if destination.as_os_str().is_empty() {
        bail!("managed repository destination is required")
    }
    let destination = destination.to_string_lossy().replace('\'', "'\\''");
    let remote = remote.replace('\'', "'\\''");
    Ok(format!("git clone '{}' '{}'", remote, destination))
}

fn validate_path(path: &Path, label: &str) -> Result<()> {
    if path.as_os_str().is_empty() {
        bail!("{label} path is required")
    }
    Ok(())
}

#[cfg(test)]
#[test]
fn opens_linked_checkout() {
    let repo = EditorRepository::linked("linked", "/Users/me/Repos/linked").unwrap();
    let checkout = open_checkout(&repo).unwrap();
    assert_eq!(checkout.path, PathBuf::from("/Users/me/Repos/linked"));
    assert!(!checkout.managed);
}

#[cfg(test)]
#[test]
fn opens_managed_clone() {
    let repo = EditorRepository::managed("managed", "/var/lib/locus/repos/managed").unwrap();
    let checkout = open_checkout(&repo).unwrap();
    assert_eq!(checkout.path, PathBuf::from("/var/lib/locus/repos/managed"));
    assert!(checkout.managed);
    assert!(managed_clone_command("file:///repo.git", &checkout.path)
        .unwrap()
        .starts_with("git clone "));
}

#[cfg(test)]
#[test]
fn no_worktrees() {
    let linked = EditorRepository::linked("repo", "/tmp/worktrees/repo").unwrap();
    assert!(open_checkout(&linked).is_err());
    let command = managed_clone_command("file:///repo.git", Path::new("/tmp/repo")).unwrap();
    assert!(!command.contains("worktree"));
}
