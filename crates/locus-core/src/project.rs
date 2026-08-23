//! Durable project configuration.
//!
//! Project policy is one typed, versioned aggregate stored through `core.settings` rather than a
//! collection of uncoordinated keys. Individual fields are added by the project-operations tasks.

use std::collections::BTreeSet;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{query, Row};
use uuid::Uuid;

use crate::store::Store;

const SETTINGS_KEY: &str = "project_settings";
const SETTINGS_VERSION: u16 = 1;

/// A repository belongs to one project through `core.repos.project_id`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectRepo {
    pub name: String,
    pub working_copy_path: String,
}

impl ProjectRepo {
    pub fn new(name: impl Into<String>, working_copy_path: impl Into<String>) -> Result<Self> {
        let repo = Self {
            name: name.into(),
            working_copy_path: working_copy_path.into(),
        };
        if repo.name.trim().is_empty() || repo.working_copy_path.trim().is_empty() {
            bail!("project repo name and working-copy path must not be empty");
        }
        Ok(repo)
    }
}

/// The persisted root for all project-local policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectSettings {
    pub version: u16,
    #[serde(default)]
    harness_allow_list: BTreeSet<String>,
    #[serde(default)]
    agent_default: Option<String>,
    #[serde(default)]
    base_context: Option<String>,
    #[serde(default)]
    base_context_token_budget: Option<u32>,
}

impl ProjectSettings {
    pub fn new() -> Self {
        Self {
            version: SETTINGS_VERSION,
            harness_allow_list: BTreeSet::new(),
            agent_default: None,
            base_context: None,
            base_context_token_budget: None,
        }
    }

    /// Replace the project-local set of harnesses that may run work.
    pub fn with_harness_allow_list<I, S>(mut self, harnesses: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.harness_allow_list = harnesses.into_iter().map(Into::into).collect();
        self.validate()?;
        Ok(self)
    }

    pub fn permits_harness(&self, harness: &str) -> bool {
        self.harness_allow_list.contains(harness)
    }

    /// Choose the one harness used when routing does not claim an enabled harness.
    pub fn with_agent_default(mut self, harness: impl Into<String>) -> Result<Self> {
        self.agent_default = Some(harness.into());
        self.validate()?;
        Ok(self)
    }

    pub fn agent_default(&self) -> Option<&str> {
        self.agent_default.as_deref()
    }

    /// Store the one base context that every project run receives with its token budget.
    pub fn with_base_context(mut self, content: impl Into<String>, token_budget: u32) -> Result<Self> {
        self.base_context = Some(content.into());
        self.base_context_token_budget = Some(token_budget);
        self.validate()?;
        Ok(self)
    }

    pub fn base_context(&self) -> Option<&str> {
        self.base_context.as_deref()
    }

    pub fn base_context_token_budget(&self) -> Option<u32> {
        self.base_context_token_budget
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
        if self.harness_allow_list.iter().any(|harness| harness.trim().is_empty()) {
            bail!("project harness allow-list cannot contain an empty harness");
        }
        if self
            .agent_default
            .as_deref()
            .is_some_and(|harness| !self.permits_harness(harness))
        {
            bail!("project agent default must be in the harness allow-list");
        }
        if self.base_context.as_deref().is_some_and(|context| context.trim().is_empty())
            || self.base_context_token_budget == Some(0)
            || self.base_context.is_some() != self.base_context_token_budget.is_some()
        {
            bail!("base context and a nonzero token budget must be set together");
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
