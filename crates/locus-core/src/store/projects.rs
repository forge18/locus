//! Persistence for project settings and their analytics rollup (`core.settings`).
//!
//! Moved out of `services/project.rs` so every query in the crate lives under `store/`.

use crate::ids::ProjectId;
use std::collections::BTreeMap;

use anyhow::{Context, Result};
use sqlx::{query, Row};
use uuid::Uuid;

use crate::{lsp::DescriptorPin, services::project::ProjectSettings, store::Store};

const SETTINGS_KEY: &str = "project_settings";

impl Store {
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
