//! Durable project configuration.
//!
//! Project policy is one typed, versioned aggregate stored through `core.settings` rather than a
//! collection of uncoordinated keys. Individual fields are added by the project-operations tasks.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    harness::materialize::extensions::ProjectExtensionScope, services::tools::ProjectToolScope,
};

const SETTINGS_VERSION: u16 = 1;

/// Token and spend facts emitted by one project run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectRunAnalytics {
    pub model: String,
    pub tokens: u64,
    pub cache_read_tokens: u64,
    pub spend_micros: u64,
}

impl ProjectRunAnalytics {
    pub fn new(
        model: impl Into<String>,
        tokens: u64,
        cache_read_tokens: u64,
        spend_micros: u64,
    ) -> Self {
        Self {
            model: model.into(),
            tokens,
            cache_read_tokens,
            spend_micros,
        }
    }
}

/// A model row aggregated from project-scoped run data.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectModelAnalytics {
    pub tokens: u64,
    pub cache_read_tokens: u64,
    pub spend_micros: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectAnalytics {
    models: BTreeMap<String, ProjectModelAnalytics>,
}

impl ProjectAnalytics {
    pub fn from_runs(runs: impl IntoIterator<Item = ProjectRunAnalytics>) -> Self {
        let mut analytics = Self::default();
        for run in runs {
            let model = analytics.models.entry(run.model).or_default();
            model.tokens += run.tokens;
            model.cache_read_tokens += run.cache_read_tokens;
            model.spend_micros += run.spend_micros;
        }
        analytics
    }

    pub fn model(&self, model: &str) -> Option<&ProjectModelAnalytics> {
        self.models.get(model)
    }
}

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

/// A project lifecycle changes the active project name/state without owning historical rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectLifecycle {
    name: String,
    archived: bool,
}

impl ProjectLifecycle {
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if name.trim().is_empty() {
            bail!("project name must not be empty");
        }
        Ok(Self {
            name,
            archived: false,
        })
    }

    pub fn rename(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn archive(mut self) -> Self {
        self.archived = true;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn is_archived(&self) -> bool {
        self.archived
    }
    pub const fn preserves_history(&self) -> bool {
        true
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
    #[serde(default)]
    extension_overrides: ProjectExtensionScope,
    #[serde(default)]
    tool_scope: ProjectToolScope,
}

impl ProjectSettings {
    pub fn new() -> Self {
        Self {
            version: SETTINGS_VERSION,
            harness_allow_list: BTreeSet::new(),
            agent_default: None,
            base_context: None,
            base_context_token_budget: None,
            extension_overrides: ProjectExtensionScope::default(),
            tool_scope: ProjectToolScope::default(),
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
    pub fn with_base_context(
        mut self,
        content: impl Into<String>,
        token_budget: u32,
    ) -> Result<Self> {
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

    pub fn with_extension_overrides(mut self, overrides: ProjectExtensionScope) -> Self {
        self.extension_overrides = overrides;
        self
    }

    pub fn extension_overrides(&self) -> &ProjectExtensionScope {
        &self.extension_overrides
    }

    pub fn with_tool_scope(mut self, scope: ProjectToolScope) -> Self {
        self.tool_scope = scope;
        self
    }

    pub fn tool_scope(&self) -> &ProjectToolScope {
        &self.tool_scope
    }

    pub fn to_stored_value(&self) -> Result<Value> {
        self.validate()?;
        serde_json::to_value(self).context("serialize project settings")
    }

    pub fn from_stored_value(value: Value) -> Result<Self> {
        let settings: Self =
            serde_json::from_value(value).context("deserialize project settings")?;
        settings.validate()?;
        Ok(settings)
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.version != SETTINGS_VERSION {
            bail!("unsupported project settings version `{}`", self.version);
        }
        if self
            .harness_allow_list
            .iter()
            .any(|harness| harness.trim().is_empty())
        {
            bail!("project harness allow-list cannot contain an empty harness");
        }
        if self
            .agent_default
            .as_deref()
            .is_some_and(|harness| !self.permits_harness(harness))
        {
            bail!("project agent default must be in the harness allow-list");
        }
        if self
            .base_context
            .as_deref()
            .is_some_and(|context| context.trim().is_empty())
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
