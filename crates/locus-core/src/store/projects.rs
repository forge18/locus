//! Persistence for projects, their repos and local remotes, project settings, and
//! the project analytics rollup (`core.projects`, `core.repos`, `core.local_remotes`,
//! `core.settings`).
//!
//! Moved out of `services/project.rs` so every query in the crate lives under `store/`.

use crate::ids::ProjectId;
use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use sqlx::{query, query_scalar, Row};
use uuid::Uuid;

use crate::{lsp::DescriptorPin, services::project::ProjectSettings, store::Store};

const SETTINGS_KEY: &str = "project_settings";

/// One row of `core.projects`.
#[derive(Debug, sqlx::FromRow)]
pub struct ProjectRow {
    pub id: ProjectId,
    pub name: String,
}

/// One row of `core.repos`.
#[derive(Debug, sqlx::FromRow)]
pub struct RepoRow {
    pub id: Uuid,
    pub project_id: ProjectId,
    pub name: String,
    pub working_copy_path: String,
}

/// One row of `core.local_remotes`, scoped to a project through its repo.
#[derive(Debug, sqlx::FromRow)]
pub struct LocalRemoteRow {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub bare_path: String,
}

impl Store {
    /// Every project, alphabetically — the Setup screen's project list.
    pub async fn projects_list(&self) -> Result<Vec<ProjectRow>> {
        sqlx::query_as::<_, ProjectRow>("SELECT id, name FROM core.projects ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .context("list projects")
    }

    /// Every repo of exactly one project, alphabetically. The WHERE clause is the
    /// ownership boundary: a repo of another project can never appear here.
    pub async fn repos_list(&self, project_id: ProjectId) -> Result<Vec<RepoRow>> {
        sqlx::query_as::<_, RepoRow>(
            "SELECT id, project_id, name, working_copy_path
             FROM core.repos
             WHERE project_id = $1
             ORDER BY name",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .context("list project repos")
    }

    /// The bare remotes of one project's repos, joined through `core.repos`.
    pub async fn local_remotes_list(&self, project_id: ProjectId) -> Result<Vec<LocalRemoteRow>> {
        sqlx::query_as::<_, LocalRemoteRow>(
            "SELECT lr.id, lr.repo_id, lr.bare_path
             FROM core.local_remotes lr
             JOIN core.repos r ON r.id = lr.repo_id
             WHERE r.project_id = $1
             ORDER BY lr.bare_path",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .context("list project local remotes")
    }

    pub async fn resolve_project_id(&self, identifier: &str) -> Result<Option<ProjectId>> {
        if let Ok(project_id) = identifier.parse::<ProjectId>() {
            return query_scalar::<_, ProjectId>("SELECT id FROM core.projects WHERE id = $1")
                .bind(project_id)
                .fetch_optional(self.pool())
                .await
                .context("resolve project UUID");
        }
        if identifier.trim().is_empty() {
            return Ok(None);
        }
        let project_ids = query_scalar::<_, ProjectId>(
            "SELECT id
             FROM core.projects
             WHERE name = $1
             ORDER BY created_at, id
             LIMIT 2",
        )
        .bind(identifier)
        .fetch_all(self.pool())
        .await
        .context("resolve project name")?;
        match project_ids.as_slice() {
            [] => Ok(None),
            [project_id] => Ok(Some(*project_id)),
            _ => bail!("project name `{identifier}` is ambiguous"),
        }
    }

    /// Replace one project's typed settings aggregate.
    pub async fn set_project_settings(
        &self,
        project_id: ProjectId,
        settings: &ProjectSettings,
    ) -> Result<()> {
        let value = settings.to_stored_value()?;
        query(
            "INSERT INTO core.settings (project_id, key, value)
             VALUES ($1, $2, $3)
             ON CONFLICT (project_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = now()",
        )
        .bind(project_id)
        .bind(SETTINGS_KEY)
        .bind(value)
        .execute(self.pool())
        .await
        .context("persist project settings")?;
        Ok(())
    }

    /// Read one project's typed settings aggregate, creating no row for untouched projects.
    pub async fn project_settings(&self, project_id: ProjectId) -> Result<ProjectSettings> {
        let row = query("SELECT value FROM core.settings WHERE project_id = $1 AND key = $2")
            .bind(project_id)
            .bind(SETTINGS_KEY)
            .fetch_optional(self.pool())
            .await
            .context("read project settings")?;
        match row {
            Some(row) => ProjectSettings::from_stored_value(row.try_get("value")?),
            None => Ok(ProjectSettings::default()),
        }
    }

    pub async fn set_project_lsp_descriptors(
        &self,
        project_id: ProjectId,
        descriptors: impl IntoIterator<Item = DescriptorPin>,
    ) -> Result<()> {
        let settings = self
            .project_settings(project_id)
            .await?
            .with_lsp_descriptors(descriptors)?;
        self.set_project_settings(project_id, &settings).await
    }

    pub async fn project_lsp_descriptors(
        &self,
        project_id: ProjectId,
    ) -> Result<BTreeMap<String, DescriptorPin>> {
        Ok(self
            .project_settings(project_id)
            .await?
            .lsp_descriptors()
            .clone())
    }

    /// Renames a project. An empty name is rejected here so the CHECK constraint
    /// never surfaces as a raw database error.
    pub async fn rename_project(&self, project_id: ProjectId, name: &str) -> Result<()> {
        if name.trim().is_empty() {
            bail!("project name must not be empty");
        }
        let result = query("UPDATE core.projects SET name = $2, updated_at = now() WHERE id = $1")
            .bind(project_id)
            .bind(name)
            .execute(&self.pool)
            .await
            .context("rename project")?;
        if result.rows_affected() == 0 {
            bail!("project was not found");
        }
        Ok(())
    }

    pub async fn set_project_archived(&self, project_id: ProjectId, archived: bool) -> Result<()> {
        query("UPDATE core.projects SET archived_at = CASE WHEN $2 THEN COALESCE(archived_at, now()) ELSE NULL END, updated_at = now() WHERE id = $1")
            .bind(project_id).bind(archived).execute(self.pool()).await.context("set project archive state")?;
        Ok(())
    }

    pub async fn project_archived(&self, project_id: ProjectId) -> Result<bool> {
        query("SELECT archived_at IS NOT NULL AS archived FROM core.projects WHERE id = $1")
            .bind(project_id)
            .fetch_one(self.pool())
            .await
            .context("read project archive state")
            .and_then(|row| {
                row.try_get("archived")
                    .context("decode project archive state")
            })
    }

    /// Move a repo while retaining an append-only historical project tag.
    pub async fn reassign_repo(&self, repo_id: Uuid, project_id: ProjectId) -> Result<()> {
        let mut tx = self
            .pool()
            .begin()
            .await
            .context("begin repo reassignment")?;
        let old: ProjectId = query("SELECT project_id FROM core.repos WHERE id = $1 FOR UPDATE")
            .bind(repo_id)
            .fetch_one(&mut *tx)
            .await
            .context("read repo project")?
            .try_get("project_id")?;
        if old == project_id {
            return Ok(());
        }
        query(
            "INSERT INTO core.repo_project_history (id, repo_id, project_id) VALUES ($1, $2, $3)",
        )
        .bind(Uuid::new_v4())
        .bind(repo_id)
        .bind(old)
        .execute(&mut *tx)
        .await
        .context("record old repo project")?;
        query("UPDATE core.repos SET project_id = $2, updated_at = now() WHERE id = $1")
            .bind(repo_id)
            .bind(project_id)
            .execute(&mut *tx)
            .await
            .context("reassign repo")?;
        tx.commit().await.context("commit repo reassignment")?;
        Ok(())
    }
}
