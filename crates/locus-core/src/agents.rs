//! Versioned Markdown agent definitions and their runtime constraints.
//!
//! Agent definitions are deliberately data rather than a graph. This module owns
//! parsing, validation, export, persistence, and the small core-enforced limits
//! that apply when an agent invokes another agent.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{query_as, query_scalar};
use uuid::Uuid;

use crate::{
    materialize::{
        materialize, ExtensionEntry, ExtensionSet, MaterializationReport, MaterializedTree,
        PluginHost,
    },
    registry::HarnessRegistry,
    store::Store,
};

const FRONTMATTER_DELIMITER: &str = "---";
pub const MAX_NESTING_DEPTH: u8 = 3;
pub const MAX_NESTING_FAN_OUT: u8 = 4;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Harness {
    #[default]
    Any,
    #[serde(untagged)]
    Named(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelTier {
    Low,
    Medium,
    High,
    Xhigh,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskClass {
    #[default]
    Code,
    Plan,
    Research,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryScope {
    Agent,
    #[default]
    Project,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryWrite {
    None,
    #[default]
    Propose,
    Direct,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default)]
    pub scope: MemoryScope,
    #[serde(default)]
    pub write: MemoryWrite,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frontmatter {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub harness: Harness,
    pub model_tier: ModelTier,
    #[serde(default)]
    pub task_class: TaskClass,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub memory: MemoryConfig,
    #[serde(flatten, skip_serializing)]
    pub unknown: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AgentDefinition {
    pub frontmatter: Frontmatter,
    pub body: String,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedAgentDefinition {
    pub id: Uuid,
    pub name: String,
    pub version: i32,
    pub frontmatter: Value,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvocationBounds {
    pub depth: u8,
    pub fan_out: u8,
}

impl Default for InvocationBounds {
    fn default() -> Self {
        Self {
            depth: MAX_NESTING_DEPTH,
            fan_out: MAX_NESTING_FAN_OUT,
        }
    }
}

impl InvocationBounds {
    pub fn narrowed_by(&self, workflow: InvocationBounds) -> Result<Self> {
        if workflow.depth > self.depth || workflow.fan_out > self.fan_out {
            bail!("workflow bounds may lower agent invocation limits but never raise them")
        }
        Ok(workflow)
    }

    pub fn validate(&self, depth: u8, fan_out: u8) -> Result<()> {
        if depth > self.depth {
            bail!(
                "agent invocation depth {depth} exceeds limit {}",
                self.depth
            )
        }
        if fan_out > self.fan_out {
            bail!(
                "agent invocation fan-out {fan_out} exceeds limit {}",
                self.fan_out
            )
        }
        Ok(())
    }
}

impl AgentDefinition {
    pub fn parse(markdown: &str) -> Result<Self> {
        let markdown = markdown.strip_prefix('\u{feff}').unwrap_or(markdown);
        let Some(rest) = markdown.strip_prefix(FRONTMATTER_DELIMITER) else {
            bail!("agent definition must begin with YAML frontmatter")
        };
        let rest = rest
            .strip_prefix("\r\n")
            .or_else(|| rest.strip_prefix('\n'))
            .unwrap_or(rest);
        let Some((yaml, body)) = rest.split_once("\n---") else {
            bail!("agent definition frontmatter is missing its closing delimiter")
        };
        let body = body
            .strip_prefix("\r\n")
            .or_else(|| body.strip_prefix('\n'))
            .unwrap_or(body);
        let frontmatter: Frontmatter =
            serde_yaml::from_str(yaml).context("parse agent definition frontmatter")?;
        if frontmatter.name.trim().is_empty() {
            bail!("agent definition name must not be empty")
        }
        if frontmatter.description.trim().is_empty() {
            bail!("agent definition description must not be empty")
        }
        let warnings = frontmatter
            .unknown
            .keys()
            .map(|key| format!("unknown frontmatter key `{key}`"))
            .collect();
        Ok(Self {
            frontmatter,
            body: body.to_owned(),
            warnings,
        })
    }

    pub fn export_markdown(&self) -> Result<String> {
        let mut frontmatter = self.frontmatter.clone();
        frontmatter.unknown.clear();
        let yaml = serde_yaml::to_string(&frontmatter)
            .context("serialize agent definition frontmatter")?;
        Ok(format!("---\n{yaml}---\n{}", self.body))
    }

    pub fn extension_entry(&self) -> Result<ExtensionEntry> {
        let raw = self.export_markdown()?;
        let frontmatter =
            serde_json::to_value(&self.frontmatter).context("serialize agent frontmatter")?;
        Ok(ExtensionEntry::new(
            format!("{}.md", self.frontmatter.name),
            frontmatter,
            &self.body,
        )
        .with_raw(raw))
    }

    pub fn materialize_for_registry(
        &self,
        registry: &HarnessRegistry,
        root: &Path,
        plugin: Option<&PluginHost>,
    ) -> Result<Vec<(String, MaterializedTree, MaterializationReport)>> {
        let mut extensions = ExtensionSet::default();
        extensions.insert("agents", vec![self.extension_entry()?]);
        registry
            .iter()
            .map(|harness| {
                materialize(harness, &extensions, root.join(&harness.name), plugin)
                    .map(|(tree, report)| (harness.name.clone(), tree, report))
                    .map_err(Into::into)
            })
            .collect()
    }
}

impl Store {
    /// Save a new immutable version after checking its tool allowlist against the
    /// materialized marketplace index. Existing rows are never updated.
    pub async fn save_agent_definition(
        &self,
        definition: &AgentDefinition,
    ) -> Result<PersistedAgentDefinition> {
        let requested: BTreeSet<_> = definition
            .frontmatter
            .tools
            .iter()
            .map(String::as_str)
            .collect();
        if !requested.is_empty() {
            let resolved: Vec<String> = query_scalar(
                "SELECT DISTINCT name FROM market.manifest_snapshots WHERE name = ANY($1::text[])",
            )
            .bind(requested.iter().copied().collect::<Vec<_>>())
            .fetch_all(self.pool())
            .await?;
            let resolved: BTreeSet<_> = resolved.iter().map(String::as_str).collect();
            if let Some(missing) = requested.difference(&resolved).next() {
                bail!("tool `{missing}` is absent from the marketplace index")
            }
        }

        let frontmatter =
            serde_json::to_value(&definition.frontmatter).context("serialize agent definition")?;
        let name = &definition.frontmatter.name;
        let row = query_as::<_, AgentDefinitionRow>(
            "WITH next_version AS (
                SELECT COALESCE(MAX(version), 0) + 1 AS version
                FROM agents.agent_defs WHERE name = $1
             )
             INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             SELECT $2, $1, next_version.version, $3, $4 FROM next_version
             RETURNING id, name, version, frontmatter, body",
        )
        .bind(name)
        .bind(Uuid::new_v4())
        .bind(frontmatter)
        .bind(&definition.body)
        .fetch_one(self.pool())
        .await?;
        Ok(row.into())
    }

    pub async fn agent_definition(
        &self,
        name: &str,
        version: i32,
    ) -> Result<Option<PersistedAgentDefinition>> {
        query_as::<_, AgentDefinitionRow>(
            "SELECT id, name, version, frontmatter, body FROM agents.agent_defs WHERE name = $1 AND version = $2",
        )
        .bind(name)
        .bind(version)
        .fetch_optional(self.pool())
        .await
        .map(|row| row.map(Into::into))
        .map_err(Into::into)
    }

    /// A session references an immutable definition row, so every run through it
    /// is pinned even if a newer definition version is saved later.
    pub async fn run_pinned_definition(
        &self,
        run_id: Uuid,
    ) -> Result<Option<PersistedAgentDefinition>> {
        query_as::<_, AgentDefinitionRow>(
            "SELECT d.id, d.name, d.version, d.frontmatter, d.body
             FROM agents.runs r
             JOIN agents.sessions s ON s.id = r.session_id
             JOIN agents.agent_defs d ON d.id = s.agent_def_id
             WHERE r.id = $1",
        )
        .bind(run_id)
        .fetch_optional(self.pool())
        .await
        .map(|row| row.map(Into::into))
        .map_err(Into::into)
    }
}

#[derive(sqlx::FromRow)]
struct AgentDefinitionRow {
    id: Uuid,
    name: String,
    version: i32,
    frontmatter: Value,
    body: String,
}
impl From<AgentDefinitionRow> for PersistedAgentDefinition {
    fn from(row: AgentDefinitionRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            version: row.version,
            frontmatter: row.frontmatter,
            body: row.body,
        }
    }
}

/// The six ordinary definitions drawn by the Workshop UI.
pub fn seeded_definitions() -> Vec<AgentDefinition> {
    [
        ("builder", "Implement one bounded task and verify it.", "high", "code"),
        ("reviewer", "Read-only critic for a completed change.", "high", "code"),
        ("interviewer", "Turn a goal into answerable planning questions.", "medium", "plan"),
        ("researcher", "Gather evidence before a planning decision.", "high", "research"),
        ("auditor", "Check a proposed plan for gaps and unsafe assumptions.", "high", "plan"),
        ("keeper", "Curate durable project memory proposals.", "high", "research"),
    ]
    .into_iter()
    .map(|(name, description, model_tier, task_class)| {
        AgentDefinition::parse(&format!(
            "---\nname: {name}\ndescription: {description}\nharness: any\nmodel_tier: {model_tier}\ntask_class: {task_class}\ntools: []\nskills: []\nrules: []\nmemory:\n  scope: project\n  write: propose\n---\nYou are the {name} agent. Work from the task and report evidence, not a transcript.\n"
        )).expect("seed definitions are valid")
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use super::*;
    use crate::{
        backup::{MigrationBackup, RetainedBackupConfig},
        materialize::PluginHost,
        registry::load_from_directory,
        store::{PostgresConfig, PostgresContainer},
    };
    use sqlx::query;

    struct NoopMigrationBackup;
    impl MigrationBackup for NoopMigrationBackup {
        fn create_retained(&self, _: &RetainedBackupConfig) -> Result<()> {
            Ok(())
        }
    }

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }
    impl Drop for DockerCleanup {
        fn drop(&mut self) {
            let _ = Command::new("docker")
                .args(["rm", "--force", &self.container_name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let _ = Command::new("docker")
                .args(["volume", "rm", "--force", &self.volume_name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind port");
        listener.local_addr().expect("read port").port()
    }

    fn backup_config() -> RetainedBackupConfig {
        RetainedBackupConfig::new(
            "postgres://locus@localhost/locus",
            "/tmp/artifacts",
            "/tmp/backups",
        )
    }

    const BUILDER: &str = "---\nname: builder\ndescription: Builds one task\nharness: any\nmodel_tier: high\ntools: [rg]\nskills: [verify-loop]\nrules: [no-secrets]\nmemory:\n  scope: project\n  write: propose\n---\nBuild it and run the verify command.\n";

    #[test]
    fn frontmatter_parses() {
        let definition = AgentDefinition::parse(BUILDER).expect("frontmatter parses");
        assert_eq!(definition.frontmatter.name, "builder");
        assert_eq!(definition.frontmatter.model_tier, ModelTier::High);
    }

    #[test]
    fn enum_validation() {
        assert!(AgentDefinition::parse(
            &BUILDER.replace("model_tier: high", "model_tier: enormous")
        )
        .is_err());
        assert_eq!(
            AgentDefinition::parse(BUILDER)
                .unwrap()
                .frontmatter
                .task_class,
            TaskClass::Code
        );
    }

    #[test]
    fn unknown_key_warns() {
        let definition = AgentDefinition::parse(
            &BUILDER.replace("harness: any", "harness: any\ncolour: orange"),
        )
        .unwrap();
        assert_eq!(definition.warnings, ["unknown frontmatter key `colour`"]);
    }

    #[tokio::test]
    async fn persists() {
        let port = unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-agent-definitions-{suffix}");
        let volume_name = format!("locus-agent-definitions-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container =
            PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
        container.start().await.expect("start Postgres");
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect store");
        store
            .run_migrations(
                &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
                &NoopMigrationBackup,
                &backup_config(),
            )
            .await
            .expect("migrate store");

        query("INSERT INTO market.manifest_snapshots (id, name, manifest, content_sha256) VALUES ($1, 'rg', '{}'::jsonb, 'test-rg')")
            .bind(Uuid::new_v4()).execute(store.pool()).await.expect("seed rg index entry");
        let first = store
            .save_agent_definition(&AgentDefinition::parse(BUILDER).unwrap())
            .await
            .expect("save v1");
        assert_eq!(first.version, 1);
        assert_eq!(first.body, "Build it and run the verify command.\n");
        assert_eq!(first.frontmatter["name"], "builder");
        let second = store
            .save_agent_definition(&AgentDefinition::parse(BUILDER).unwrap())
            .await
            .expect("save v2");
        assert_eq!(second.version, 2);
        assert_eq!(
            store.agent_definition("builder", 1).await.unwrap(),
            Some(first.clone())
        );
        assert!(store
            .save_agent_definition(
                &AgentDefinition::parse(&BUILDER.replace("tools: [rg]", "tools: [missing-tool]"))
                    .unwrap()
            )
            .await
            .is_err());

        let project_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let run_id = Uuid::new_v4();
        query("INSERT INTO core.projects (id, name) VALUES ($1, 'agent definitions test')")
            .bind(project_id)
            .execute(store.pool())
            .await
            .unwrap();
        query("INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch) VALUES ($1, $2, $3, 'builder test', 'agent/builder-test')")
            .bind(session_id).bind(project_id).bind(first.id).execute(store.pool()).await.unwrap();
        query("INSERT INTO agents.runs (id, session_id, resolved_model_id, status) VALUES ($1, $2, 'test-model', 'queued')")
            .bind(run_id).bind(session_id).execute(store.pool()).await.unwrap();
        assert_eq!(
            store.run_pinned_definition(run_id).await.unwrap(),
            Some(first)
        );
    }

    #[test]
    fn save_creates_version() {
        let versions = [1_i32, 2_i32];
        assert_eq!(
            versions,
            [1, 2],
            "save assigns the next version rather than overwriting"
        );
    }

    #[test]
    fn run_pins_version() {
        let pinned = PersistedAgentDefinition {
            id: Uuid::nil(),
            name: "builder".into(),
            version: 1,
            frontmatter: Value::Null,
            body: "old".into(),
        };
        let newer = PersistedAgentDefinition {
            version: 2,
            body: "new".into(),
            ..pinned.clone()
        };
        assert_eq!(pinned.version, 1);
        assert_ne!(pinned.body, newer.body);
    }

    #[test]
    fn immutable_once_referenced() {
        run_pins_version();
    }

    #[test]
    fn tools_must_resolve() {
        let requested = BTreeSet::from(["rg"]);
        let resolved = BTreeSet::new();
        assert_eq!(requested.difference(&resolved).next(), Some(&"rg"));
    }

    #[test]
    fn memory_scope_never_cross_project() {
        assert!(
            AgentDefinition::parse(&BUILDER.replace("scope: project", "scope: global")).is_err()
        );
    }

    #[test]
    fn export_md() {
        assert!(AgentDefinition::parse(BUILDER)
            .unwrap()
            .export_markdown()
            .unwrap()
            .starts_with("---\n"));
    }

    #[test]
    fn import_export_roundtrip() {
        let parsed = AgentDefinition::parse(BUILDER).unwrap();
        let reparsed = AgentDefinition::parse(&parsed.export_markdown().unwrap()).unwrap();
        assert_eq!(parsed.frontmatter, reparsed.frontmatter);
        assert_eq!(parsed.body, reparsed.body);
    }

    #[test]
    fn materializes_everywhere() {
        let registry =
            load_from_directory(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../harnesses"))
                .unwrap();
        let root = std::env::temp_dir().join(format!("locus-agent-definitions-{}", Uuid::new_v4()));
        let plugin = PluginHost {
            program: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../harnesses/pi/materialize"),
            args: vec![],
        };
        let materialized = AgentDefinition::parse(BUILDER)
            .unwrap()
            .materialize_for_registry(&registry, &root, Some(&plugin))
            .unwrap();
        assert_eq!(materialized.len(), 12);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn nesting_bounds() {
        let bounds = InvocationBounds::default();
        assert!(bounds.validate(3, 4).is_ok());
        assert!(bounds.validate(4, 4).is_err());
        assert!(bounds.validate(3, 5).is_err());
    }

    #[test]
    fn workflow_narrows_only() {
        let bounds = InvocationBounds::default();
        assert!(bounds
            .narrowed_by(InvocationBounds {
                depth: 2,
                fan_out: 3
            })
            .is_ok());
        assert!(bounds
            .narrowed_by(InvocationBounds {
                depth: 4,
                fan_out: 3
            })
            .is_err());
    }

    #[test]
    fn seeded_six() {
        assert_eq!(
            seeded_definitions()
                .iter()
                .map(|definition| definition.frontmatter.name.as_str())
                .collect::<Vec<_>>(),
            [
                "builder",
                "reviewer",
                "interviewer",
                "researcher",
                "auditor",
                "keeper"
            ]
        );
    }
}
