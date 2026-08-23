//! Deterministic per-run harness configuration materialization.
//!
//! The registry describes where a harness consumes an extension. This module applies those
//! declarations to an authored extension set without naming any individual harness.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::{
    lint::validate_filenames,
    registry::{HarnessDefinition, HarnessRegistry, LayoutEntry},
};

pub const EXTENSIONS: [&str; 8] = [
    "agents",
    "commands",
    "hooks",
    "linters",
    "output-styles",
    "rules",
    "skills",
    "context",
];

/// One authored extension file. `frontmatter` and `body` are the plugin-facing representation;
/// `raw` preserves an authored file for the `dir` strategy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtensionEntry {
    pub name: String,
    #[serde(default)]
    pub frontmatter: Value,
    #[serde(default)]
    pub body: String,
    #[serde(skip)]
    pub raw: Option<String>,
}

impl ExtensionEntry {
    pub fn new(name: impl Into<String>, frontmatter: Value, body: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            frontmatter,
            body: body.into(),
            raw: None,
        }
    }

    pub fn with_raw(mut self, raw: impl Into<String>) -> Self {
        self.raw = Some(raw.into());
        self
    }

    fn content(&self, strip_frontmatter: bool) -> String {
        if strip_frontmatter || self.frontmatter.is_null() || self.frontmatter == json!({}) {
            return self.body.clone();
        }
        if let Some(raw) = &self.raw {
            return raw.clone();
        }
        format!(
            "---\n{}\n---\n{}",
            serde_json::to_string(&self.frontmatter).expect("JSON frontmatter serializes"),
            self.body
        )
    }
}

/// Authored files grouped by the extension type that owns them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExtensionSet {
    entries: BTreeMap<String, Vec<ExtensionEntry>>,
}

/// Project toggles can remove extension groups or individual entries, but cannot add authored
/// extensions to a run.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectExtensionScope {
    #[serde(default)]
    disabled_extensions: BTreeSet<String>,
    #[serde(default)]
    disabled_entries: BTreeMap<String, BTreeSet<String>>,
}

impl ProjectExtensionScope {
    pub fn disable_extension(&mut self, extension: impl Into<String>) {
        self.disabled_extensions.insert(extension.into());
    }

    pub fn disable_entry(&mut self, extension: impl Into<String>, entry: impl Into<String>) {
        self.disabled_entries
            .entry(extension.into())
            .or_default()
            .insert(entry.into());
    }

    fn includes(&self, extension: &str, entry: &str) -> bool {
        !self.disabled_extensions.contains(extension)
            && !self
                .disabled_entries
                .get(extension)
                .is_some_and(|entries| entries.contains(entry))
    }
}

impl ExtensionSet {
    pub fn insert(&mut self, extension: impl Into<String>, entries: Vec<ExtensionEntry>) {
        self.entries.insert(extension.into(), entries);
    }

    /// Return the authored extensions after applying project-only subtraction.
    pub fn project_scoped(&self, scope: &ProjectExtensionScope) -> Self {
        Self {
            entries: self
                .entries
                .iter()
                .filter(|(extension, _)| !scope.disabled_extensions.contains(*extension))
                .map(|(extension, entries)| {
                    (
                        extension.clone(),
                        entries
                            .iter()
                            .filter(|entry| scope.includes(extension, &entry.name))
                            .cloned()
                            .collect(),
                    )
                })
                .collect(),
        }
    }

    pub fn entries(&self, extension: &str) -> &[ExtensionEntry] {
        self.entries
            .get(extension)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn sorted_entries(&self, extension: &str) -> Vec<&ExtensionEntry> {
        let mut entries = self.entries(extension).iter().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        entries
    }
}

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
    files: BTreeMap<PathBuf, GeneratedFile>,
    pub core_driven: Vec<CoreDrivenEvent>,
}

impl MaterializedTree {
    pub fn files(&self) -> impl Iterator<Item = &GeneratedFile> {
        self.files.values()
    }

    pub fn file(&self, path: impl AsRef<Path>) -> Option<&GeneratedFile> {
        self.files.get(path.as_ref())
    }

    fn put(&mut self, file: GeneratedFile) -> Result<(), MaterializeError> {
        let path = relative_path(&file.path)?;
        self.files
            .insert(path.clone(), GeneratedFile { path, ..file });
        Ok(())
    }

    fn append(&mut self, path: PathBuf, content: String) -> Result<(), MaterializeError> {
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

/// A loss in native behavior that the Extensions screen must show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterializationLoss {
    pub extension: String,
    pub weaker_than_native: String,
}

/// The registry-derived report displayed by the Extensions screen.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterializationReport {
    pub harness: String,
    pub losses: Vec<MaterializationLoss>,
}

pub fn reports_for_registry(registry: &HarnessRegistry) -> Vec<MaterializationReport> {
    registry
        .iter()
        .map(|harness| MaterializationReport {
            harness: harness.name.clone(),
            losses: harness
                .layout
                .named_entries()
                .into_iter()
                .filter_map(|(extension, entry)| {
                    entry
                        .weaker_than_native
                        .as_ref()
                        .map(|loss| MaterializationLoss {
                            extension: extension.into(),
                            weaker_than_native: loss.clone(),
                        })
                })
                .collect(),
        })
        .collect()
}

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

