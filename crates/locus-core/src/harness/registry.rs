//! Data-only declarations describing how each supported harness launches,
//! reports telemetry, and consumes every Locus extension.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

/// An error encountered while reading or parsing a harness registry.
#[derive(Debug, thiserror::Error)]
pub enum RegistryLoadError {
    #[error("failed to read harness registry directory `{path}`: {source}")]
    ReadDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to inspect entry in harness registry directory `{directory}`: {source}")]
    ReadEntry {
        directory: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to inspect harness registry path `{path}`: {source}")]
    FileType {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read harness definition `{path}`: {source}")]
    ReadDefinition {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("harness definition `{path}` materializes an extension via a plugin but declares no `materializer`")]
    MissingMaterializer { path: PathBuf },
    #[error("failed to parse harness definition `{path}`: {source}")]
    ParseDefinition {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("harness definition `{path}` is missing required layout extension `{extension}`")]
    MissingLayoutExtension {
        path: PathBuf,
        extension: &'static str,
    },
    #[error(
        "harness definition `{path}` has unknown materialization strategy `{via}` for layout extension `{extension}`"
    )]
    UnknownMaterializationStrategy {
        path: PathBuf,
        extension: &'static str,
        via: String,
    },
    #[error(
        "harness definition `{path}` uses downgraded materialization strategy `{via}` for layout extension `{extension}` without required `weaker_than_native` explanation"
    )]
    MissingWeakerThanNative {
        path: PathBuf,
        extension: &'static str,
        via: String,
    },
    #[error("harness definition `{path}` has `tui = true`; TUI harnesses are unsupported")]
    TuiUnsupported { path: PathBuf },
    #[error("harness definition `{path}` is missing required telemetry source")]
    MissingTelemetrySource { path: PathBuf },
    #[error(
        "harness definition `{path}` has unknown telemetry source `{telemetry_source}`; expected `acp`"
    )]
    UnknownTelemetrySource {
        path: PathBuf,
        telemetry_source: String,
    },
}

/// A registry is accepted only after each harness passes the deterministic canary preflight.
#[derive(Debug, thiserror::Error)]
pub enum RegistryRegistrationError {
    #[error(transparent)]
    Load(#[from] RegistryLoadError),
    #[error("harness `{harness}` failed its canary smoke test: {reason}")]
    SmokeFailed { harness: String, reason: String },
}

/// Registry-wide materialization counts consumed by the UI.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryCounts {
    pub entries: usize,
    pub downgrades: usize,
}

/// The harness declarations registered with Locus.
#[derive(Debug)]
pub struct HarnessRegistry {
    definitions: Vec<HarnessDefinition>,
}

impl HarnessRegistry {
    /// Return the harness whose declared name exactly matches `name`.
    pub fn by_name(&self, name: &str) -> Option<&HarnessDefinition> {
        self.definitions
            .iter()
            .find(|definition| definition.name == name)
    }

    /// Return harnesses whose telemetry declaration exactly matches `source`.
    pub fn by_telemetry_source<'a, 'b>(
        &'a self,
        source: &'b str,
    ) -> impl Iterator<Item = &'a HarnessDefinition> + 'b
    where
        'a: 'b,
    {
        self.definitions
            .iter()
            .filter(move |definition| definition.telemetry.source == source)
    }

    /// Return harnesses that declare every requested telemetry verb.
    ///
    /// Harnesses with no `emits` declaration never match a non-empty request.
    pub fn by_declared_verbs<'a, 'b>(
        &'a self,
        verbs: &'b [&'b str],
    ) -> impl Iterator<Item = &'a HarnessDefinition> + 'b
    where
        'a: 'b,
    {
        self.definitions.iter().filter(move |definition| {
            definition.telemetry.emits.as_deref().is_some_and(|emits| {
                verbs
                    .iter()
                    .all(|verb| emits.iter().any(|emit| emit == verb))
            })
        })
    }

    /// Return the number of registered harness definitions.
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Return whether there are no registered harness definitions.
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Count layout entries and entries that declare a native-behavior loss.
    pub fn counts(&self) -> RegistryCounts {
        self.definitions.iter().fold(
            RegistryCounts {
                entries: 0,
                downgrades: 0,
            },
            |mut counts, definition| {
                for entry in definition.layout.entries() {
                    counts.entries += 1;
                    counts.downgrades += usize::from(entry.weaker_than_native.is_some());
                }
                counts
            },
        )
    }

    /// Return an iterator over registered definitions in deterministic name order.
    pub fn iter(&self) -> impl Iterator<Item = &HarnessDefinition> {
        self.definitions.iter()
    }
}

