//! Persistence for agent definitions (`agents.agent_defs`).
//!
//! Moved out of `services/agents.rs` so every query in the crate lives under `store/`.

use crate::ids::RunId;
use anyhow::{bail, Context, Result};
use serde_json::Value;
use sqlx::query_as;
use sqlx::query_scalar;
use std::collections::BTreeSet;
use uuid::Uuid;

use crate::{
    services::agents::{AgentDefinition, PersistedAgentDefinition},
    store::Store,
};

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
        let mut transaction = self.pool().begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(name)
            .execute(&mut *transaction)
            .await
            .context("lock agent definition version")?;
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
        .fetch_one(&mut *transaction)
        .await?;
        transaction.commit().await?;
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
        run_id: RunId,
    ) -> Result<Option<PersistedAgentDefinition>> {
        // Keep both definition joins indexable. The second arm only serves legacy runs created
        // before runs.agent_def_id was added; new bot runs always use the first arm.
        query_as::<_, AgentDefinitionRow>(
            "SELECT d.id, d.name, d.version, d.frontmatter, d.body
             FROM agents.runs r
             JOIN agents.sessions s ON s.id = r.session_id
             JOIN agents.agent_defs d ON d.id = r.agent_def_id
             WHERE r.id = $1 AND r.agent_def_id IS NOT NULL
             UNION ALL
             SELECT d.id, d.name, d.version, d.frontmatter, d.body
             FROM agents.runs r
             JOIN agents.sessions s ON s.id = r.session_id
             JOIN agents.agent_defs d ON d.id = s.agent_def_id
             WHERE r.id = $1 AND r.agent_def_id IS NULL
             LIMIT 1",
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