/// Errors that preserve why a run configuration could not be created.
#[derive(Debug, thiserror::Error)]
pub enum MaterializeError {
    #[error("unknown extension `{0}`")]
    UnknownExtension(String),
    #[error("layout entry for extension `{extension}` is missing `{field}`")]
    MissingLayoutField {
        extension: String,
        field: &'static str,
    },
    #[error("unsupported materialization strategy `{0}`")]
    UnsupportedStrategy(String),
    #[error("generated path `{0}` escapes the materialization root")]
    PathEscape(PathBuf),
    #[error("plugin wrote directly to the materialization root")]
    PluginWroteDirectly,
    #[error("plugin `{program}` failed: {message}")]
    PluginFailed { program: PathBuf, message: String },
    #[error("plugin response was not valid JSON: {0}")]
    PluginResponse(#[from] serde_json::Error),
    #[error("failed to write `{path}`: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("materialized tree is frozen")]
    Frozen,
    #[error("materializations were not byte-identical")]
    NonDeterministic,
    #[error("invalid linter definition: {0}")]
    InvalidLinter(String),
}

/// Plugin invocation settings. The plugin returns data; it never owns the config tree.
#[derive(Clone, Debug)]
pub struct PluginHost {
    pub program: PathBuf,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PluginRequest<'a> {
    jsonrpc: &'static str,
    id: u8,
    method: &'static str,
    params: PluginParams<'a>,
}

#[derive(Debug, Serialize)]
struct PluginParams<'a> {
    harness: &'a str,
    extension: &'a str,
    root: String,
    entries: Vec<&'a ExtensionEntry>,
    extensions: BTreeMap<&'a str, Vec<&'a ExtensionEntry>>,
}

#[derive(Debug, Deserialize)]
struct PluginResponse {
    #[serde(default)]
    result: Option<PluginResult>,
    #[serde(default)]
    files: Vec<PluginFile>,
}

#[derive(Debug, Deserialize)]
struct PluginResult {
    #[serde(default)]
    files: Vec<PluginFile>,
}

#[derive(Debug, Deserialize)]
struct PluginFile {
    path: PathBuf,
    #[serde(default = "default_mode")]
    mode: u32,
    content: String,
}

const fn default_mode() -> u32 {
    0o644
}

impl PluginHost {
    /// Invoke one plugin once for the complete plugin extension set of a run.
    fn materialize(
        &self,
        harness: &str,
        root: &Path,
        extensions: BTreeMap<&str, Vec<&ExtensionEntry>>,
    ) -> Result<Vec<GeneratedFile>, MaterializeError> {
        let before = snapshot(root)?;
        let entries = extensions.values().flatten().copied().collect();
        let request = PluginRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "materialize",
            params: PluginParams {
                harness,
                extension: "all",
                root: root.display().to_string(),
                entries,
                extensions,
            },
        };
        let mut child = Command::new(&self.program)
            .args(&self.args)
            .env("ROOT", root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| MaterializeError::PluginFailed {
                program: self.program.clone(),
                message: source.to_string(),
            })?;
        child
            .stdin
            .take()
            .expect("piped stdin exists")
            .write_all(
                format!(
                    "{}\n",
                    serde_json::to_string(&request).expect("request serializes")
                )
                .as_bytes(),
            )
            .map_err(|source| MaterializeError::PluginFailed {
                program: self.program.clone(),
                message: source.to_string(),
            })?;
        let output = child
            .wait_with_output()
            .map_err(|source| MaterializeError::PluginFailed {
                program: self.program.clone(),
                message: source.to_string(),
            })?;
        if snapshot(root)? != before {
            restore_snapshot(root, &before)?;
            return Err(MaterializeError::PluginWroteDirectly);
        }
        if !output.status.success() {
            return Err(MaterializeError::PluginFailed {
                program: self.program.clone(),
                message: String::from_utf8_lossy(&output.stderr).trim().into(),
            });
        }
        let response: PluginResponse = serde_json::from_slice(&output.stdout)?;
        let files = response
            .result
            .map(|result| result.files)
            .unwrap_or(response.files);
        files
            .into_iter()
            .map(|file| {
                let path = path_under_root(root, &file.path)?;
                Ok(GeneratedFile {
                    path,
                    mode: file.mode,
                    content: file.content,
                })
            })
            .collect()
    }
}

/// Materialize all eight extensions according to one registry declaration.
///
/// `root` is the real per-run config directory. Registry paths rooted at `/locus/config` are
/// translated below it, making this function equally usable in tests and in a container.
/// Materialize authored extensions after applying a project's subtractive extension policy.
///
/// The scope is applied at this boundary so disabled entries cannot leak into a generated run tree.
pub fn materialize_project(
    harness: &HarnessDefinition,
    extensions: &ExtensionSet,
    scope: &ProjectExtensionScope,
    root: impl AsRef<Path>,
    plugin: Option<&PluginHost>,
) -> Result<(MaterializedTree, MaterializationReport), MaterializeError> {
    materialize(harness, &extensions.project_scoped(scope), root, plugin)
}

