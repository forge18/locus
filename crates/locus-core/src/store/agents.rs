//! Persistence for agent definitions (`agents.agent_defs`).
//!
//! Moved out of `services/agents.rs` so every query in the crate lives under `store/`.

use crate::ids::{ProjectId, RunId};
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

/// One running run with its project and agent — the shell's dispatch pill and
/// session popover. A project id scopes the read; `None` is the cross-project
/// shell view.
#[derive(Debug, sqlx::FromRow)]
pub struct RunningRunRow {
    pub id: Uuid,
    pub project: String,
    pub agent: String,
    pub status: String,
    pub started_epoch: i64,
}

/// One row of the Dispatch runs table: every run, newest first. Event and
/// error counts roll up from `agents.events`.
#[derive(Debug, sqlx::FromRow)]
pub struct DispatchRunRow {
    pub id: Uuid,
    pub project: String,
    pub agent: String,
    pub branch: String,
    pub status: String,
    pub harness: Option<String>,
    pub role: Option<String>,
    pub model: String,
    pub events: i64,
    pub errors: i64,
    pub started_at: Option<String>,
}

/// One row of `agents.sessions` with its project and agent names resolved —
/// the session list's wire shape.
#[derive(Debug, sqlx::FromRow)]
pub struct SessionRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub project: String,
    pub agent: String,
    pub name: String,
    pub branch: String,
    pub status: String,
    pub created_at: Option<String>,
}

/// One session's runs, oldest first — the session detail's run list.
#[derive(Debug, sqlx::FromRow)]
pub struct SessionRunRow {
    pub id: Uuid,
    pub session_id: Uuid,
    pub status: String,
    pub resolved_model: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub exit_code: Option<i32>,
}

/// Counts used by the first live Analytics and Telemetry projections.
#[derive(Debug, sqlx::FromRow)]
pub struct ActivityCountsRow {
    pub sessions: i64,
    pub runs: i64,
    pub events: i64,
    pub errors: i64,
}

impl Store {
    /// The latest immutable version of each named agent definition.
    pub async fn agent_definitions(&self) -> Result<Vec<PersistedAgentDefinition>> {
        query_as::<_, AgentDefinitionRow>(
            "SELECT DISTINCT ON (name) id, name, version, frontmatter, body
             FROM agents.agent_defs
             ORDER BY name, version DESC",
        )
        .fetch_all(self.pool())
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .context("list agent definitions")
    }

    pub async fn latest_agent_definition(
        &self,
        name: &str,
    ) -> Result<Option<PersistedAgentDefinition>> {
        query_as::<_, AgentDefinitionRow>(
            "SELECT id, name, version, frontmatter, body
             FROM agents.agent_defs
             WHERE name = $1
             ORDER BY version DESC
             LIMIT 1",
        )
        .bind(name)
        .fetch_optional(self.pool())
        .await
        .map(|row| row.map(Into::into))
        .context("read latest agent definition")
    }

