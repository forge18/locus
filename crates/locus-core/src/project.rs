//! Durable project configuration.
//!
//! Project policy is one typed, versioned aggregate stored through `core.settings` rather than a
//! collection of uncoordinated keys. Individual fields are added by the project-operations tasks.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{query, Row};
use uuid::Uuid;

use crate::store::Store;

const SETTINGS_KEY: &str = "project_settings";
const SETTINGS_VERSION: u16 = 1;

/// The persisted root for all project-local policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectSettings {
    pub version: u16,
}

impl ProjectSettings {
    pub fn new() -> Self {
        Self {
            version: SETTINGS_VERSION,
        }
    }

    pub fn to_stored_value(&self) -> Result<Value> {
        self.validate()?;
        serde_json::to_value(self).context("serialize project settings")
    }

    pub fn from_stored_value(value: Value) -> Result<Self> {
        let settings: Self = serde_json::from_value(value).context("deserialize project settings")?;
        settings.validate()?;
        Ok(settings)
    }

    fn validate(&self) -> Result<()> {
        if self.version != SETTINGS_VERSION {
            bail!("unsupported project settings version `{}`", self.version);
        }
        Ok(())
    }
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self::new()
    }
}

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