pub fn materialize(
    harness: &HarnessDefinition,
    extensions: &ExtensionSet,
    root: impl AsRef<Path>,
    plugin: Option<&PluginHost>,
) -> Result<(MaterializedTree, MaterializationReport), MaterializeError> {
    let root = root.as_ref();
    fs::create_dir_all(root).map_err(|source| MaterializeError::Write {
        path: root.to_path_buf(),
        source,
    })?;
    let mut tree = MaterializedTree::default();
    let mut plugin_entries = BTreeMap::new();

    // Files must exist before merged context is appended to it.
    for (extension, entry) in harness.layout.named_entries() {
        let entries = extensions.sorted_entries(extension);
        if extension == "linters" {
            validate_filenames(entries.iter().map(|entry| &entry.name))
                .map_err(|error| MaterializeError::InvalidLinter(error.to_string()))?;
        }
        match entry.via.as_str() {
            "dir" => materialize_dir(extension, entry, &entries, root, &mut tree)?,
            "listed-in" => {
                materialize_dir(extension, entry, &entries, root, &mut tree)?;
            }
            "entries-in" => materialize_entry_files(extension, entry, &entries, root, &mut tree)?,
            "plugin" => {
                plugin_entries.insert(extension, entries);
            }
            "core-driven" => tree.core_driven.push(CoreDrivenEvent {
                extension: extension.into(),
                events: entry
                    .events
                    .clone()
                    .unwrap_or_else(|| vec!["session_start".into(), "session_end".into()]),
            }),
            "merged-into" => {}
            strategy => return Err(MaterializeError::UnsupportedStrategy(strategy.into())),
        }
    }

    for (extension, entry) in harness.layout.named_entries() {
        let entries = extensions.sorted_entries(extension);
        match entry.via.as_str() {
            "merged-into" => {
                let target = merge_target(harness, entry, root, extension)?;
                MergedIntoStrategy {
                    target,
                    strip_frontmatter: entry.strip_frontmatter.unwrap_or(false),
                }
                .materialize(&entries, &mut tree)?;
            }
            "listed-in" => {
                let target = layout_target(entry, root, extension)?;
                let logical_dir =
                    entry
                        .dir
                        .as_deref()
                        .ok_or_else(|| MaterializeError::MissingLayoutField {
                            extension: extension.into(),
                            field: "dir",
                        })?;
                let paths = entries
                    .iter()
                    .map(|generated| {
                        logical_path(
                            generated,
                            Path::new(logical_dir),
                            entry.suffix.as_deref(),
                            entry.flat.unwrap_or(false),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                ListedInStrategy {
                    target,
                    key: required(entry.key.as_deref(), extension, "key")?.into(),
                    paths,
                }
                .materialize(&entries, &mut tree)?;
            }
            "dir" => {
                if let (Some(target), Some(key), Some(active)) =
                    (&entry.target, &entry.key, &entry.active)
                {
                    update_json_value(
                        &mut tree,
                        &registry_path(root, target),
                        key,
                        Value::String(active.clone()),
                    )?;
                }
            }
            _ => {}
        }
    }

    if !plugin_entries.is_empty() {
        let plugin = plugin.ok_or_else(|| MaterializeError::MissingLayoutField {
            extension: "plugin".into(),
            field: "plugin executable",
        })?;
        for file in plugin.materialize(&harness.name, root, plugin_entries)? {
            tree.put(file)?;
        }
    }

    let report = MaterializationReport {
        harness: harness.name.clone(),
        losses: harness
            .layout
            .named_entries()
            .into_iter()
            .filter_map(|(extension, entry)| {
                entry
                    .weaker_than_native
                    .as_ref()
                    .map(|loss| MaterializationLoss {
                        extension: extension.into(),
                        weaker_than_native: loss.clone(),
                    })
            })
            .collect(),
    };
    Ok((tree, report))
}

fn materialize_dir(
    extension: &str,
    entry: &LayoutEntry,
    entries: &[&ExtensionEntry],
    root: &Path,
    tree: &mut MaterializedTree,
) -> Result<(), MaterializeError> {
    let dir = if let Some(dir) = &entry.dir {
        registry_path(root, dir)
    } else if let Some(file) = &entry.file {
        let file = registry_path(root, file);
        let content = entries
            .iter()
            .map(|entry| entry.content(false))
            .collect::<Vec<_>>()
            .join("\n");
        return tree.put(GeneratedFile {
            path: file,
            mode: 0o644,
            content,
        });
    } else {
        return Err(MaterializeError::MissingLayoutField {
            extension: extension.into(),
            field: "dir or file",
        });
    };
    DirStrategy {
        dir,
        suffix: entry.suffix.clone(),
        flat: entry.flat.unwrap_or(false),
    }
    .materialize(entries, tree)
}

fn materialize_entry_files(
    extension: &str,
    layout: &LayoutEntry,
    entries: &[&ExtensionEntry],
    root: &Path,
    tree: &mut MaterializedTree,
) -> Result<(), MaterializeError> {
    if let Some(dir) = &layout.dir {
        DirStrategy {
            dir: registry_path(root, dir),
            suffix: None,
            flat: false,
        }
        .materialize(entries, tree)?;
    }
    let target = layout_target(layout, root, extension)?;
    if target.extension().is_none() {
        // Codex's agents are one TOML file each. Keep the generic conversion data-driven.
        for entry in entries {
            let mut values = Map::new();
            if let Some(keys) = &layout.keys {
                for key in keys {
                    if key == "developer_instructions" {
                        values.insert(key.clone(), Value::String(entry.body.clone()));
                    } else if let Some(value) = entry.frontmatter.get(key) {
                        values.insert(key.clone(), value.clone());
                    }
                }
            }
            tree.put(GeneratedFile {
                path: target.join(output_name(&entry.name, Some(".toml"), true)?),
                mode: 0o644,
                content: toml::to_string(&Value::Object(values)).expect("TOML values serialize"),
            })?;
        }
        return Ok(());
    }
    let paths = entries
        .iter()
        .map(|entry| {
            layout
                .dir
                .as_ref()
                .map(|dir| logical_path(entry, Path::new(dir), None, false))
                .transpose()
                .map(|path| path.unwrap_or_else(|| entry.name.clone()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    EntriesInStrategy {
        target,
        key: layout.key.clone(),
        paths,
    }
    .materialize(entries, tree)
}

fn merge_target(
    harness: &HarnessDefinition,
    entry: &LayoutEntry,
    root: &Path,
    extension: &str,
) -> Result<PathBuf, MaterializeError> {
    match entry.target.as_deref() {
        Some("context") => harness
            .layout
            .context
            .file
            .as_ref()
            .map(|file| registry_path(root, file))
            .ok_or_else(|| MaterializeError::MissingLayoutField {
                extension: extension.into(),
                field: "context file",
            }),
        Some(target) => Ok(registry_path(root, target)),
        None => Err(MaterializeError::MissingLayoutField {
            extension: extension.into(),
            field: "target",
        }),
    }
}

fn layout_target(
    entry: &LayoutEntry,
    root: &Path,
    extension: &str,
) -> Result<PathBuf, MaterializeError> {
    entry
        .target
        .as_ref()
        .map(|target| registry_path(root, target))
        .ok_or_else(|| MaterializeError::MissingLayoutField {
            extension: extension.into(),
            field: "target",
        })
}

fn required<'a>(
    value: Option<&'a str>,
    extension: &str,
    field: &'static str,
) -> Result<&'a str, MaterializeError> {
    value.ok_or_else(|| MaterializeError::MissingLayoutField {
        extension: extension.into(),
        field,
    })
}

fn logical_path(
    entry: &ExtensionEntry,
    dir: &Path,
    suffix: Option<&str>,
    flat: bool,
) -> Result<String, MaterializeError> {
    Ok(dir
        .join(output_name(&entry.name, suffix, flat)?)
        .display()
        .to_string())
}

fn output_name(name: &str, suffix: Option<&str>, flat: bool) -> Result<PathBuf, MaterializeError> {
    let path = relative_path(Path::new(name))?;
    let path = if flat {
        PathBuf::from(
            path.file_name()
                .ok_or_else(|| MaterializeError::PathEscape(path.clone()))?,
        )
    } else {
        path
    };
    let Some(suffix) = suffix else {
        return Ok(path);
    };
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let stem = path
        .file_stem()
        .ok_or_else(|| MaterializeError::PathEscape(path.clone()))?;
    Ok(parent.join(format!("{}{}", stem.to_string_lossy(), suffix)))
}

fn registry_path(_root: &Path, path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let configured = Path::new("/locus/config");
    path.strip_prefix(configured)
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            if path.is_absolute() {
                path.strip_prefix("/")
                    .expect("absolute path has root")
                    .into()
            } else {
                path.into()
            }
        })
}

fn relative_path(path: &Path) -> Result<PathBuf, MaterializeError> {
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(MaterializeError::PathEscape(path.to_path_buf()));
    }
    Ok(path.to_path_buf())
}

fn path_under_root(root: &Path, path: &Path) -> Result<PathBuf, MaterializeError> {
    let candidate = if path.is_absolute() {
        path.strip_prefix(root)
            .map(PathBuf::from)
            .map_err(|_| MaterializeError::PathEscape(path.to_path_buf()))?
    } else {
        path.to_path_buf()
    };
    relative_path(&candidate)
}

fn update_json_list(
    tree: &mut MaterializedTree,
    target: &Path,
    key: &str,
    mut values: Vec<String>,
) -> Result<(), MaterializeError> {
    values.sort();
    values.dedup();
    let previous = json_value(tree, target)?;
    let mut values = previous
        .pointer(&format!("/{}", key.replace('.', "/")))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .chain(values)
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    update_json_value(
        tree,
        target,
        key,
        Value::Array(values.into_iter().map(Value::String).collect()),
    )
}

fn update_json_value(
    tree: &mut MaterializedTree,
    target: &Path,
    key: &str,
    value: Value,
) -> Result<(), MaterializeError> {
    let mut document = json_value(tree, target)?;
    set_json_key(&mut document, key, value);
    tree.put(GeneratedFile {
        path: target.to_path_buf(),
        mode: 0o644,
        content: format!(
            "{}\n",
            serde_json::to_string_pretty(&document).expect("JSON serializes")
        ),
    })
}

fn json_value(tree: &MaterializedTree, target: &Path) -> Result<Value, MaterializeError> {
    match tree.file(target) {
        Some(file) => serde_json::from_str(&file.content).map_err(MaterializeError::PluginResponse),
        None => Ok(json!({})),
    }
}

fn set_json_key(document: &mut Value, key: &str, value: Value) {
    let keys = key.split('.').collect::<Vec<_>>();
    let mut current = document
        .as_object_mut()
        .expect("materializer starts JSON objects");
    for key in &keys[..keys.len() - 1] {
        current = current
            .entry((*key).to_string())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .expect("generated key remains an object");
    }
    current.insert(keys.last().expect("key is non-empty").to_string(), value);
}

fn snapshot(root: &Path) -> Result<BTreeMap<PathBuf, Vec<u8>>, MaterializeError> {
    let mut files = BTreeMap::new();
    if !root.exists() {
        return Ok(files);
    }
    snapshot_directory(root, root, &mut files)?;
    Ok(files)
}

fn snapshot_directory(
    root: &Path,
    directory: &Path,
    files: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), MaterializeError> {
    let mut entries = fs::read_dir(directory)
        .map_err(|source| MaterializeError::Write {
            path: directory.into(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| MaterializeError::Write {
            path: directory.into(),
            source,
        })?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            snapshot_directory(root, &path, files)?;
        } else if path.is_file() {
            let mut content = Vec::new();
            fs::File::open(&path)
                .map_err(|source| MaterializeError::Write {
                    path: path.clone(),
                    source,
                })?
                .read_to_end(&mut content)
                .map_err(|source| MaterializeError::Write {
                    path: path.clone(),
                    source,
                })?;
            files.insert(
                path.strip_prefix(root).expect("walk remains rooted").into(),
                content,
            );
        }
    }
    Ok(())
}

fn restore_snapshot(
    root: &Path,
    files: &BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), MaterializeError> {
    if root.exists() {
        make_writable(root)?;
        fs::remove_dir_all(root).map_err(|source| MaterializeError::Write {
            path: root.into(),
            source,
        })?;
    }
    fs::create_dir_all(root).map_err(|source| MaterializeError::Write {
        path: root.into(),
        source,
    })?;
    for (relative, content) in files {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| MaterializeError::Write {
                path: parent.into(),
                source,
            })?;
        }
        fs::write(&path, content).map_err(|source| MaterializeError::Write { path, source })?;
    }
    Ok(())
}