    pub async fn running_runs(&self, project_id: Option<ProjectId>) -> Result<Vec<RunningRunRow>> {
        query_as(
            "SELECT r.id, p.name AS project, ad.name AS agent, r.status,
                    EXTRACT(EPOCH FROM COALESCE(r.started_at, r.created_at))::bigint AS started_epoch
             FROM agents.runs r
             JOIN agents.sessions s ON s.id = r.session_id
             JOIN core.projects p ON p.id = s.project_id
             JOIN agents.agent_defs ad ON ad.id = s.agent_def_id
             WHERE r.status = 'running' AND ($1::uuid IS NULL OR s.project_id = $1)
             ORDER BY COALESCE(r.started_at, r.created_at) DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .context("list running runs")
    }

    /// Every session across projects, newest first — the run slice's session list.
    /// One session by id, with its project and agent names resolved.
    pub async fn session(&self, session_id: Uuid) -> Result<Option<SessionRow>> {
        query_as(
            "SELECT s.id, s.project_id, p.name AS project, ad.name AS agent,
                    s.name, s.branch, s.status, s.created_at::text AS created_at
             FROM agents.sessions s
             JOIN core.projects p ON p.id = s.project_id
             JOIN agents.agent_defs ad ON ad.id = s.agent_def_id
             WHERE s.id = $1",
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .context("read one session")
    }

    pub async fn sessions_page(
        &self,
        project_id: Option<ProjectId>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<SessionRow>> {
        query_as(
            "SELECT s.id, s.project_id, p.name AS project, ad.name AS agent,
                    s.name, s.branch, s.status, s.created_at::text AS created_at
             FROM agents.sessions s
             JOIN core.projects p ON p.id = s.project_id
             JOIN agents.agent_defs ad ON ad.id = s.agent_def_id
             WHERE ($1::uuid IS NULL OR s.project_id = $1)
             ORDER BY s.created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(project_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .context("page sessions")
    }

    pub async fn sessions_count(&self, project_id: Option<ProjectId>) -> Result<i64> {
        query_scalar(
            "SELECT COUNT(*)
             FROM agents.sessions s
             WHERE ($1::uuid IS NULL OR s.project_id = $1)",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
        .context("count sessions")
    }

    pub async fn activity_counts(
        &self,
        project_id: Option<ProjectId>,
        since_epoch: Option<i64>,
    ) -> Result<ActivityCountsRow> {
        query_as(
            "SELECT COUNT(DISTINCT s.id) FILTER (
                        WHERE $2::bigint IS NULL
                           OR COALESCE(r.started_at, r.created_at) >= to_timestamp($2)
                    ) AS sessions,
                    COUNT(DISTINCT r.id) FILTER (
                        WHERE $2::bigint IS NULL
                           OR COALESCE(r.started_at, r.created_at) >= to_timestamp($2)
                    ) AS runs,
                    COUNT(e.id) FILTER (
                        WHERE $2::bigint IS NULL OR e.ts >= to_timestamp($2)
                    ) AS events,
                    COUNT(e.id) FILTER (
                        WHERE e.verb = 'tool_error'
                          AND ($2::bigint IS NULL OR e.ts >= to_timestamp($2))
                    ) AS errors
             FROM agents.sessions s
             LEFT JOIN agents.runs r ON r.session_id = s.id
             LEFT JOIN agents.events e ON e.run_id = r.id
             WHERE ($1::uuid IS NULL OR s.project_id = $1)",
        )
        .bind(project_id)
        .bind(since_epoch)
        .fetch_one(self.pool())
        .await
        .context("count project activity")
    }

    pub async fn runs_for_session(&self, session_id: Uuid) -> Result<Vec<SessionRunRow>> {
        query_as(
            "SELECT id, session_id, status, resolved_model_id AS resolved_model,
                    started_at::text AS started_at, ended_at::text AS ended_at, exit_code
             FROM agents.runs
             WHERE session_id = $1
             ORDER BY COALESCE(started_at, created_at)",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await
        .context("list session runs")
    }

    /// One row of the Dispatch runs table: every run, newest first. Event and
    /// error counts roll up from `agents.events`.
    pub async fn dispatch_runs_page(
        &self,
        project_id: Option<ProjectId>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<DispatchRunRow>> {
        query_as(
            "SELECT r.id, p.name AS project, ad.name AS agent, s.branch, r.status,
                    ad.frontmatter ->> 'harness' AS harness,
                    ad.frontmatter ->> 'role' AS role,
                    r.resolved_model_id AS model,
                    COALESCE(ev.events, 0) AS events,
                    COALESCE(ev.errors, 0) AS errors,
                    COALESCE(r.started_at, r.created_at)::text AS started_at
             FROM agents.runs r
             JOIN agents.sessions s ON s.id = r.session_id
             JOIN core.projects p ON p.id = s.project_id
             JOIN agents.agent_defs ad ON ad.id = s.agent_def_id
             LEFT JOIN (
                 SELECT run_id, COUNT(*) AS events,
                        COUNT(*) FILTER (WHERE verb = 'tool_error') AS errors
                 FROM agents.events GROUP BY run_id
             ) ev ON ev.run_id = r.id
             WHERE ($1::uuid IS NULL OR s.project_id = $1)
             ORDER BY COALESCE(r.started_at, r.created_at) DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(project_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .context("page dispatch runs")
    }

    pub async fn dispatch_runs_count(&self, project_id: Option<ProjectId>) -> Result<i64> {
        query_scalar(
            "SELECT COUNT(*)
             FROM agents.runs r
             JOIN agents.sessions s ON s.id = r.session_id
             WHERE ($1::uuid IS NULL OR s.project_id = $1)",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
        .context("count dispatch runs")
    }

    /// Agents only — the host shell itself never runs an agent.
    pub async fn running_run_count(&self, project_id: Option<ProjectId>) -> Result<i64> {
        query_scalar(
            "SELECT COUNT(*)
             FROM agents.runs r
             JOIN agents.sessions s ON s.id = r.session_id
             WHERE r.status = 'running' AND ($1::uuid IS NULL OR s.project_id = $1)",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
        .context("count running runs")
    }

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