/// Load every harness definition directly in `directory` or in one plugin subdirectory.
///
/// Definition paths are sorted before parsing so callers receive a stable order regardless of
/// filesystem enumeration order.
pub fn load_from_directory(
    directory: impl AsRef<Path>,
) -> Result<HarnessRegistry, RegistryLoadError> {
    let directory = directory.as_ref();
    let mut definitions = toml_files_in(directory)?;

    for entry in directory_entries(directory)? {
        let path = entry.path();
        if entry
            .file_type()
            .map_err(|source| RegistryLoadError::FileType {
                path: path.clone(),
                source,
            })?
            .is_dir()
        {
            definitions.extend(toml_files_in(&path)?);
        }
    }

    definitions.sort();
    definitions
        .into_iter()
        .map(|path| {
            let definition =
                fs::read_to_string(&path).map_err(|source| RegistryLoadError::ReadDefinition {
                    path: path.clone(),
                    source,
                })?;
            let mut document: toml::Value = toml::from_str(&definition).map_err(|source| {
                RegistryLoadError::ParseDefinition {
                    path: path.clone(),
                    source,
                }
            })?;
            validate_layout_extensions(&mut document, &path)?;
            validate_tui(&document, &path)?;
            validate_telemetry_source(&document, &path)?;
            validate_materializer(&document, &path)?;
            HarnessDefinition::deserialize(document)
                .map(|mut definition| {
                    definition.registry_root = directory.to_path_buf();
                    definition
                })
                .map_err(|source| RegistryLoadError::ParseDefinition { path, source })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|definitions| HarnessRegistry { definitions })
}

/// Load and accept a registry only when every definition can expose both canary fixtures.
pub fn register_from_directory(
    directory: impl AsRef<Path>,
) -> Result<HarnessRegistry, RegistryRegistrationError> {
    let registry = load_from_directory(directory)?;
    for harness in registry.iter() {
        crate::harness::canary::run_canary_smoke(harness).map_err(|error| {
            RegistryRegistrationError::SmokeFailed {
                harness: harness.name.clone(),
                reason: error.to_string(),
            }
        })?;
    }
    Ok(registry)
}

const REQUIRED_LAYOUT_EXTENSIONS: &[&str] = &[
    "agents",
    "commands",
    "hooks",
    "linters",
    "output-styles",
    "rules",
    "skills",
    "context",
];

/// How one extension reaches a harness.
///
/// An enum rather than a string, so adding a seventh strategy is one variant plus one
/// `Strategy` implementation and the compiler names every place that must handle it.
/// Previously this was a `const &[&str]`, a validator, and two `match` arms in two passes
/// — four places nothing linked together.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", try_from = "String")]
pub enum Via {
    Dir,
    MergedInto,
    ListedIn,
    EntriesIn,
    Plugin,
    CoreDriven,
}

impl Via {
    /// Whether this strategy is weaker than a native mechanism, so the entry has to say
    /// what was lost. The registry counts these to produce the downgrade figure.
    pub fn is_downgrade(self) -> bool {
        matches!(self, Self::MergedInto | Self::ListedIn | Self::CoreDriven)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dir => "dir",
            Self::MergedInto => "merged-into",
            Self::ListedIn => "listed-in",
            Self::EntriesIn => "entries-in",
            Self::Plugin => "plugin",
            Self::CoreDriven => "core-driven",
        }
    }
}

