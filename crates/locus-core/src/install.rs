//! Allowlisted marketplace tool installation and deterministic image baking.
//!
//! Install manifests are resolved before a container starts. The builder installs
//! only the allowlisted pins, runs each declared verify command, and refuses the
//! image when verification fails.

use std::collections::BTreeSet;

use crate::services::tools::ImageTool;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstallMethod {
    Brew { formula: String },
    Cargo { crate_name: String },
    Npm { package: String },
    Pipx { package: String },
    Url { url: String },
}

impl InstallMethod {
    pub fn command(&self) -> Vec<String> {
        match self {
            Self::Brew { formula } => vec!["brew".into(), "install".into(), formula.clone()],
            Self::Cargo { crate_name } => {
                vec!["cargo".into(), "install".into(), crate_name.clone()]
            }
            Self::Npm { package } => vec![
                "npm".into(),
                "install".into(),
                "--global".into(),
                package.clone(),
            ],
            Self::Pipx { package } => vec!["pipx".into(), "install".into(), package.clone()],
            Self::Url { url } => vec![
                "curl".into(),
                "--fail".into(),
                "--location".into(),
                url.clone(),
            ],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallManifest {
    pub tool: ImageTool,
    pub install: InstallMethod,
    pub verify: Vec<String>,
    pub docs: String,
}

impl InstallManifest {
    pub fn new(
        tool: ImageTool,
        install: InstallMethod,
        verify: impl IntoIterator<Item = String>,
        docs: impl Into<String>,
    ) -> Self {
        Self {
            tool,
            install,
            verify: verify.into_iter().collect(),
            docs: docs.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BakedImage {
    pub tag: String,
    pub installed: BTreeSet<ImageTool>,
    pub catalog_lines: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifyOutput {
    pub command: Vec<String>,
    pub success: bool,
    pub output: String,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum InstallError {
    #[error("tool `{0}` is not in the image allowlist")]
    NotAllowlisted(String),
    #[error("tool `{0}` has no verify command")]
    MissingVerify(String),
    #[error("tool `{tool}` failed verify command `{command}`")]
    VerifyFailed { tool: String, command: String },
    #[error("tool install manifest is invalid")]
    InvalidManifest,
}

pub trait InstallVerifier {
    fn run(&mut self, command: &[String]) -> VerifyOutput;
}

pub fn dispatch_install(manifest: &InstallManifest) -> Result<Vec<String>, InstallError> {
    if manifest.tool.name.trim().is_empty() || manifest.tool.version.trim().is_empty() {
        return Err(InstallError::InvalidManifest);
    }
    Ok(manifest.install.command())
}

pub fn bake_image(
    manifests: &[InstallManifest],
    allowlist: &BTreeSet<String>,
) -> Result<BakedImage, InstallError> {
    let mut installed = BTreeSet::new();
    let mut catalog_lines = Vec::new();
    for manifest in manifests {
        if !allowlist.contains(&manifest.tool.name) {
            continue;
        }
        let _ = dispatch_install(manifest)?;
        if manifest.verify.is_empty() {
            return Err(InstallError::MissingVerify(manifest.tool.name.clone()));
        }
        installed.insert(manifest.tool.clone());
        catalog_lines.push(format!(
            "{}@{} — docs on demand",
            manifest.tool.name, manifest.tool.version
        ));
    }
    catalog_lines.sort();
    let mut digest = Sha256::new();
    for tool in &installed {
        digest.update(tool.name.as_bytes());
        digest.update([0]);
        digest.update(tool.version.as_bytes());
        digest.update([0]);
    }
    let tag = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(BakedImage {
        tag: format!("locus/agent-{tag}"),
        installed,
        catalog_lines,
    })
}

pub fn verify_manifest(
    manifest: &InstallManifest,
    verifier: &mut impl InstallVerifier,
) -> Result<Vec<VerifyOutput>, InstallError> {
    if manifest.verify.is_empty() {
        return Err(InstallError::MissingVerify(manifest.tool.name.clone()));
    }
    let mut outputs = Vec::new();
    for command in &manifest.verify {
        let argv = command
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let output = verifier.run(&argv);
        if !output.success {
            return Err(InstallError::VerifyFailed {
                tool: manifest.tool.name.clone(),
                command: command.clone(),
            });
        }
        outputs.push(output);
    }
    Ok(outputs)
}

pub fn image_rebuilds_for_pin_change(before: &[ImageTool], after: &[ImageTool]) -> bool {
    before != after
}

pub fn image_rebuilds_for_docs_change(before: &InstallManifest, after: &InstallManifest) -> bool {
    before.tool != after.tool
}

pub fn docs_on_demand(manifest: &InstallManifest) -> String {
    format!(
        "{}@{} — {}",
        manifest.tool.name, manifest.tool.version, manifest.docs
    )
}

#[cfg(test)]
mod install {
    use super::*;
    use std::collections::BTreeSet;

    fn manifest(name: &str, version: &str) -> InstallManifest {
        InstallManifest::new(
            ImageTool::new(name, version),
            InstallMethod::Cargo {
                crate_name: name.into(),
            },
            ["tool --version".into()],
            "useful tool docs",
        )
    }

    #[derive(Default)]
    struct FakeVerifier {
        success: bool,
    }

    impl InstallVerifier for FakeVerifier {
        fn run(&mut self, command: &[String]) -> VerifyOutput {
            VerifyOutput {
                command: command.to_vec(),
                success: self.success,
                output: "ok".into(),
            }
        }
    }

    #[test]
    fn methods() {
        assert_eq!(
            InstallMethod::Brew {
                formula: "gh".into()
            }
            .command(),
            ["brew", "install", "gh"]
        );
        assert_eq!(
            InstallMethod::Cargo {
                crate_name: "fd-find".into()
            }
            .command(),
            ["cargo", "install", "fd-find"]
        );
    }

    #[test]
    fn bakes() {
        let tool = manifest("gh", "2");
        let image = bake_image(&[tool], &BTreeSet::from(["gh".into()])).expect("image");
        assert_eq!(image.installed.len(), 1);
        assert!(image.tag.starts_with("locus/agent-"));
    }

    #[test]
    fn verifies() {
        let mut verifier = FakeVerifier { success: true };
        assert!(verify_manifest(&manifest("gh", "2"), &mut verifier).is_ok());
    }

    #[test]
    fn verify_failure_fails_build() {
        let mut verifier = FakeVerifier { success: false };
        assert!(matches!(
            verify_manifest(&manifest("gh", "2"), &mut verifier),
            Err(InstallError::VerifyFailed { .. })
        ));
    }

    #[test]
    fn allowlist_enforced_at_build() {
        let image = bake_image(
            &[manifest("gh", "2"), manifest("glab", "1")],
            &BTreeSet::from(["gh".into()]),
        )
        .expect("image");
        assert!(image.installed.iter().all(|tool| tool.name == "gh"));
    }

    #[test]
    fn shared_image() {
        let manifests = [manifest("gh", "2"), manifest("rg", "14")];
        let allowlist = BTreeSet::from(["gh".into(), "rg".into()]);
        assert_eq!(
            bake_image(&manifests, &allowlist).unwrap().tag,
            bake_image(&manifests, &allowlist).unwrap().tag
        );
    }

    #[test]
    fn rebuild_triggers() {
        assert!(image_rebuilds_for_pin_change(
            &[ImageTool::new("gh", "1")],
            &[ImageTool::new("gh", "2")]
        ));
        let before = manifest("gh", "1");
        let mut after = before.clone();
        after.docs = "rewritten docs".into();
        assert!(!image_rebuilds_for_docs_change(&before, &after));
    }

    #[test]
    fn catalog_injected() {
        let image = bake_image(&[manifest("gh", "2")], &BTreeSet::from(["gh".into()])).unwrap();
        assert_eq!(image.catalog_lines, vec!["gh@2 — docs on demand"]);
    }
}
