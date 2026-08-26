//! The local, read-only marketplace manifest index.
//!
//! The index is a source of tool metadata, not an installer.  It resolves names for an
//! agent's allowlist and exposes a compact catalog; binaries remain behind the signed
//! admission boundary in [`super::tools::ToolCatalog`].

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// The package-manager names and package names used to install a tool at image-build time.
///
/// The resolver preserves this metadata but does not execute it.  Installation and signature
/// verification belong to the marketplace installer milestone.
pub type InstallSpec = BTreeMap<String, String>;

/// One `<name>.toml` file in the local marketplace index.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub name: String,
    pub summary: String,
    pub install: InstallSpec,
    pub verify: String,
    pub docs: String,
    #[serde(default)]
    pub caps: Vec<String>,
}

impl Manifest {
    /// The baseline CLIs used by the seeded agent definitions. These are metadata only; image
    /// admission still requires the trusted installer path.
    pub fn seeded_agent_cli_tools() -> Vec<Self> {
        [
            ("cargo", "Rust package manager and build tool"),
            ("gh", "GitHub CLI for issues and pull requests"),
            ("rg", "Fast recursive search through workspace files"),
        ]
        .into_iter()
        .map(|(name, summary)| Self {
            name: name.into(),
            summary: summary.into(),
            install: [("brew".into(), name.into())].into_iter().collect(),
            verify: format!("{name} --version"),
            docs: format!("docs/{name}.md"),
            caps: Vec::new(),
        })
        .collect()
    }

    /// Parse and validate one manifest.  Validation is intentionally strict so malformed
    /// metadata cannot silently become an image or an agent capability.
    pub fn parse(source: &str) -> Result<Self, ManifestError> {
        let manifest: Self = toml::from_str(source).map_err(ManifestError::Toml)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), ManifestError> {
        if !is_tool_name(&self.name) {
            return Err(ManifestError::Invalid {
                name: self.name.clone(),
                reason: "name must be a non-empty CLI name without whitespace or path separators",
            });
        }
        if self.summary.trim().is_empty() || self.summary.lines().count() != 1 {
            return Err(ManifestError::Invalid {
                name: self.name.clone(),
                reason: "summary must be one non-empty line",
            });
        }
        if self.install.is_empty()
            || self.install.iter().any(|(k, v)| {
                k.trim().is_empty() || v.trim().is_empty() || k.contains(char::is_whitespace)
            })
        {
            return Err(ManifestError::Invalid {
                name: self.name.clone(),
                reason: "install must contain non-empty package-manager entries",
            });
        }
        if self.verify.trim().is_empty() || self.docs.trim().is_empty() {
            return Err(ManifestError::Invalid {
                name: self.name.clone(),
                reason: "verify and docs must be non-empty",
            });
        }
        validate_relative_path(&self.docs).map_err(|reason| ManifestError::Invalid {
            name: self.name.clone(),
            reason,
        })?;
        let mut caps = BTreeSet::new();
        if self
            .caps
            .iter()
            .any(|cap| cap.trim().is_empty() || !caps.insert(cap))
        {
            return Err(ManifestError::Invalid {
                name: self.name.clone(),
                reason: "caps must contain unique non-empty values",
            });
        }
        Ok(())
    }

    pub fn canonical_toml(&self) -> Result<String> {
        toml::to_string(self).context("serialize marketplace manifest")
    }

    pub fn content_sha256(&self) -> Result<String> {
        Ok(format!(
            "{:x}",
            Sha256::digest(self.canonical_toml()?.as_bytes())
        ))
    }
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("manifest is malformed: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("manifest `{name}` is invalid: {reason}")]
    Invalid { name: String, reason: &'static str },
}

fn is_tool_name(name: &str) -> bool {
    !name.trim().is_empty()
        && name == name.trim()
        && !name
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '/' | '\\'))
}

fn validate_relative_path(path: &str) -> std::result::Result<(), &'static str> {
    let path = Path::new(path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err("docs must be a relative path inside the index");
    }
    Ok(())
}

/// A deterministic snapshot of all valid manifests under a directory.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ManifestIndex {
    root: PathBuf,
    manifests: BTreeMap<String, Manifest>,
}