impl std::str::FromStr for Via {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            // `file` is a compatibility alias: one target file is a directory of one.
            "dir" | "file" => Self::Dir,
            "merged-into" => Self::MergedInto,
            "listed-in" => Self::ListedIn,
            "entries-in" => Self::EntriesIn,
            "plugin" => Self::Plugin,
            "core-driven" => Self::CoreDriven,
            other => return Err(other.into()),
        })
    }
}

impl TryFrom<String> for Via {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl std::fmt::Display for Via {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

fn validate_layout_extensions(
    document: &mut toml::Value,
    path: &Path,
) -> Result<(), RegistryLoadError> {
    let Some(layout) = document
        .get_mut("layout")
        .and_then(toml::Value::as_table_mut)
    else {
        return Err(RegistryLoadError::MissingLayoutExtension {
            path: path.to_path_buf(),
            extension: REQUIRED_LAYOUT_EXTENSIONS[0],
        });
    };

    for extension in REQUIRED_LAYOUT_EXTENSIONS {
        let Some(entry) = layout
            .get_mut(*extension)
            .and_then(toml::Value::as_table_mut)
        else {
            return Err(RegistryLoadError::MissingLayoutExtension {
                path: path.to_path_buf(),
                extension,
            });
        };
        let Some(via) = entry
            .get("via")
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
        else {
            continue;
        };

        let Ok(strategy) = via.parse::<Via>() else {
            return Err(RegistryLoadError::UnknownMaterializationStrategy {
                path: path.to_path_buf(),
                extension,
                via,
            });
        };
        // Normalize the `file` alias in the document so the parsed entry has one spelling.
        entry.insert("via".into(), toml::Value::String(strategy.as_str().into()));
        if strategy.is_downgrade()
            && !entry
                .get("weaker_than_native")
                .and_then(toml::Value::as_str)
                .is_some_and(|explanation| !explanation.trim().is_empty())
        {
            return Err(RegistryLoadError::MissingWeakerThanNative {
                path: path.to_path_buf(),
                extension,
                via,
            });
        }
    }

    Ok(())
}

fn validate_tui(document: &toml::Value, path: &Path) -> Result<(), RegistryLoadError> {
    if document
        .get("launch")
        .and_then(|launch| launch.get("tui"))
        .and_then(toml::Value::as_bool)
        == Some(true)
    {
        return Err(RegistryLoadError::TuiUnsupported {
            path: path.to_path_buf(),
        });
    }

    Ok(())
}

// ACP is the only harness interface, so it is the only telemetry source. `hooks`,
// `stream-json`, and `session-log` are retired — see PLAN.md §ACP and .specs/telemetry.
const TELEMETRY_SOURCES: &[&str] = &["acp"];

fn validate_telemetry_source(document: &toml::Value, path: &Path) -> Result<(), RegistryLoadError> {
    let Some(source) = document
        .get("telemetry")
        .and_then(|telemetry| telemetry.get("source"))
        .and_then(toml::Value::as_str)
    else {
        return Err(RegistryLoadError::MissingTelemetrySource {
            path: path.to_path_buf(),
        });
    };

    if !TELEMETRY_SOURCES.contains(&source) {
        return Err(RegistryLoadError::UnknownTelemetrySource {
            path: path.to_path_buf(),
            telemetry_source: source.into(),
        });
    }

    Ok(())
}

/// A harness whose config is code must say which executable generates it. Leaving it out is
/// how one harness's materializer silently runs on another's behalf.
fn validate_materializer(document: &toml::Value, path: &Path) -> Result<(), RegistryLoadError> {
    let uses_plugin = document
        .get("layout")
        .and_then(toml::Value::as_table)
        .is_some_and(|layout| {
            layout
                .values()
                .any(|entry| entry.get("via").and_then(toml::Value::as_str) == Some("plugin"))
        });

    if uses_plugin
        && document
            .get("materializer")
            .and_then(toml::Value::as_str)
            .is_none()
    {
        return Err(RegistryLoadError::MissingMaterializer {
            path: path.to_path_buf(),
        });
    }

    Ok(())
}

fn directory_entries(directory: &Path) -> Result<Vec<fs::DirEntry>, RegistryLoadError> {
    let entries = fs::read_dir(directory).map_err(|source| RegistryLoadError::ReadDirectory {
        path: directory.to_path_buf(),
        source,
    })?;
    entries
        .map(|entry| {
            entry.map_err(|source| RegistryLoadError::ReadEntry {
                directory: directory.to_path_buf(),
                source,
            })
        })
        .collect()
}

fn toml_files_in(directory: &Path) -> Result<Vec<PathBuf>, RegistryLoadError> {
    directory_entries(directory)?
        .into_iter()
        .try_fold(Vec::new(), |mut definitions, entry| {
            let path = entry.path();
            if entry
                .file_type()
                .map_err(|source| RegistryLoadError::FileType {
                    path: path.clone(),
                    source,
                })?
                .is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension == "toml")
            {
                definitions.push(path);
            }
            Ok(definitions)
        })
}

/// A complete harness declaration loaded from one registry TOML file.
#[derive(Debug, Deserialize)]
pub struct HarnessDefinition {
    /// The registry directory this declaration was loaded from. `materializer` resolves
    /// against it, so core reaches a harness's executable without ever spelling its name.
    #[serde(skip)]
    pub registry_root: PathBuf,
    /// The executable that materializes this harness's `plugin` extensions, relative to the
    /// registry root. Required whenever any layout entry declares `via = "plugin"`.
    pub materializer: Option<String>,
    pub name: String,
    pub binary: String,
    pub detect: Vec<String>,
    pub image: Image,
    pub launch: Launch,
    pub telemetry: Telemetry,
    pub models: Models,
    pub layout: Layout,
    pub config: Option<Config>,
    pub auth: Option<Auth>,
    pub hooks: Option<Hooks>,
    pub memory: Option<Memory>,
}

/// Declarative base-image metadata. An unverified entry is retained in the registry
/// but refused by the image builder rather than guessing an install command.
#[derive(Debug, Deserialize)]
pub struct Image {
    pub base: String,
    pub version: String,
    pub install: Vec<String>,
    #[serde(default)]
    pub env: Vec<String>,
    pub verified: bool,
}

/// The non-interactive command line that starts one terminal session.
#[derive(Debug, Deserialize)]
pub struct Launch {
    pub argv: Vec<String>,
    pub tui: bool,
}

/// The structured source from which a harness reports run events.
#[derive(Debug, Deserialize)]
pub struct Telemetry {
    pub source: String,
    pub argv: Option<Vec<String>>,
    pub log_dir: Option<String>,
    pub log: Option<String>,
    pub format: Option<String>,
    pub emits: Option<Vec<String>>,
    pub generated: Option<String>,
    pub bridge: Option<Bridge>,
}

/// A bridge plugin registered by a telemetry source.
#[derive(Debug, Deserialize)]
pub struct Bridge {
    pub plugin: String,
    pub config: String,
    pub registered_in: String,
}

/// The harness-specific mechanics for selecting a model.
#[derive(Debug, Deserialize)]
pub struct Models {
    pub flag: String,
    pub list_argv: Vec<String>,
}

/// The complete set of extensions a harness consumes.
#[derive(Debug, Deserialize)]
pub struct Layout {
    pub agents: LayoutEntry,
    pub commands: LayoutEntry,
    pub hooks: LayoutEntry,
    pub linters: LayoutEntry,
    #[serde(rename = "output-styles")]
    pub output_styles: LayoutEntry,
    pub rules: LayoutEntry,
    pub skills: LayoutEntry,
    pub context: LayoutEntry,
    pub config: Option<String>,
}

impl Layout {
    fn entries(&self) -> [&LayoutEntry; 8] {
        self.named_entries().map(|(_, entry)| entry)
    }

