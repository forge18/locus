//! Data-only declarations describing how each supported harness launches,
//! reports telemetry, and consumes every Locus extension.

use serde::Deserialize;

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