impl ManifestIndex {
    pub fn read_from_directory(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let mut paths = fs::read_dir(root)
            .with_context(|| format!("read marketplace index `{}`", root.display()))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::result::Result<Vec<_>, _>>()
            .with_context(|| format!("read marketplace index entries `{}`", root.display()))?;
        paths.sort();

        let mut manifests = BTreeMap::new();
        for path in paths {
            if path.extension().and_then(|extension| extension.to_str()) != Some("toml") {
                continue;
            }
            let source = fs::read_to_string(&path)
                .with_context(|| format!("read marketplace manifest `{}`", path.display()))?;
            let manifest = Manifest::parse(&source).map_err(|error| {
                anyhow::anyhow!(
                    "{}: {error}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )
            })?;
            if manifests.contains_key(&manifest.name) {
                bail!("duplicate marketplace manifest name `{}`", manifest.name)
            }
            manifests.insert(manifest.name.clone(), manifest);
        }

        Ok(Self {
            root: root.to_path_buf(),
            manifests,
        })
    }

    pub fn get(&self, name: &str) -> Option<&Manifest> {
        self.manifests.get(name)
    }

    pub fn manifests(&self) -> impl Iterator<Item = &Manifest> {
        self.manifests.values()
    }

    /// Resolve and inject only the catalog for the supplied allowlist. Documentation is not
    /// read by this operation; callers must use `ToolResolution::docs` after choosing a tool.
    pub fn inject_catalog(&self, context: &str, tools: &[String]) -> Result<String> {
        let resolution = self.resolve(tools)?;
        Ok(inject_catalog(context, &resolution))
    }

    /// Resolve an agent's tool list in sorted order.  Unknown names are rejected together with
    /// the exact name, rather than being silently omitted from the image or catalog.
    pub fn resolve(&self, tools: &[String]) -> Result<ToolResolution> {
        let mut names = BTreeSet::new();
        for name in tools {
            if !names.insert(name.clone()) {
                continue;
            }
            if !self.manifests.contains_key(name) {
                bail!("tool `{name}` is absent from the marketplace index")
            }
        }
        Ok(ToolResolution {
            root: self.root.clone(),
            manifests: names
                .into_iter()
                .map(|name| self.manifests[&name].clone())
                .collect(),
        })
    }

    /// Return the full documentation page only after the caller has resolved the allowlist.
    pub fn docs(&self, allowlisted: &[String], name: &str) -> Result<String> {
        self.resolve(allowlisted)?.docs(name)
    }
}

/// The resolved, per-agent view of the marketplace.  It is the boundary used by catalog and docs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolResolution {
    root: PathBuf,
    manifests: Vec<Manifest>,
}

impl ToolResolution {
    pub fn manifests(&self) -> &[Manifest] {
        &self.manifests
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.manifests.iter().map(|manifest| manifest.name.as_str())
    }

    /// The CLI list is projected from the resolved allowlist, never from the full index.
    pub fn list(&self) -> Vec<String> {
        self.manifests
            .iter()
            .map(|manifest| manifest.name.clone())
            .collect()
    }