    /// Return every extension declaration paired with its stable extension name.
    pub fn named_entries(&self) -> [(&'static str, &LayoutEntry); 8] {
        [
            ("agents", &self.agents),
            ("commands", &self.commands),
            ("hooks", &self.hooks),
            ("linters", &self.linters),
            ("output-styles", &self.output_styles),
            ("rules", &self.rules),
            ("skills", &self.skills),
            ("context", &self.context),
        ]
    }
}

impl HarnessDefinition {
    /// The absolute path to this harness's materializer, when it declares one.
    pub fn materializer_program(&self) -> Option<PathBuf> {
        self.materializer
            .as_ref()
            .map(|program| self.registry_root.join(program))
    }
}

/// One extension's materialization declaration.
#[derive(Debug, Deserialize)]
pub struct LayoutEntry {
    pub via: Via,
    pub dir: Option<String>,
    pub format: Option<String>,
    pub target: Option<String>,
    pub key: Option<String>,
    pub keys: Option<Vec<String>>,
    /// Which of `keys` receives the entry body rather than a frontmatter value.
    pub body_key: Option<String>,
    pub active: Option<String>,
    pub suffix: Option<String>,
    pub flat: Option<bool>,
    pub strip_frontmatter: Option<bool>,
    pub weaker_than_native: Option<String>,
    pub file: Option<String>,
    pub flag: Option<String>,
    pub emits: Option<String>,
    pub enable_in: Option<String>,
    pub events: Option<Vec<String>>,
    pub schema: Option<String>,
}

/// An environment or home-directory override for a harness configuration tree.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub home_env: Option<String>,
    pub home: Option<String>,
}

/// Authentication values supplied at container start rather than baked into an image.
#[derive(Debug, Deserialize)]
pub struct Auth {
    pub env: Option<Vec<String>>,
    pub pre_auth: Option<String>,
}

/// A harness hook configuration.
#[derive(Debug, Deserialize)]
pub struct Hooks {
    pub config: Option<String>,
    pub key: Option<String>,
    pub events: Option<Vec<String>>,
    pub inject_session: Option<String>,
    pub inject_turn: Option<String>,
    pub generated: Option<String>,
    pub enable_in: Option<String>,
}

/// A harness's private memory location, when it has one.
#[derive(Debug, Deserialize)]
pub struct Memory {
    pub native: String,
    pub locus_owns: bool,
}

#[cfg(test)]
#[test]
fn schema_parses() {
    const DEFINITIONS: &[&str] = &[
        include_str!("../../../../harnesses/aider.toml"),
        include_str!("../../../../harnesses/antigravity.toml"),
        include_str!("../../../../harnesses/claude.toml"),
        include_str!("../../../../harnesses/codex.toml"),
        include_str!("../../../../harnesses/copilot.toml"),
        include_str!("../../../../harnesses/cursor.toml"),
        include_str!("../../../../harnesses/dsh.toml"),
        include_str!("../../../../harnesses/gemini.toml"),
        include_str!("../../../../harnesses/omp.toml"),
        include_str!("../../../../harnesses/opencode.toml"),
        include_str!("../../../../harnesses/pi/pi.toml"),
    ];

    for definition in DEFINITIONS {
        let definition: HarnessDefinition =
            toml::from_str(definition).expect("harness definition matches the schema");
        assert!(!definition.name.is_empty());
        assert!(!definition.binary.is_empty());
    }
}

#[cfg(test)]
#[test]
fn rejects_missing_extension() {
    const EXTENSIONS: &[&str] = &[
        "agents",
        "commands",
        "hooks",
        "linters",
        "output-styles",
        "rules",
        "skills",
        "context",
    ];

    for extension in EXTENSIONS {
        let registry = std::env::temp_dir().join(format!(
            "locus-registry-missing-{extension}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("the system clock is after the Unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&registry).expect("temporary registry directory exists");

        let mut definition: toml::Value =
            toml::from_str(include_str!("../../../../harnesses/claude.toml"))
                .expect("reference declaration parses");
        definition["layout"]
            .as_table_mut()
            .expect("reference declaration has a layout")
            .remove(*extension);
        let path = registry.join("missing.toml");
        std::fs::write(
            &path,
            toml::to_string(&definition).expect("declaration serializes"),
        )
        .expect("incomplete declaration can be written");

        let error = load_from_directory(&registry).expect_err("missing extension is refused");
        std::fs::remove_dir_all(registry).expect("temporary registry directory is removed");

        assert_eq!(
            error.to_string(),
            format!(
                "harness definition `{}` is missing required layout extension `{extension}`",
                path.display()
            )
        );
    }
}

#[cfg(test)]
#[test]
fn rejects_unknown_strategy() {
    let registry = std::env::temp_dir().join(format!(
        "locus-registry-unknown-strategy-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&registry).expect("temporary registry directory exists");

    let mut definition: toml::Value =
        toml::from_str(include_str!("../../../../harnesses/claude.toml"))
            .expect("reference declaration parses");
    definition["layout"]["agents"]["via"] = toml::Value::String("unknown".into());
    let path = registry.join("unknown.toml");
    std::fs::write(
        &path,
        toml::to_string(&definition).expect("declaration serializes"),
    )
    .expect("invalid declaration can be written");

    let error = load_from_directory(&registry).expect_err("unknown strategy is refused");
    std::fs::remove_dir_all(registry).expect("temporary registry directory is removed");

    assert_eq!(
        error.to_string(),
        format!(
            "harness definition `{}` has unknown materialization strategy `unknown` for layout extension `agents`",
            path.display()
        )
    );
}

#[cfg(test)]
#[test]
fn rejects_tui_true() {
    let registry = std::env::temp_dir().join(format!(
        "locus-registry-tui-true-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&registry).expect("temporary registry directory exists");

    let mut definition: toml::Value =
        toml::from_str(include_str!("../../../../harnesses/claude.toml"))
            .expect("reference declaration parses");
    definition["launch"]["tui"] = toml::Value::Boolean(true);
    let path = registry.join("tui-true.toml");
    std::fs::write(
        &path,
        toml::to_string(&definition).expect("declaration serializes"),
    )
    .expect("invalid declaration can be written");

    let error = load_from_directory(&registry).expect_err("TUI harness is refused");
    std::fs::remove_dir_all(registry).expect("temporary registry directory is removed");

    assert_eq!(
        error.to_string(),
        format!(
            "harness definition `{}` has `tui = true`; TUI harnesses are unsupported",
            path.display()
        )
    );
}

#[cfg(test)]
#[test]
fn rejects_bad_source() {
    let registry = std::env::temp_dir().join(format!(
        "locus-registry-bad-source-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&registry).expect("temporary registry directory exists");

    let mut definition: toml::Value =
        toml::from_str(include_str!("../../../../harnesses/claude.toml"))
            .expect("reference declaration parses");
    definition["telemetry"]["source"] = toml::Value::String("unknown".into());
    let unknown_path = registry.join("unknown.toml");
    std::fs::write(
        &unknown_path,
        toml::to_string(&definition).expect("declaration serializes"),
    )
    .expect("invalid declaration can be written");

    let error = load_from_directory(&registry).expect_err("unknown telemetry source is refused");
    assert_eq!(
        error.to_string(),
        format!(
            "harness definition `{}` has unknown telemetry source `unknown`; expected `acp`",
            unknown_path.display()
        )
    );

    std::fs::remove_file(&unknown_path).expect("unknown declaration is removed");
    definition["telemetry"]
        .as_table_mut()
        .expect("reference declaration has telemetry")
        .remove("source");
    let missing_path = registry.join("missing.toml");
    std::fs::write(
        &missing_path,
        toml::to_string(&definition).expect("declaration serializes"),
    )
    .expect("incomplete declaration can be written");

    let error = load_from_directory(&registry).expect_err("missing telemetry source is refused");
    std::fs::remove_dir_all(registry).expect("temporary registry directory is removed");
    assert_eq!(
        error.to_string(),
        format!(
            "harness definition `{}` is missing required telemetry source",
            missing_path.display()
        )
    );
}

#[cfg(test)]
#[test]
fn rejects_unexplained_downgrade() {
    let registry = std::env::temp_dir().join(format!(
        "locus-registry-unexplained-downgrade-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&registry).expect("temporary registry directory exists");

    let mut definition: toml::Value =
        toml::from_str(include_str!("../../../../harnesses/claude.toml"))
            .expect("reference declaration parses");
    definition["layout"]["agents"]["via"] = toml::Value::String("merged-into".into());
    let path = registry.join("unexplained.toml");
    std::fs::write(
        &path,
        toml::to_string(&definition).expect("declaration serializes"),
    )
    .expect("downgraded declaration can be written");

    let error = load_from_directory(&registry).expect_err("unexplained downgrade is refused");
    std::fs::remove_dir_all(registry).expect("temporary registry directory is removed");

    assert_eq!(
        error.to_string(),
        format!(
            "harness definition `{}` uses downgraded materialization strategy `merged-into` for layout extension `agents` without required `weaker_than_native` explanation",
            path.display()
        )
    );
}

#[cfg(test)]
#[test]
fn loads_all_registered_harnesses() {
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses");
    let registry = std::env::temp_dir().join(format!(
        "locus-registry-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&registry).expect("temporary registry directory exists");

    // Mirror the real tree, including the one-level subdirectory a code-carrying
    // harness uses, so the copy exercises the same scan the loader performs.
    let mut sources = vec![(source, registry.clone())];
    while let Some((from, into)) = sources.pop() {
        for entry in std::fs::read_dir(&from).expect("source registry can be read") {
            let path = entry.expect("source registry entry can be read").path();
            if path.is_dir() {
                let nested = into.join(path.file_name().expect("directory has a name"));
                std::fs::create_dir_all(&nested).expect("nested registry directory exists");
                sources.push((path, nested));
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("toml") {
                continue;
            }
            let destination = into.join(path.file_name().expect("TOML path has a file name"));
            std::fs::copy(path, destination).expect("harness declaration can be copied");
        }
    }

    let harnesses = load_from_directory(&registry).expect("registry definitions load");
    std::fs::remove_dir_all(registry).expect("temporary registry directory is removed");

    assert_eq!(harnesses.len(), 11);
    assert!(
        harnesses
            .iter()
            .all(|harness| harness.layout.context.via == Via::Dir),
        "the `file` compatibility alias is normalized to `dir`"
    );
    assert_eq!(
        harnesses
            .iter()
            .map(|harness| harness.name.as_str())
            .collect::<Vec<_>>(),
        [
            "aider",
            "antigravity",
            "claude",
            "codex",
            "copilot",
            "cursor",
            "dsh",
            "gemini",
            "omp",
            "opencode",
            "pi",
        ]
    );
}

#[cfg(test)]
#[test]
fn queries() {
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses");
    let harnesses = load_from_directory(source).expect("registry definitions load");

    assert_eq!(
        harnesses
            .by_name("claude")
            .expect("named harness exists")
            .binary,
        "claude"
    );
    assert!(harnesses.by_name("unknown").is_none());

    assert_eq!(
        harnesses
            .by_telemetry_source("acp")
            .map(|harness| harness.name.as_str())
            .collect::<Vec<_>>(),
        [
            "aider",
            "antigravity",
            "claude",
            "codex",
            "copilot",
            "cursor",
            "dsh",
            "gemini",
            "omp",
            "opencode",
            "pi",
        ]
    );
    assert_eq!(
        harnesses
            .by_declared_verbs(&["tool_call", "tool_result"])
            .map(|harness| harness.name.as_str())
            .collect::<Vec<_>>(),
        ["claude", "cursor"]
    );
}

#[cfg(test)]
#[test]
fn smoke_gates_registration() {
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses");
    let registry =
        register_from_directory(&source).expect("registered harnesses pass canary smoke");
    assert_eq!(registry.len(), 11);
}

#[cfg(test)]
#[test]
fn counts_are_88_and_29() {
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses");
    let harnesses = load_from_directory(source).expect("registry definitions load");

    assert_eq!(
        harnesses.counts(),
        RegistryCounts {
            entries: 88,
            downgrades: 29,
        }
    );
}
