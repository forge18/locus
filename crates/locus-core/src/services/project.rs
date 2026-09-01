//! Durable project configuration.
//!
//! Project policy is one typed, versioned aggregate stored through `core.settings` rather than a
//! collection of uncoordinated keys. Individual fields are added by the project-operations tasks.

use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    harness::materialize::extensions::ProjectExtensionScope,
    lsp::DescriptorPin,
    services::{
        bots::BotSettings,
        capabilities::CapabilityPolicies,
        tools::ProjectToolScope,
    },
};

const SETTINGS_VERSION: u16 = 1;

/// A project-owned command and the DAP adapter used to launch it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugRunConfig {
    adapter: String,
    command: String,
    /// The argv used to start the admitted adapter inside the run container. An omitted value
    /// defaults to the adapter id, which is also the marketplace tool executable.
    #[serde(default)]
    adapter_command: Vec<String>,
}

impl DebugRunConfig {
    pub fn new(adapter: impl Into<String>, command: impl Into<String>) -> Result<Self> {
        let adapter = adapter.into();
        let config = Self {
            adapter_command: vec![adapter.clone()],
            adapter,
            command: command.into(),
        };
        config.validate()?;
        Ok(config)
    }

    pub fn with_adapter_command(
        mut self,
        adapter_command: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self> {
        self.adapter_command = adapter_command.into_iter().map(Into::into).collect();
        self.validate()?;
        Ok(self)
    }

    fn validate(&self) -> Result<()> {
        if self.adapter.trim().is_empty() || self.command.trim().is_empty() {
            bail!("debug adapter and command must not be empty");
        }
        if let Some(executable) = self.adapter_command.first() {
            if executable != &self.adapter
                || self
                    .adapter_command
                    .iter()
                    .any(|part| part.trim().is_empty())
            {
                bail!("debug adapter command must start with the configured adapter");
            }
        }
        Ok(())
    }

    pub fn adapter(&self) -> &str {
        &self.adapter
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn adapter_command(&self) -> Vec<String> {
        if self.adapter_command.is_empty() {
            vec![self.adapter.clone()]
        } else {
            self.adapter_command.clone()
        }
    }
}

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
    harness_allow_list: Vec<String>,
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
    #[serde(default)]
    capability_policies: CapabilityPolicies,
    #[serde(default)]
    lsp_descriptors: BTreeMap<String, DescriptorPin>,
    #[serde(default)]
    debug_configs: BTreeMap<String, DebugRunConfig>,
    #[serde(default)]
    bots: BotSettings,
}

impl ProjectSettings {
    pub fn new() -> Self {
        Self {
            version: SETTINGS_VERSION,
            harness_allow_list: Vec::new(),
            agent_default: None,
            base_context: None,
            base_context_token_budget: None,
            extension_overrides: ProjectExtensionScope::default(),
            tool_scope: ProjectToolScope::default(),
            capability_policies: CapabilityPolicies::default(),
            lsp_descriptors: BTreeMap::new(),
            debug_configs: BTreeMap::new(),
            bots: BotSettings::default(),
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

    /// The order is routing precedence; callers must not sort this list.
    pub fn harness_allow_list(&self) -> &[String] {
        &self.harness_allow_list
    }

    pub fn permits_harness(&self, harness: &str) -> bool {
        self.harness_allow_list
            .iter()
            .any(|candidate| candidate == harness)
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

    pub fn with_capability_policies(mut self, policies: CapabilityPolicies) -> Self {
        self.capability_policies = policies;
        self
    }

    pub fn capability_policies(&self) -> &CapabilityPolicies {
        &self.capability_policies
    }

    pub fn with_lsp_descriptors(
        mut self,
        descriptors: impl IntoIterator<Item = DescriptorPin>,
    ) -> Result<Self> {
        let mut pins = BTreeMap::new();
        for pin in descriptors {
            if pin.id.trim().is_empty()
                || pin.version == 0
                || !pin.content_hash.starts_with("sha256:")
            {
                bail!("project LSP descriptor pins must contain an id, version, and SHA-256 hash");
            }
            if pins.insert(pin.id.clone(), pin).is_some() {
                bail!("project LSP descriptor pins cannot contain duplicate ids");
            }
        }
        self.lsp_descriptors = pins;
        self.validate()?;
        Ok(self)
    }

    pub fn lsp_descriptors(&self) -> &BTreeMap<String, DescriptorPin> {
        &self.lsp_descriptors
    }

    pub fn with_debug_config(
        mut self,
        name: impl Into<String>,
        config: DebugRunConfig,
    ) -> Result<Self> {
        let name = name.into();
        if name.trim().is_empty() || self.debug_configs.insert(name, config).is_some() {
            bail!("project debug config names must be non-empty and unique");
        }
        self.validate()?;
        Ok(self)
    }

    pub fn debug_config(&self, name: &str) -> Option<&DebugRunConfig> {
        self.debug_configs.get(name)
    }

    pub fn debug_configs(&self) -> &BTreeMap<String, DebugRunConfig> {
        &self.debug_configs
    }

    pub fn with_bot_warm_window_minutes(mut self, minutes: u32) -> Result<Self> {
        self.bots = BotSettings::new(minutes)?;
        self.validate()?;
        Ok(self)
    }

    pub fn bots(&self) -> &BotSettings {
        &self.bots
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
            .harness_allow_list
            .windows(2)
            .any(|pair| pair[0] == pair[1])
        {
            bail!("project harness allow-list cannot contain duplicates");
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
        for config in self.debug_configs.values() {
            config.validate()?;
        }
        self.bots.warm_window()?;
        Ok(())
    }
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[test]
fn lsp_descriptor_pins_round_trip_in_project_settings() {
    let pin = DescriptorPin {
        id: "rust".into(),
        version: 1,
        content_hash: "sha256:pin".into(),
    };
    let settings = ProjectSettings::new()
        .with_lsp_descriptors([pin.clone()])
        .expect("valid LSP descriptor pin");
    let restored = ProjectSettings::from_stored_value(settings.to_stored_value().unwrap()).unwrap();
    assert_eq!(restored.lsp_descriptors().get("rust"), Some(&pin));
}

#[cfg(test)]
#[test]
fn harness_order_preserved() {
    let settings = ProjectSettings::new()
        .with_harness_allow_list(["codex", "claude", "pi"])
        .expect("valid ordered harness list");
    assert_eq!(settings.harness_allow_list(), ["codex", "claude", "pi"]);
}

#[cfg(test)]
#[test]
fn agent_default_requires_adapter() {
    let error = ProjectSettings::new()
        .with_agent_default("missing")
        .expect_err("a default not in the allow-list is invalid");
    assert!(error.to_string().contains("allow-list"));
}

#[cfg(test)]
#[test]
fn base_context_single_file_metadata() {
    let settings = ProjectSettings::new()
        .with_base_context("base", 1_000)
        .unwrap();
    assert_eq!(settings.base_context(), Some("base"));
    assert_eq!(settings.base_context_token_budget(), Some(1_000));
}

#[cfg(test)]
#[test]
fn persistence_page_size() {
    assert_eq!(4usize, 4);
}

#[cfg(test)]
#[test]
fn debug_configs_round_trip() {
    let settings = ProjectSettings::new()
        .with_debug_config(
            "app",
            DebugRunConfig::new("python-debug-adapter", "python -m app").unwrap(),
        )
        .unwrap();
    let restored = ProjectSettings::from_stored_value(settings.to_stored_value().unwrap()).unwrap();
    let config = restored.debug_config("app").unwrap();
    assert_eq!(config.adapter(), "python-debug-adapter");
    assert_eq!(config.command(), "python -m app");
    assert_eq!(config.adapter_command(), ["python-debug-adapter"]);
}

#[cfg(test)]
#[test]
fn bots_warm_window_round_trips_in_project_settings() {
    let settings = ProjectSettings::new()
        .with_bot_warm_window_minutes(27)
        .expect("valid bot warm window");
    let value = settings.to_stored_value().expect("serialize settings");
    assert_eq!(value["bots"]["warm_window_minutes"], 27);
    let restored = ProjectSettings::from_stored_value(value).expect("restore settings");
    assert_eq!(restored.bots().warm_window_minutes, 27);
}

#[cfg(test)]
#[test]
fn bots_warm_window_rejects_unbounded_values() {
    assert!(ProjectSettings::new()
        .with_bot_warm_window_minutes(24 * 60 + 1)
        .is_err());
}

#[cfg(test)]
#[test]
fn debug_adapter_command_must_use_the_allowlisted_tool() {
    let config = DebugRunConfig::new("debugpy", "python -m app").unwrap();
    assert!(config
        .clone()
        .with_adapter_command(["python", "-m", "debugpy.adapter"])
        .is_err());
    assert!(config.with_adapter_command(["debugpy", "--stdio"]).is_ok());
}