    /// Compact, one-line-per-tool context.  Deliberately excludes install commands, verify
    /// commands, capabilities, and documentation bodies.
    pub fn catalog(&self) -> String {
        self.manifests
            .iter()
            .map(|manifest| {
                format!(
                    "{}: {}",
                    manifest.name,
                    manifest
                        .summary
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// A stable, inexpensive estimate used by token-budget tests and metrics.
    pub fn catalog_tokens(&self) -> usize {
        self.catalog().split_whitespace().count()
    }

    pub fn docs(&self, name: &str) -> Result<String> {
        let manifest = self
            .manifests
            .iter()
            .find(|manifest| manifest.name == name)
            .with_context(|| format!("tool `{name}` is not allowlisted"))?;
        let root = self
            .root
            .canonicalize()
            .with_context(|| format!("canonicalize marketplace index `{}`", self.root.display()))?;
        let path = root.join(&manifest.docs);
        let canonical = path
            .canonicalize()
            .with_context(|| format!("read docs for tool `{name}`"))?;
        if !canonical.starts_with(&root) {
            bail!("docs for tool `{name}` leave the marketplace index")
        }
        fs::read_to_string(&canonical).with_context(|| format!("read docs for tool `{name}`"))
    }
}

/// Append a catalog to context without reading or embedding any documentation page.
pub fn inject_catalog(context: &str, resolution: &ToolResolution) -> String {
    if resolution.manifests.is_empty() {
        return context.to_owned();
    }
    format!("{context}\n\nAvailable tools:\n{}", resolution.catalog())
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod market {
    use super::*;
    use std::fs;

    fn manifest(name: &str, docs: &str) -> String {
        format!(
            "name = \"{name}\"\nsummary = \"A useful tool\"\ninstall = {{ cargo = \"{name}\" }}\nverify = \"{name} --version\"\ndocs = \"{docs}\"\ncaps = [\"search\"]\n"
        )
    }

    #[test]
    fn manifest_schema() {
        let parsed = Manifest::parse(&manifest("amq", "docs/amq.md")).unwrap();
        assert_eq!(parsed.install["cargo"], "amq");
    }

    #[test]
    fn reads_local_dir() {
        let root = tempfile_dir();
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(root.join("amq.toml"), manifest("amq", "docs/amq.md")).unwrap();
        fs::write(root.join("docs/amq.md"), "flags and examples").unwrap();
        let index = ManifestIndex::read_from_directory(&root).unwrap();
        assert_eq!(index.get("amq").unwrap().name, "amq");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn resolves_tools() {
        let root = tempfile_dir();
        fs::write(root.join("zeta.toml"), manifest("zeta", "zeta.md")).unwrap();
        fs::write(root.join("alpha.toml"), manifest("alpha", "alpha.md")).unwrap();
        let index = ManifestIndex::read_from_directory(&root).unwrap();
        let resolved = index.resolve(&["zeta".into(), "alpha".into()]).unwrap();
        assert_eq!(resolved.list(), vec!["alpha", "zeta"]);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_unknown_tool() {
        let root = tempfile_dir();
        let index = ManifestIndex::read_from_directory(&root).unwrap();
        let error = index.resolve(&["unknown".into()]).unwrap_err();
        assert!(error.to_string().contains("unknown"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn catalog() {
        let root = tempfile_dir();
        fs::write(root.join("amq.toml"), manifest("amq", "amq.md")).unwrap();
        let index = ManifestIndex::read_from_directory(&root).unwrap();
        assert_eq!(
            index.resolve(&["amq".into()]).unwrap().catalog(),
            "amq: A useful tool"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn catalog_cost() {
        let root = tempfile_dir();
        for name in (0..15).map(|number| format!("tool{number}")) {
            fs::write(
                root.join(format!("{name}.toml")),
                manifest(&name, "docs.md"),
            )
            .unwrap();
        }
        let index = ManifestIndex::read_from_directory(&root).unwrap();
        let tokens = index
            .resolve(&(0..15).map(|n| format!("tool{n}")).collect::<Vec<_>>())
            .unwrap()
            .catalog_tokens();
        assert!(
            (15..=75).contains(&tokens),
            "catalog should stay compact: {tokens} tokens"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn catalog_injected() {
        let root = tempfile_dir();
        fs::write(root.join("amq.toml"), manifest("amq", "amq.md")).unwrap();
        let index = ManifestIndex::read_from_directory(&root).unwrap();
        let context = index.inject_catalog("base", &["amq".into()]).unwrap();
        assert!(context.starts_with("base\n\nAvailable tools:"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn docs_only_when_allowlisted() {
        let root = tempfile_dir();
        fs::write(root.join("amq.toml"), manifest("amq", "amq.md")).unwrap();
        fs::write(root.join("amq.md"), "full flags and examples").unwrap();
        let index = ManifestIndex::read_from_directory(&root).unwrap();
        assert_eq!(
            index.docs(&["amq".into()], "amq").unwrap(),
            "full flags and examples"
        );
        assert!(index.docs(&[], "amq").is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn seeded() {
        let names: Vec<_> = Manifest::seeded_agent_cli_tools()
            .into_iter()
            .map(|manifest| manifest.name)
            .collect();
        assert_eq!(names, vec!["cargo", "gh", "rg"]);
    }

    #[test]
    fn validates() {
        let error = Manifest::parse("name = \"bad\"\nsummary = \"\"\n").unwrap_err();
        assert!(error.to_string().contains("bad"));
    }

    #[test]
    fn catalog_is_a_line() {
        let root = tempfile_dir();
        fs::write(root.join("amq.toml"), manifest("amq", "amq.md")).unwrap();
        let index = ManifestIndex::read_from_directory(&root).unwrap();
        let catalog = index.resolve(&["amq".into()]).unwrap().catalog();
        assert_eq!(catalog, "amq: A useful tool");
        assert!(!catalog.contains("--version"));
        assert!(!catalog.contains("cargo"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn allowlist_is_a_boundary() {
        let root = tempfile_dir();
        fs::write(root.join("amq.toml"), manifest("amq", "amq.md")).unwrap();
        let index = ManifestIndex::read_from_directory(&root).unwrap();
        let error = index.resolve(&["missing".into()]).unwrap_err();
        assert!(error.to_string().contains("missing"));
        let _ = fs::remove_dir_all(root);
    }

    fn tempfile_dir() -> PathBuf {
        let path = std::env::temp_dir().join(format!("locus-market-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
