//! The generated tree: what materialization produces, and the only thing that writes it.

use super::*;

/// A generated file held in memory until core writes it to the run tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub mode: u32,
    pub content: String,
}

/// Events that core, rather than a harness, must fire at a run boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreDrivenEvent {
    pub extension: String,
    pub events: Vec<String>,
}

/// A complete materialization result.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MaterializedTree {
    pub(super) files: BTreeMap<PathBuf, GeneratedFile>,
    pub core_driven: Vec<CoreDrivenEvent>,
}

impl MaterializedTree {
    pub fn files(&self) -> impl Iterator<Item = &GeneratedFile> {
        self.files.values()
    }

    pub fn file(&self, path: impl AsRef<Path>) -> Option<&GeneratedFile> {
        self.files.get(path.as_ref())
    }

    pub(super) fn put(&mut self, file: GeneratedFile) -> Result<(), MaterializeError> {
        let path = relative_path(&file.path)?;
        self.files
            .insert(path.clone(), GeneratedFile { path, ..file });
        Ok(())
    }

    pub(super) fn append(
        &mut self,
        path: PathBuf,
        content: String,
    ) -> Result<(), MaterializeError> {
        let path = relative_path(&path)?;
        if let Some(file) = self.files.get_mut(&path) {
            if !file.content.is_empty() && !content.is_empty() {
                file.content.push_str("\n\n");
            }
            file.content.push_str(&content);
        } else {
            self.put(GeneratedFile {
                path,
                mode: 0o644,
                content,
            })?;
        }
        Ok(())
    }

    /// Core is the sole writer. Once written, freeze the tree for the run lifetime.
    pub fn write_to(&self, root: impl AsRef<Path>) -> Result<(), MaterializeError> {
        let root = root.as_ref();
        fs::create_dir_all(root).map_err(|source| MaterializeError::Write {
            path: root.to_path_buf(),
            source,
        })?;
        for file in self.files.values() {
            let path = root.join(&file.path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|source| MaterializeError::Write {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            fs::write(&path, &file.content).map_err(|source| MaterializeError::Write {
                path: path.clone(),
                source,
            })?;
            set_mode(&path, file.mode)?;
        }
        freeze(root)
    }
}