fn freeze(root: &Path) -> Result<(), MaterializeError> {
    #[cfg(unix)]
    set_tree_modes(root, 0o555, 0o444)?;
    Ok(())
}

fn make_writable(root: &Path) -> Result<(), MaterializeError> {
    #[cfg(unix)]
    set_tree_modes(root, 0o755, 0o644)?;
    Ok(())
}

#[cfg(unix)]
fn set_tree_modes(
    root: &Path,
    directory_mode: u32,
    file_mode: u32,
) -> Result<(), MaterializeError> {
    let mut entries = fs::read_dir(root)
        .map_err(|source| MaterializeError::Write {
            path: root.into(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| MaterializeError::Write {
            path: root.into(),
            source,
        })?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            set_tree_modes(&path, directory_mode, file_mode)?;
        } else if path.is_file() {
            set_mode(&path, file_mode)?;
        }
    }
    set_mode(root, directory_mode)
}

fn set_mode(path: &Path, mode: u32) -> Result<(), MaterializeError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(|source| {
            MaterializeError::Write {
                path: path.into(),
                source,
            }
        })?;
    }
    let _ = mode;
    Ok(())
}

/// Execute a materializer twice and reject any byte-level difference.
pub fn assert_deterministic<F>(mut materialize: F) -> Result<(), MaterializeError>
where
    F: FnMut() -> Result<MaterializedTree, MaterializeError>,
{
    let first = materialize()?;
    let second = materialize()?;
    if first.files != second.files {
        return Err(MaterializeError::NonDeterministic);
    }
    Ok(())
}

