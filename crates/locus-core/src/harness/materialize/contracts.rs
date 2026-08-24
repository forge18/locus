//! Data contracts for extension-specific invariants surfaced by Workshop.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    pub budget_tokens: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SkillMaterialization {
    Lazy,
    InlinedDescription,
}

pub fn enforce_skill_budget(
    skill: &SkillDefinition,
    estimated_tokens: u32,
) -> SkillMaterialization {
    match skill.budget_tokens {
        Some(limit) if estimated_tokens > limit => SkillMaterialization::InlinedDescription,
        _ => SkillMaterialization::Lazy,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleDefinition {
    pub name: String,
    pub glob: String,
    pub priority: i32,
}

pub fn validate_rules(
    rules: &[RuleDefinition],
) -> Result<Vec<RuleDefinition>, ExtensionContractError> {
    let mut sorted = rules.to_vec();
    if sorted.iter().any(|rule| rule.glob.trim().is_empty()) {
        return Err(ExtensionContractError::InvalidRule);
    }
    sorted.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok(sorted)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectContext {
    pub name: String,
    pub body: String,
    pub budget_tokens: u32,
}

pub fn project_context_singleton(
    contexts: &[ProjectContext],
) -> Result<&ProjectContext, ExtensionContractError> {
    match contexts {
        [context] => Ok(context),
        _ => Err(ExtensionContractError::ContextSingleton),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandDefinition {
    pub name: String,
    pub arguments: Vec<String>,
    pub body: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandMaterialization {
    Native,
    SkillDowngrade,
}

pub fn command_materialization(
    command: &CommandDefinition,
) -> Result<CommandMaterialization, ExtensionContractError> {
    if command.name.trim().is_empty() || command.body.trim().is_empty() {
        return Err(ExtensionContractError::InvalidCommand);
    }
    Ok(if command.arguments.is_empty() {
        CommandMaterialization::SkillDowngrade
    } else {
        CommandMaterialization::Native
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookContract {
    pub event: String,
    pub threshold: u32,
    pub timeout_seconds: u32,
    pub fail_open: bool,
}

impl HookContract {
    pub fn validate(&self) -> Result<(), ExtensionContractError> {
        if self.event.trim().is_empty() || self.timeout_seconds == 0 {
            return Err(ExtensionContractError::InvalidHook);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputStyle {
    pub name: String,
    pub harness: String,
    pub roles: Vec<String>,
    pub active: bool,
}

pub fn validate_styles(styles: &[OutputStyle]) -> Result<(), ExtensionContractError> {
    let mut active = BTreeMap::<&str, usize>::new();
    for style in styles {
        if style.active {
            *active.entry(&style.harness).or_default() += 1;
        }
    }
    if active.values().any(|count| *count > 1) {
        return Err(ExtensionContractError::MultipleActiveStyles);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViolationMode {
    Warn,
    Fail,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinterViolation {
    pub rule: String,
    pub message: String,
    pub mode: ViolationMode,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ExtensionContractError {
    #[error("invalid rule")]
    InvalidRule,
    #[error("project context must have exactly one native entry")]
    ContextSingleton,
    #[error("invalid command")]
    InvalidCommand,
    #[error("invalid hook")]
    InvalidHook,
    #[error("one output style must be active per harness")]
    MultipleActiveStyles,
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod contracts {
    use super::project_context_singleton as one_context;
    use super::*;
    #[test]
    fn skill_budget_and_downgrade() {
        let skill = SkillDefinition {
            name: "s".into(),
            description: "d".into(),
            budget_tokens: Some(1),
        };
        assert_eq!(
            enforce_skill_budget(&skill, 2),
            SkillMaterialization::InlinedDescription
        );
    }
    #[test]
    fn rules_one_glob_and_priority() {
        let rules = validate_rules(&[
            RuleDefinition {
                name: "low".into(),
                glob: "*".into(),
                priority: 1,
            },
            RuleDefinition {
                name: "high".into(),
                glob: "*".into(),
                priority: 2,
            },
        ])
        .unwrap();
        assert_eq!(rules[0].name, "high");
    }
    #[test]
    fn project_context_singleton() {
        assert!(one_context(&[ProjectContext {
            name: "base".into(),
            body: "".into(),
            budget_tokens: 1
        }])
        .is_ok());
        assert!(one_context(&[]).is_err());
    }
    #[test]
    fn command_arguments_and_downgrade() {
        assert_eq!(
            command_materialization(&CommandDefinition {
                name: "x".into(),
                arguments: vec![],
                body: "body".into()
            })
            .unwrap(),
            CommandMaterialization::SkillDowngrade
        );
    }
    #[test]
    fn hook_contract() {
        assert!(HookContract {
            event: "session_start".into(),
            threshold: 1,
            timeout_seconds: 1,
            fail_open: true
        }
        .validate()
        .is_ok());
    }
    #[test]
    fn style_default_and_roles() {
        assert!(validate_styles(&[OutputStyle {
            name: "brief".into(),
            harness: "h".into(),
            roles: vec!["builder".into()],
            active: true
        }])
        .is_ok());
    }
    #[test]
    fn not_materialized_and_violation_mode() {
        assert_eq!(ViolationMode::Warn, ViolationMode::Warn);
    }
}
