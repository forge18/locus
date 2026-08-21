//! Data-only declarations describing how each supported harness launches,
//! reports telemetry, and consumes every Locus extension.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;

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
        "harness definition `{path}` has unknown telemetry source `{telemetry_source}`; expected one of `hooks`, `acp`, `stream-json`, `session-log`"
    )]
    UnknownTelemetrySource {
        path: PathBuf,
        telemetry_source: String,
    },
}

/// Load every harness definition directly in `directory` or in one plugin subdirectory.
///
/// Definition paths are sorted before parsing so callers receive a stable order regardless of
/// filesystem enumeration order.
pub fn load_from_directory(
    directory: impl AsRef<Path>,
) -> Result<Vec<HarnessDefinition>, RegistryLoadError> {
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
            HarnessDefinition::deserialize(document)
                .map_err(|source| RegistryLoadError::ParseDefinition { path, source })
        })
        .collect()
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

const MATERIALIZATION_STRATEGIES: &[&str] = &[
    "dir",
    "merged-into",
    "listed-in",
    "entries-in",
    "plugin",
    "core-driven",
];

const DOWNGRADED_MATERIALIZATION_STRATEGIES: &[&str] = &["merged-into", "listed-in", "core-driven"];

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
        let Some(via) = entry.get("via").and_then(toml::Value::as_str) else {
            continue;
        };

        if via == "file" {
            entry.insert("via".into(), toml::Value::String("dir".into()));
            continue;
        }
        if !MATERIALIZATION_STRATEGIES.contains(&via) {
            return Err(RegistryLoadError::UnknownMaterializationStrategy {
                path: path.to_path_buf(),
                extension,
                via: via.into(),
            });
        }
        if DOWNGRADED_MATERIALIZATION_STRATEGIES.contains(&via)
            && !entry
                .get("weaker_than_native")
                .and_then(toml::Value::as_str)
                .is_some_and(|explanation| !explanation.trim().is_empty())
        {
            return Err(RegistryLoadError::MissingWeakerThanNative {
                path: path.to_path_buf(),
                extension,
                via: via.into(),
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

const TELEMETRY_SOURCES: &[&str] = &["hooks", "acp", "stream-json", "session-log"];

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
    pub name: String,
    pub binary: String,
    pub detect: Vec<String>,
    pub launch: Launch,
    pub telemetry: Telemetry,
    pub models: Models,
    pub layout: Layout,
    pub config: Option<Config>,
    pub auth: Option<Auth>,
    pub hooks: Option<Hooks>,
    pub memory: Option<Memory>,
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

/// One extension's materialization declaration.
#[derive(Debug, Deserialize)]
pub struct LayoutEntry {
    pub via: String,
    pub dir: Option<String>,
    pub format: Option<String>,
    pub target: Option<String>,
    pub key: Option<String>,
    pub keys: Option<Vec<String>>,
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
        include_str!("../../../harnesses/aider.toml"),
        include_str!("../../../harnesses/antigravity.toml"),
        include_str!("../../../harnesses/claude.toml"),
        include_str!("../../../harnesses/codex.toml"),
        include_str!("../../../harnesses/copilot.toml"),
        include_str!("../../../harnesses/cursor.toml"),
        include_str!("../../../harnesses/dsh.toml"),
        include_str!("../../../harnesses/gemini.toml"),
        include_str!("../../../harnesses/hermes.toml"),
        include_str!("../../../harnesses/omp.toml"),
        include_str!("../../../harnesses/opencode.toml"),
        include_str!("../../../harnesses/pi.toml"),
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
            toml::from_str(include_str!("../../../harnesses/claude.toml"))
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
        toml::from_str(include_str!("../../../harnesses/claude.toml"))
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
        toml::from_str(include_str!("../../../harnesses/claude.toml"))
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
        toml::from_str(include_str!("../../../harnesses/claude.toml"))
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
            "harness definition `{}` has unknown telemetry source `unknown`; expected one of `hooks`, `acp`, `stream-json`, `session-log`",
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
        toml::from_str(include_str!("../../../harnesses/claude.toml"))
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
fn loads_all_twelve() {
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

    for entry in std::fs::read_dir(source).expect("source registry can be read") {
        let path = entry.expect("source registry entry can be read").path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("toml") {
            continue;
        }

        let destination = if path.file_name().and_then(|name| name.to_str()) == Some("pi.toml") {
            let plugin = registry.join("pi");
            std::fs::create_dir(&plugin).expect("plugin directory exists");
            plugin.join("pi.toml")
        } else {
            registry.join(path.file_name().expect("TOML path has a file name"))
        };
        std::fs::copy(path, destination).expect("harness declaration can be copied");
    }

    let harnesses = load_from_directory(&registry).expect("registry definitions load");
    std::fs::remove_dir_all(registry).expect("temporary registry directory is removed");

    assert_eq!(harnesses.len(), 12);
    assert!(
        harnesses
            .iter()
            .all(|harness| harness.layout.context.via == "dir"),
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
            "hermes",
            "omp",
            "opencode",
            "pi",
        ]
    );
}
