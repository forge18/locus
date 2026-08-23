//! Persistence for project settings and their analytics rollup (`core.settings`).
//!
//! Moved out of `services/project.rs` so every query in the crate lives under `store/`.

use anyhow::{Context, Result};
use sqlx::{query, Row};
use uuid::Uuid;

use crate::{services::project::ProjectSettings, store::Store};

const SETTINGS_KEY: &str = "project_settings";

impl Store {
    /// Replace one project's typed settings aggregate.
    pub async fn set_project_settings(
        &self,
        project_id: Uuid,
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
    pub async fn project_settings(&self, project_id: Uuid) -> Result<ProjectSettings> {
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
}