#[cfg(test)]
use crate::registry::load_from_directory;
#[cfg(test)]
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(test)]
fn entry(name: &str, body: &str) -> ExtensionEntry {
    ExtensionEntry::new(name, json!({"name": name}), body)
        .with_raw(format!("---\nname: {name}\n---\n{body}"))
}

#[cfg(test)]
fn root(label: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "locus-materialize-{label}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

#[test]
fn trait_shape() {
    fn assert_strategy<T: Strategy>() {}
    assert_strategy::<DirStrategy>();
    assert_strategy::<MergedIntoStrategy>();
    assert_strategy::<ListedInStrategy>();
    assert_strategy::<EntriesInStrategy>();
    let mut extensions = ExtensionSet::default();
    extensions.insert("rules", vec![entry("one.md", "one")]);
    assert_eq!(extensions.entries("rules").len(), 1);
}

#[test]
fn dir() {
    let mut tree = MaterializedTree::default();
    DirStrategy {
        dir: PathBuf::from("agents"),
        suffix: Some(".agent.md".into()),
        flat: true,
    }
    .materialize(&[&entry("nested/reviewer.md", "review")], &mut tree)
    .expect("copy files");
    assert_eq!(
        tree.file("agents/reviewer.agent.md")
            .expect("renamed copy")
            .content,
        "---\nname: nested/reviewer.md\n---\nreview"
    );
}

#[test]
fn merged_into() {
    let mut tree = MaterializedTree::default();
    let mut entries = [entry("z.md", "z"), entry("a.md", "a")];
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    MergedIntoStrategy {
        target: PathBuf::from("AGENTS.md"),
        strip_frontmatter: true,
    }
    .materialize(&entries.iter().collect::<Vec<_>>(), &mut tree)
    .expect("merge files");
    assert_eq!(
        tree.file("AGENTS.md").expect("merged file").content,
        "# a.md\na\n\n# z.md\nz"
    );
}

#[test]
fn listed_in() {
    let mut tree = MaterializedTree::default();
    ListedInStrategy {
        target: PathBuf::from("config.json"),
        key: "instructions".into(),
        paths: vec!["z.md".into(), "a.md".into()],
    }
    .materialize(&[], &mut tree)
    .expect("write list");
    assert_eq!(
        tree.file("config.json").expect("config").content,
        "{\n  \"instructions\": [\n    \"a.md\",\n    \"z.md\"\n  ]\n}\n"
    );
}

#[test]
fn entries_in() {
    let mut tree = MaterializedTree::default();
    let entries = [entry("b.sh", "b"), entry("a.sh", "a")];
    EntriesInStrategy {
        target: PathBuf::from("settings.json"),
        key: Some("hooks".into()),
        paths: vec!["hooks/a.sh".into(), "hooks/b.sh".into()],
    }
    .materialize(&entries.iter().rev().collect::<Vec<_>>(), &mut tree)
    .expect("write entries");
    assert!(tree
        .file("settings.json")
        .expect("settings")
        .content
        .contains("hooks/a.sh"));
}

#[test]
fn core_driven() {
    let registry = registry();
    let (tree, _) = materialize(
        registry.by_name("aider").expect("aider"),
        &ExtensionSet::default(),
        root("core-driven"),
        None,
    )
    .expect("materialize");
    assert_eq!(tree.core_driven[0].events, ["session_start", "session_end"]);
}

#[cfg(all(test, unix))]
fn plugin_script(root: &Path, direct_write: bool) -> PluginHost {
    use std::os::unix::fs::PermissionsExt;
    let script = root.with_extension("sh");
    let body = if direct_write {
        "#!/bin/sh\nread request\nmkdir -p \"$ROOT\"\necho bad > \"$ROOT/direct\"\nprintf '{\"files\":[{\"path\":\"generated.ts\",\"content\":\"ok\"}]}'\n"
    } else {
        "#!/bin/sh\nread request\nprintf '{\"result\":{\"files\":[{\"path\":\"generated.ts\",\"mode\":420,\"content\":\"ok\"}]}}'\n"
    };
    fs::write(&script, body).expect("script writes");
    let mut permissions = fs::metadata(&script).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("script executable");
    PluginHost {
        program: script,
        args: Vec::new(),
    }
}

#[test]
#[cfg(unix)]
fn plugin_roundtrip() {
    let root = root("plugin");
    let files = plugin_script(&root, false)
        .materialize(
            "test",
            &root,
            BTreeMap::from([("hooks", vec![&entry("hook", "body")])]),
        )
        .expect("plugin returns files");
    assert_eq!(files[0].path, PathBuf::from("generated.ts"));
    assert_eq!(files[0].content, "ok");
    let _ = fs::remove_file(root.with_extension("sh"));
}

#[test]
#[cfg(unix)]
fn plugin_path_escape_rejected() {
    use std::os::unix::fs::PermissionsExt;
    let root = root("escape");
    let script = root.with_extension("sh");
    fs::write(&script, "#!/bin/sh\nread request\nprintf '{\"files\":[{\"path\":\"../escape\",\"content\":\"bad\"}]}'\n").expect("script");
    let mut permissions = fs::metadata(&script).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("permissions");
    let error = PluginHost {
        program: script.clone(),
        args: vec![],
    }
    .materialize("test", &root, BTreeMap::new())
    .expect_err("escape rejected");
    assert!(matches!(error, MaterializeError::PathEscape(_)));
    assert!(!root.exists(), "core has written nothing");
    let _ = fs::remove_file(script);
}

#[test]
#[cfg(unix)]
fn plugin_returns_never_writes() {
    let root = root("direct");
    let host = plugin_script(&root, true);
    let error = host
        .materialize("test", &root, BTreeMap::new())
        .expect_err("direct writes refused");
    assert!(matches!(error, MaterializeError::PluginWroteDirectly));
    assert!(!root.join("direct").exists());
    let _ = fs::remove_file(host.program);
}

#[test]
fn project_scope_subtracts() {
    use crate::tools::{ImageTool, ProjectToolScope, RoleToolScope, ToolCatalog, TrustedKeyStore};

    let mut extensions = ExtensionSet::default();
    extensions.insert(
        "rules",
        vec![entry("no-secrets.md", "never commit a credential")],
    );
    extensions.insert(
        "skills",
        vec![entry("verify/SKILL.md", "run the focused check")],
    );
    let mut scope = ProjectExtensionScope::default();
    scope.disable_extension("rules");
    scope.disable_entry("skills", "verify/SKILL.md");

    let registry = registry();
    let (tree, _) = materialize(
        registry.by_name("claude").expect("claude"),
        &extensions.project_scoped(&scope),
        root("project-scope"),
        None,
    )
    .expect("materialize scoped extensions");
    assert!(tree.file("rules/no-secrets.md").is_none());
    assert!(tree.file("skills/verify/SKILL.md").is_none());

    let mut catalog = ToolCatalog::new(TrustedKeyStore::default());
    for tool in [
        ImageTool::new("git", "2.49"),
        ImageTool::new("rg", "14.1"),
        ImageTool::new("sqlx", "0.8"),
    ] {
        catalog.add_builtin(tool).expect("add built-in");
    }
    let project_tools = ProjectToolScope::new(["sqlx"]);
    let role_tools = RoleToolScope::new(["git"]);
    assert_eq!(
        catalog.scoped_image_set(&project_tools, &role_tools),
        vec![ImageTool::new("rg", "14.1")]
    );
}

#[test]
fn sorted_file_order() {
    let mut extensions = ExtensionSet::default();
    extensions.insert("rules", vec![entry("z.md", "z"), entry("a.md", "a")]);
    let registry = registry();
    let (tree, _) = materialize(
        registry.by_name("claude").expect("claude"),
        &extensions,
        root("order"),
        None,
    )
    .expect("tree");
    assert_eq!(
        tree.files()
            .map(|file| file.path.clone())
            .collect::<Vec<_>>(),
        tree.files()
            .map(|file| file.path.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    );
}

#[test]
fn sorted_inner_lists() {
    let mut tree = MaterializedTree::default();
    ListedInStrategy {
        target: PathBuf::from("config.json"),
        key: "instructions".into(),
        paths: vec!["z.md".into(), "a.md".into()],
    }
    .materialize(&[], &mut tree)
    .expect("list");
    assert!(
        tree.file("config.json")
            .expect("config")
            .content
            .find("a.md")
            .unwrap()
            < tree
                .file("config.json")
                .expect("config")
                .content
                .find("z.md")
                .unwrap()
    );
}

#[test]
fn no_volatile_content() {
    let mut extensions = ExtensionSet::default();
    extensions.insert("context", vec![entry("base.md", "stable")]);
    let registry = registry();
    let (tree, _) = materialize(
        registry.by_name("claude").expect("claude"),
        &extensions,
        root("volatile"),
        None,
    )
    .expect("tree");
    let output = tree
        .files()
        .map(|file| file.content.as_str())
        .collect::<String>();
    assert!(!output.contains("hostname") && !output.contains("run_id") && !output.contains("202"));
}

#[test]
fn ci_determinism() {
    let registry = registry();
    let harness = registry.by_name("claude").expect("claude");
    let mut extensions = ExtensionSet::default();
    extensions.insert("rules", vec![entry("rule.md", "body")]);
    assert_deterministic(|| Ok(materialize(harness, &extensions, root("identical"), None)?.0))
        .expect("identical trees");
}

#[test]
fn ci_detects_timestamp() {
    let mut call = 0;
    let error = assert_deterministic(|| {
        call += 1;
        let mut tree = MaterializedTree::default();
        tree.put(GeneratedFile {
            path: PathBuf::from("context"),
            mode: 0o644,
            content: format!("generated_at=2026-01-01T00:00:0{call}Z"),
        })?;
        Ok(tree)
    })
    .expect_err("changing output fails");
    assert!(matches!(error, MaterializeError::NonDeterministic));
}

#[test]
#[cfg(unix)]
fn tree_is_frozen() {
    let root = root("frozen");
    let mut tree = MaterializedTree::default();
    tree.put(GeneratedFile {
        path: PathBuf::from("rules/fixed.md"),
        mode: 0o644,
        content: "fixed".into(),
    })
    .expect("file");
    tree.write_to(&root).expect("write and freeze");
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(root.join("rules/fixed.md"))
            .expect("file")
            .permissions()
            .mode()
            & 0o777
    };
    assert_eq!(mode, 0o444);
    assert!(fs::write(root.join("rules/mid-run.md"), "blocked").is_err());
    make_writable(&root).expect("cleanup");
    let _ = fs::remove_dir_all(root);
}

#[test]
#[cfg(unix)]
fn pi_plugin_generates() {
    let registry = registry();
    let mut extensions = ExtensionSet::default();
    extensions.insert("hooks", vec![entry("audit.sh", "echo hook")]);
    extensions.insert("rules", vec![entry("safety.md", "never leak")]);
    let root = root("pi");
    let host = PluginHost {
        program: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses/pi/materialize"),
        args: vec![],
    };
    let (tree, _) = materialize(
        registry.by_name("pi").expect("pi"),
        &extensions,
        &root,
        Some(&host),
    )
    .expect("pi plugin");
    assert!(tree
        .file("extensions/locus-hooks.ts")
        .expect("hooks extension")
        .content
        .contains("audit.sh"));
    assert!(tree
        .file("extensions/locus-rules.ts")
        .expect("rules extension")
        .content
        .contains("never leak"));
}

#[test]
#[ignore = "requires Docker image: set LOCUS_PI_IMAGE and run with --ignored"]
fn pi_loads_generated() {
    let image = env::var("LOCUS_PI_IMAGE").expect("pi container image is configured");
    let registry = registry();
    let mut extensions = ExtensionSet::default();
    extensions.insert("hooks", vec![entry("audit.sh", "echo hook")]);
    extensions.insert("rules", vec![entry("safety.md", "never leak")]);
    let root = root("pi-container");
    let host = PluginHost {
        program: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses/pi/materialize"),
        args: vec![],
    };
    let (tree, _) = materialize(
        registry.by_name("pi").expect("pi"),
        &extensions,
        &root,
        Some(&host),
    )
    .expect("generate pi extensions");
    tree.write_to(&root).expect("freeze generated extensions");

    let status = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-v",
            &format!("{}:/locus/config:ro", root.display()),
            &image,
            "pi",
            "--no-extensions",
            "--extension",
            "/locus/config/extensions/locus-hooks.ts",
            "--extension",
            "/locus/config/extensions/locus-rules.ts",
            "--help",
        ])
        .status()
        .expect("run pi image");
    make_writable(&root).expect("cleanup");
    fs::remove_dir_all(root).expect("remove generated config");
    assert!(status.success(), "pi refused the generated extensions");
}

