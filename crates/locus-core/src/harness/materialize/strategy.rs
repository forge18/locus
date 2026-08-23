//! The four data-parameterized strategies. Adding a harness never adds one.

use super::*;
use crate::harness::materialize::extensions::ExtensionEntry;
use crate::harness::materialize::tree::{GeneratedFile, MaterializedTree};

/// A generic strategy can turn one extension's entries into generated files.
pub trait Strategy {
    fn materialize(
        &self,
        entries: &[&ExtensionEntry],
        tree: &mut MaterializedTree,
    ) -> Result<(), MaterializeError>;
}

/// Copy files into a declared directory, optionally flattening and renaming them.
pub struct DirStrategy {
    pub dir: PathBuf,
    pub suffix: Option<String>,
    pub flat: bool,
}

impl Strategy for DirStrategy {
    fn materialize(
        &self,
        entries: &[&ExtensionEntry],
        tree: &mut MaterializedTree,
    ) -> Result<(), MaterializeError> {
        for entry in entries {
            let name = output_name(&entry.name, self.suffix.as_deref(), self.flat)?;
            tree.put(GeneratedFile {
                path: self.dir.join(name),
                mode: 0o644,
                content: entry.content(false),
            })?;
        }
        Ok(())
    }
}

/// Render entries as deterministic prose in one target file.
pub struct MergedIntoStrategy {
    pub target: PathBuf,
    pub strip_frontmatter: bool,
}

impl Strategy for MergedIntoStrategy {
    fn materialize(
        &self,
        entries: &[&ExtensionEntry],
        tree: &mut MaterializedTree,
    ) -> Result<(), MaterializeError> {
        let content = entries
            .iter()
            .map(|entry| {
                format!(
                    "# {}\n{}",
                    entry.name,
                    entry.content(self.strip_frontmatter)
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        tree.append(self.target.clone(), content)
    }
}

/// Write deterministic paths into a JSON config key.
pub struct ListedInStrategy {
    pub target: PathBuf,
    pub key: String,
    pub paths: Vec<String>,
}

impl Strategy for ListedInStrategy {
    fn materialize(
        &self,
        _entries: &[&ExtensionEntry],
        tree: &mut MaterializedTree,
    ) -> Result<(), MaterializeError> {
        update_json_list(tree, &self.target, &self.key, self.paths.clone())
    }
}

/// Convert each entry into a structured config entry.
pub struct EntriesInStrategy {
    pub target: PathBuf,
    pub key: Option<String>,
    pub paths: Vec<String>,
}

impl Strategy for EntriesInStrategy {
    fn materialize(
        &self,
        entries: &[&ExtensionEntry],
        tree: &mut MaterializedTree,
    ) -> Result<(), MaterializeError> {
        let values = entries
            .iter()
            .zip(&self.paths)
            .map(|(entry, path)| {
                json!({
                    "name": entry.name,
                    "path": path,
                    "frontmatter": entry.frontmatter,
                    "body": entry.body,
                })
            })
            .collect::<Vec<_>>();
        update_json_value(
            tree,
            &self.target,
            self.key.as_deref().unwrap_or("entries"),
            Value::Array(values),
        )
    }
}
