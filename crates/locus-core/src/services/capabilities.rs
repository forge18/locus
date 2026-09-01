//! Non-escalating capability policy resolution for agent runs.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum CapabilityPolicy {
    #[default]
    DeferToProject,
    AllowOnly(BTreeSet<String>),
}

impl CapabilityPolicy {
    pub fn allow_only<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::AllowOnly(values.into_iter().map(Into::into).collect())
    }

    fn apply(&self, baseline: &BTreeSet<String>) -> BTreeSet<String> {
        match self {
            Self::DeferToProject => baseline.clone(),
            Self::AllowOnly(allowed) => baseline.intersection(allowed).cloned().collect(),
        }
    }

    pub fn is_deferred(&self) -> bool {
        matches!(self, Self::DeferToProject)
    }

    pub fn allowed(&self) -> Option<&BTreeSet<String>> {
        match self {
            Self::DeferToProject => None,
            Self::AllowOnly(allowed) => Some(allowed),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityPolicies {
    #[serde(default)]
    pub cli_tools: CapabilityPolicy,
    #[serde(default)]
    pub commands: CapabilityPolicy,
    #[serde(default)]
    pub skills: CapabilityPolicy,
}

impl CapabilityPolicies {
    pub fn restrict_cli_tools(mut self, policy: CapabilityPolicy) -> Self {
        self.cli_tools = policy;
        self
    }

    pub fn restrict_commands(mut self, policy: CapabilityPolicy) -> Self {
        self.commands = policy;
        self
    }

    pub fn restrict_skills(mut self, policy: CapabilityPolicy) -> Self {
        self.skills = policy;
        self
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityCatalog {
    pub cli_tools: BTreeSet<String>,
    pub commands: BTreeSet<String>,
    pub skills: BTreeSet<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveCapabilities {
    pub cli_tools: BTreeSet<String>,
    pub commands: BTreeSet<String>,
    pub skills: BTreeSet<String>,
}

fn resolve_one(
    catalog: &BTreeSet<String>,
    project: &CapabilityPolicy,
    agent: &CapabilityPolicy,
    workflow: &CapabilityPolicy,
) -> BTreeSet<String> {
    workflow.apply(&agent.apply(&project.apply(catalog)))
}

/// Resolve the catalog through project, agent, and workflow restrictions.
/// `None` means the narrower layer has no restriction and must inherit.
pub fn resolve_capabilities(
    catalog: &CapabilityCatalog,
    project: &CapabilityPolicies,
    agent: Option<&CapabilityPolicies>,
    workflow: Option<&CapabilityPolicies>,
) -> EffectiveCapabilities {
    let agent = agent.cloned().unwrap_or_default();
    let workflow = workflow.cloned().unwrap_or_default();
    EffectiveCapabilities {
        cli_tools: resolve_one(
            &catalog.cli_tools,
            &project.cli_tools,
            &agent.cli_tools,
            &workflow.cli_tools,
        ),
        commands: resolve_one(
            &catalog.commands,
            &project.commands,
            &agent.commands,
            &workflow.commands,
        ),
        skills: resolve_one(
            &catalog.skills,
            &project.skills,
            &agent.skills,
            &workflow.skills,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> CapabilityCatalog {
        CapabilityCatalog {
            cli_tools: ["git", "rg"].into_iter().map(String::from).collect(),
            commands: ["verify", "review"].into_iter().map(String::from).collect(),
            skills: ["rust", "security"].into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn policies_only_narrow_the_project_catalog() {
        let project = CapabilityPolicies::default()
            .restrict_cli_tools(CapabilityPolicy::allow_only(["git", "rg"]))
            .restrict_commands(CapabilityPolicy::allow_only(["verify"]))
            .restrict_skills(CapabilityPolicy::allow_only(["rust", "security"]));
        let agent = CapabilityPolicies::default()
            .restrict_cli_tools(CapabilityPolicy::allow_only(["rg"]))
            .restrict_skills(CapabilityPolicy::allow_only(["security"]));
        let workflow = CapabilityPolicies::default()
            .restrict_cli_tools(CapabilityPolicy::allow_only(["git", "rg", "curl"]))
            .restrict_commands(CapabilityPolicy::allow_only(std::iter::empty::<&str>()));
        let effective = resolve_capabilities(&catalog(), &project, Some(&agent), Some(&workflow));
        assert_eq!(effective.cli_tools, BTreeSet::from([String::from("rg")]));
        assert!(effective.commands.is_empty());
        assert_eq!(effective.skills, BTreeSet::from([String::from("security")]));
    }

    #[test]
    fn absent_restrictions_inherit_and_empty_allow_only_is_not_inherit() {
        let project = CapabilityPolicies::default();
        let empty = CapabilityPolicies::default()
            .restrict_commands(CapabilityPolicy::allow_only(std::iter::empty::<&str>()));
        let inherited = resolve_capabilities(&catalog(), &project, None, None);
        let narrowed = resolve_capabilities(&catalog(), &project, Some(&empty), None);
        assert_eq!(inherited.commands.len(), 2);
        assert!(narrowed.commands.is_empty());
    }
}