#[test]
fn all_registered_harnesses_all_eight() {
    let registry = registry();
    let mut extensions = ExtensionSet::default();
    for extension in EXTENSIONS {
        extensions.insert(
            extension,
            vec![entry(format!("{extension}.md").as_str(), extension)],
        );
    }
    for harness in registry.iter() {
        let root = root(&harness.name);
        let plugin = harness
            .layout
            .named_entries()
            .iter()
            .any(|(_, entry)| entry.via == "plugin")
            .then(|| PluginHost {
                program: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../harnesses/pi/materialize"),
                args: vec![],
            });
        let result = materialize(harness, &extensions, root, plugin.as_ref());
        assert!(result.is_ok(), "{}: {:?}", harness.name, result.err());
    }
}

#[cfg(test)]
mod lint {
    use super::*;

    #[test]
    fn materializes() {
        let registry = registry();
        let harness = registry.by_name("claude").expect("Claude registry entry");
        let mut extensions = ExtensionSet::default();
        extensions.insert(
            "linters",
            vec![
                ExtensionEntry::new("format.sh", json!({}), "exit 0"),
                ExtensionEntry::new("format.md", json!({}), "Run formatter before commit."),
            ],
        );

        let (tree, _) = materialize(harness, &extensions, root("linters"), None)
            .expect("linter pair materializes");
        assert_eq!(
            tree.file("linters/format.sh").expect("check").content,
            "exit 0"
        );
        assert_eq!(
            tree.file("linters/format.md").expect("rule").content,
            "Run formatter before commit."
        );
    }

    #[test]
    fn identical_across_harnesses() {
        let mut extensions = ExtensionSet::default();
        extensions.insert(
            "linters",
            vec![
                ExtensionEntry::new("format.sh", json!({}), "exit 0"),
                ExtensionEntry::new("format.md", json!({}), "Run formatter before commit."),
            ],
        );

        for harness in registry().iter() {
            let root = root(format!("linters-{}", harness.name).as_str());
            let plugin = harness
                .layout
                .named_entries()
                .iter()
                .any(|(_, entry)| entry.via == "plugin")
                .then(|| PluginHost {
                    program: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../harnesses/pi/materialize"),
                    args: vec![],
                });
            let (tree, _) = materialize(harness, &extensions, root, plugin.as_ref())
                .unwrap_or_else(|error| panic!("{}: {error}", harness.name));
            assert_eq!(
                tree.file("linters/format.sh").expect("check").content,
                "exit 0"
            );
            assert_eq!(
                tree.file("linters/format.md").expect("rule").content,
                "Run formatter before commit."
            );
        }
    }
}

#[test]
fn report_carries_losses() {
    let reports = reports_for_registry(&registry());
    assert_eq!(reports.len(), 11);
    assert_eq!(reports.iter().flat_map(|report| &report.losses).count(), 29);
    assert!(reports
        .iter()
        .flat_map(|report| &report.losses)
        .all(|loss| !loss.weaker_than_native.is_empty()));
}

#[cfg(test)]
fn registry() -> HarnessRegistry {
    load_from_directory(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses"))
        .expect("registry loads")
}
