//! Ownership-transfer payloads for moving one run to a successor agent.
//!
//! A handoff is deliberately separate from mail and nested invocation. It closes the predecessor,
//! opens one successor on the same task and branch, and primes that successor from one structured
//! artifact. The predecessor transcript is not a field in any type here.

use crate::ids::{ArtifactId, ProjectId, RunId, SessionId};
use crate::services::artifact::{ArtifactKind, ArtifactRow};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffItem {
    pub summary: String,
    #[serde(default)]
    pub evidence: Option<String>,
}

impl HandoffItem {
    pub fn new(summary: impl Into<String>, evidence: Option<impl Into<String>>) -> Result<Self> {
        let summary = summary.into();
        let evidence = evidence.map(Into::into);
        if summary.trim().is_empty() {
            bail!("handoff item summary must not be empty")
        }
        if evidence
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            bail!("handoff item evidence must not be empty")
        }
        Ok(Self { summary, evidence })
    }
}

/// The one payload shape used by every ownership-transfer trigger.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffPayload {
    pub goal: String,
    pub done: Vec<HandoffItem>,
    pub remaining: Vec<HandoffItem>,
    pub attempted: Vec<HandoffItem>,
    pub decisions: Vec<HandoffItem>,
    pub open: Vec<HandoffItem>,
    pub branch: String,
    pub task: String,
    /// References to existing artifacts. The payload never embeds their bodies.
    pub artifacts: Vec<ArtifactId>,
}

impl HandoffPayload {
    pub fn new(
        goal: impl Into<String>,
        branch: impl Into<String>,
        task: impl Into<String>,
    ) -> Result<Self> {
        let payload = Self {
            goal: goal.into(),
            done: Vec::new(),
            remaining: Vec::new(),
            attempted: Vec::new(),
            decisions: Vec::new(),
            open: Vec::new(),
            branch: branch.into(),
            task: task.into(),
            artifacts: Vec::new(),
        };
        payload.validate(false)
    }

    pub fn validate(&self, stuck: bool) -> Result<Self> {
        if self.goal.trim().is_empty() {
            bail!("handoff goal must not be empty")
        }
        if self.branch.trim().is_empty() {
            bail!("handoff branch must not be empty")
        }
        if self.task.trim().is_empty() {
            bail!("handoff task must not be empty")
        }
        for item in self
            .done
            .iter()
            .chain(self.remaining.iter())
            .chain(self.attempted.iter())
            .chain(self.decisions.iter())
            .chain(self.open.iter())
        {
            HandoffItem::new(item.summary.clone(), item.evidence.clone())?;
        }
        if self.done.iter().any(|item| item.evidence.is_none()) {
            bail!("each completed handoff item requires evidence")
        }
        if stuck && self.attempted.is_empty() {
            bail!("a stuck handoff requires at least one attempted item")
        }
        if self.artifacts.windows(2).any(|pair| pair[0] == pair[1]) {
            bail!("handoff artifact references must be unique")
        }
        Ok(self.clone())
    }

    pub fn with_done(
        mut self,
        summary: impl Into<String>,
        evidence: impl Into<String>,
    ) -> Result<Self> {
        self.done.push(HandoffItem::new(summary, Some(evidence))?);
        Ok(self)
    }

    pub fn with_remaining(mut self, summary: impl Into<String>) -> Result<Self> {
        self.remaining
            .push(HandoffItem::new(summary, None::<String>)?);
        Ok(self)
    }

    pub fn with_attempted(
        mut self,
        summary: impl Into<String>,
        evidence: Option<impl Into<String>>,
    ) -> Result<Self> {
        self.attempted.push(HandoffItem::new(summary, evidence)?);
        Ok(self)
    }

    pub fn with_decision(mut self, summary: impl Into<String>) -> Result<Self> {
        self.decisions
            .push(HandoffItem::new(summary, None::<String>)?);
        Ok(self)
    }

    pub fn with_open(mut self, summary: impl Into<String>) -> Result<Self> {
        self.open.push(HandoffItem::new(summary, None::<String>)?);
        Ok(self)
    }

    pub fn with_artifact(mut self, artifact: ArtifactId) -> Result<Self> {
        if self.artifacts.contains(&artifact) {
            bail!("handoff artifact references must be unique")
        }
        self.artifacts.push(artifact);
        Ok(self)
    }

    pub fn artifact_body(&self) -> Result<String> {
        serde_json::to_string(self).context("serialize handoff payload")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffTrigger {
    Stuck,
    ContextExhausted,
    RoleChange,
    HumanReassignment,
}

impl HandoffTrigger {
    fn requires_attempted(self) -> bool {
        matches!(self, Self::Stuck)
    }
}

/// The minimum predecessor context needed to create a successor. Transcript data is intentionally
/// absent; callers provide artifact handles instead.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffContext {
    pub project_id: ProjectId,
    pub session_id: SessionId,
    pub run_id: RunId,
    pub branch: String,
    pub task: String,
}

impl HandoffContext {
    pub fn new(
        project_id: ProjectId,
        session_id: SessionId,
        run_id: RunId,
        branch: impl Into<String>,
        task: impl Into<String>,
    ) -> Result<Self> {
        let context = Self {
            project_id,
            session_id,
            run_id,
            branch: branch.into(),
            task: task.into(),
        };
        if context.branch.trim().is_empty() || context.task.trim().is_empty() {
            bail!("handoff context requires a branch and task")
        }
        Ok(context)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffSessionStatus {
    Active,
    Closed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HandoffSession {
    pub session_id: SessionId,
    pub run_id: RunId,
    pub branch: String,
    pub task: String,
    pub agent: String,
    pub status: HandoffSessionStatus,
    pub handed_off_from: Option<SessionId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HandoffRecord {
    pub project_id: ProjectId,
    pub artifact_id: ArtifactId,
    pub predecessor_session_id: SessionId,
    pub predecessor_run_id: RunId,
    pub successor_session_id: SessionId,
    pub successor_run_id: RunId,
    /// The successor stores the same link under this name for direct traversal.
    pub handed_off_from: SessionId,
    pub target_agent: String,
    pub trigger: HandoffTrigger,
    pub payload: HandoffPayload,
}

impl HandoffRecord {
    fn new(
        predecessor: &HandoffContext,
        successor: &HandoffContext,
        target_agent: impl Into<String>,
        trigger: HandoffTrigger,
        payload: HandoffPayload,
    ) -> Result<Self> {
        let target_agent = target_agent.into();
        if target_agent.trim().is_empty() {
            bail!("handoff target agent must not be empty")
        }
        if predecessor.project_id != successor.project_id {
            bail!("handoffs cannot cross projects")
        }
        if predecessor.session_id == successor.session_id || predecessor.run_id == successor.run_id
        {
            bail!("handoff successor must be distinct from its predecessor")
        }
        if predecessor.branch != successor.branch || predecessor.task != successor.task {
            bail!("handoff successor must keep the predecessor branch and task")
        }
        if payload.branch != predecessor.branch || payload.task != predecessor.task {
            bail!("handoff payload must keep the predecessor branch and task")
        }
        payload.validate(trigger.requires_attempted())?;
        Ok(Self {
            project_id: predecessor.project_id,
            artifact_id: ArtifactId::generate(),
            predecessor_session_id: predecessor.session_id,
            predecessor_run_id: predecessor.run_id,
            successor_session_id: successor.session_id,
            successor_run_id: successor.run_id,
            handed_off_from: predecessor.session_id,
            target_agent,
            trigger,
            payload,
        })
    }

    pub fn artifact_row(&self, project_id: ProjectId) -> Result<ArtifactRow> {
        if project_id != self.project_id {
            bail!("handoff artifact project does not match the transfer")
        }
        let mut row = ArtifactRow::text(
            project_id,
            self.predecessor_run_id,
            ArtifactKind::Payload,
            self.payload.artifact_body()?,
        );
        row.id = self.artifact_id;
        row.summary = Some(format!(
            "handoff to {} · {} remaining · {} attempted",
            self.target_agent,
            self.payload.remaining.len(),
            self.payload.attempted.len()
        ));
        Ok(row)
    }
}

/// Core-owned handoff state. It stores references and priming payloads, never a transcript.
#[derive(Clone, Default)]
pub struct HandoffRegistry {
    records: BTreeMap<ArtifactId, HandoffRecord>,
    predecessor_by_successor: BTreeMap<SessionId, SessionId>,
    successor_by_predecessor: BTreeMap<SessionId, SessionId>,
    sessions: BTreeMap<SessionId, HandoffSession>,
    priming: BTreeMap<SessionId, HandoffPayload>,
}

impl HandoffRegistry {
    pub fn transfer(
        &mut self,
        predecessor: HandoffContext,
        successor: HandoffContext,
        target_agent: impl Into<String>,
        trigger: HandoffTrigger,
        payload: HandoffPayload,
    ) -> Result<HandoffRecord> {
        if self
            .successor_by_predecessor
            .contains_key(&predecessor.session_id)
        {
            bail!("handoff predecessor has already transferred ownership")
        }
        let record = HandoffRecord::new(&predecessor, &successor, target_agent, trigger, payload)?;
        self.sessions.insert(
            predecessor.session_id,
            HandoffSession {
                session_id: predecessor.session_id,
                run_id: predecessor.run_id,
                branch: predecessor.branch.clone(),
                task: predecessor.task.clone(),
                agent: "predecessor".into(),
                status: HandoffSessionStatus::Closed,
                handed_off_from: None,
            },
        );
        self.sessions.insert(
            successor.session_id,
            HandoffSession {
                session_id: successor.session_id,
                run_id: successor.run_id,
                branch: successor.branch,
                task: successor.task,
                agent: record.target_agent.clone(),
                status: HandoffSessionStatus::Active,
                handed_off_from: Some(predecessor.session_id),
            },
        );
        self.predecessor_by_successor
            .insert(record.successor_session_id, record.predecessor_session_id);
        self.successor_by_predecessor
            .insert(record.predecessor_session_id, record.successor_session_id);
        self.priming
            .insert(record.successor_session_id, record.payload.clone());
        self.records.insert(record.artifact_id, record.clone());
        Ok(record)
    }

    pub fn record(&self, artifact_id: ArtifactId) -> Option<&HandoffRecord> {
        self.records.get(&artifact_id)
    }

    pub fn predecessor(&self, successor: SessionId) -> Option<SessionId> {
        self.predecessor_by_successor.get(&successor).copied()
    }

    pub fn successor(&self, predecessor: SessionId) -> Option<SessionId> {
        self.successor_by_predecessor.get(&predecessor).copied()
    }

    pub fn session(&self, session_id: SessionId) -> Option<&HandoffSession> {
        self.sessions.get(&session_id)
    }

    /// This is the exact context injected into a successor. It has no transcript field.
    pub fn priming_context(&self, successor: SessionId) -> Option<Value> {
        self.priming
            .get(&successor)
            .and_then(|payload| serde_json::to_value(payload).ok())
    }

    pub fn records(&self) -> impl Iterator<Item = &HandoffRecord> {
        self.records.values()
    }
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod handoff {
    use super::*;
    use crate::services::artifact::ArtifactStore;

    fn contexts() -> (HandoffContext, HandoffContext) {
        let project = ProjectId::generate();
        let predecessor = HandoffContext::new(
            project,
            SessionId::generate(),
            RunId::generate(),
            "agent/feature",
            "task-17",
        )
        .unwrap();
        let successor = HandoffContext::new(
            project,
            SessionId::generate(),
            RunId::generate(),
            "agent/feature",
            "task-17",
        )
        .unwrap();
        (predecessor, successor)
    }

    fn payload(trigger: HandoffTrigger) -> HandoffPayload {
        let artifact = ArtifactId::generate();
        let payload = HandoffPayload::new("ship the feature", "agent/feature", "task-17")
            .unwrap()
            .with_done("reproduced the failure", "cargo test repro::case")
            .unwrap()
            .with_remaining("implement the fix")
            .unwrap()
            .with_artifact(artifact)
            .unwrap();
        if trigger.requires_attempted() {
            payload
                .with_attempted("tried the old retry loop", Some("it deadlocked"))
                .unwrap()
        } else {
            payload
        }
    }

    #[test]
    fn payload_shape() {
        let payload = payload(HandoffTrigger::HumanReassignment);
        let value = serde_json::to_value(&payload).unwrap();
        for field in [
            "goal",
            "done",
            "remaining",
            "attempted",
            "decisions",
            "open",
            "branch",
            "task",
            "artifacts",
        ] {
            assert!(value.get(field).is_some(), "missing {field}");
        }
        assert!(value.get("transcript").is_none());
    }

    #[test]
    fn persists_as_artifact() {
        let (predecessor, successor) = contexts();
        let mut registry = HandoffRegistry::default();
        let record = registry
            .transfer(
                predecessor.clone(),
                successor,
                "auditor",
                HandoffTrigger::HumanReassignment,
                payload(HandoffTrigger::HumanReassignment),
            )
            .unwrap();
        let row = record.artifact_row(predecessor.project_id).unwrap();
        let mut artifacts = ArtifactStore::default();
        artifacts.put(row.clone());
        assert_eq!(artifacts.get(row.id), Some(&row));
    }

    #[test]
    fn links_sessions() {
        let (predecessor, successor) = contexts();
        let mut registry = HandoffRegistry::default();
        let record = registry
            .transfer(
                predecessor.clone(),
                successor.clone(),
                "auditor",
                HandoffTrigger::HumanReassignment,
                payload(HandoffTrigger::HumanReassignment),
            )
            .unwrap();
        assert_eq!(record.handed_off_from, predecessor.session_id);
        assert_eq!(
            registry
                .session(successor.session_id)
                .unwrap()
                .handed_off_from,
            Some(predecessor.session_id)
        );
    }

    #[test]
    fn chain_traversable() {
        let (first, second) = contexts();
        let third = HandoffContext::new(
            first.project_id,
            SessionId::generate(),
            RunId::generate(),
            "agent/feature",
            "task-17",
        )
        .unwrap();
        let mut registry = HandoffRegistry::default();
        registry
            .transfer(
                first.clone(),
                second.clone(),
                "reviewer",
                HandoffTrigger::HumanReassignment,
                payload(HandoffTrigger::HumanReassignment),
            )
            .unwrap();
        registry
            .transfer(
                second.clone(),
                third.clone(),
                "builder",
                HandoffTrigger::ContextExhausted,
                payload(HandoffTrigger::ContextExhausted),
            )
            .unwrap();
        assert_eq!(
            registry.predecessor(third.session_id),
            Some(second.session_id)
        );
        assert_eq!(
            registry.predecessor(second.session_id),
            Some(first.session_id)
        );
        assert_eq!(
            registry.successor(first.session_id),
            Some(second.session_id)
        );
        assert_eq!(
            registry.successor(second.session_id),
            Some(third.session_id)
        );
    }

    #[test]
    fn same_task_and_branch() {
        let (predecessor, mut successor) = contexts();
        successor.branch = "agent/other".into();
        let error = HandoffRegistry::default().transfer(
            predecessor,
            successor,
            "auditor",
            HandoffTrigger::HumanReassignment,
            payload(HandoffTrigger::HumanReassignment),
        );
        assert!(error.unwrap_err().to_string().contains("branch and task"));
    }

    #[test]
    fn primes_from_payload() {
        let (predecessor, successor) = contexts();
        let mut registry = HandoffRegistry::default();
        let record = registry
            .transfer(
                predecessor,
                successor.clone(),
                "auditor",
                HandoffTrigger::HumanReassignment,
                payload(HandoffTrigger::HumanReassignment),
            )
            .unwrap();
        let priming = registry.priming_context(successor.session_id).unwrap();
        assert_eq!(priming["goal"], record.payload.goal);
        assert!(priming.get("transcript").is_none());
    }

    #[test]
    fn no_transcript_replay() {
        let (predecessor, successor) = contexts();
        let mut registry = HandoffRegistry::default();
        registry
            .transfer(
                predecessor,
                successor.clone(),
                "auditor",
                HandoffTrigger::HumanReassignment,
                payload(HandoffTrigger::HumanReassignment),
            )
            .unwrap();
        let injected =
            serde_json::to_string(&registry.priming_context(successor.session_id)).unwrap();
        assert!(!injected.contains("transcript"));
    }

    #[test]
    fn attempted_required_when_stuck() {
        let error = payload(HandoffTrigger::HumanReassignment)
            .validate(true)
            .unwrap_err();
        assert!(error.to_string().contains("attempted"));
    }

    fn transfer_for(trigger: HandoffTrigger) -> HandoffRecord {
        let (predecessor, successor) = contexts();
        HandoffRegistry::default()
            .transfer(
                predecessor,
                successor,
                "next-agent",
                trigger,
                payload(trigger),
            )
            .unwrap()
    }

    #[test]
    fn from_guardrail() {
        assert_eq!(
            transfer_for(HandoffTrigger::Stuck).trigger,
            HandoffTrigger::Stuck
        );
    }

    #[test]
    fn from_context_exhaustion() {
        assert_eq!(
            transfer_for(HandoffTrigger::ContextExhausted).trigger,
            HandoffTrigger::ContextExhausted
        );
    }

    #[test]
    fn from_graph() {
        assert_eq!(
            transfer_for(HandoffTrigger::RoleChange).trigger,
            HandoffTrigger::RoleChange
        );
    }

    #[test]
    fn from_human() {
        assert_eq!(
            transfer_for(HandoffTrigger::HumanReassignment).trigger,
            HandoffTrigger::HumanReassignment
        );
    }

    #[test]
    fn one_shape_four_triggers() {
        let shapes = [
            HandoffTrigger::Stuck,
            HandoffTrigger::ContextExhausted,
            HandoffTrigger::RoleChange,
            HandoffTrigger::HumanReassignment,
        ]
        .map(|trigger| {
            let value = serde_json::to_value(transfer_for(trigger).payload).unwrap();
            value
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect::<Vec<_>>()
        });
        assert!(shapes.windows(2).all(|pair| pair[0] == pair[1]));
    }

    #[test]
    fn does_not_return() {
        let (predecessor, successor) = contexts();
        let mut registry = HandoffRegistry::default();
        registry
            .transfer(
                predecessor.clone(),
                successor,
                "auditor",
                HandoffTrigger::HumanReassignment,
                payload(HandoffTrigger::HumanReassignment),
            )
            .unwrap();
        assert!(registry
            .transfer(
                predecessor,
                contexts().1,
                "builder",
                HandoffTrigger::HumanReassignment,
                payload(HandoffTrigger::HumanReassignment),
            )
            .is_err());
    }

    #[test]
    fn references_not_copies() {
        let (predecessor, successor) = contexts();
        let artifact = ArtifactId::generate();
        let payload = HandoffPayload::new("goal", "agent/feature", "task-17")
            .unwrap()
            .with_artifact(artifact)
            .unwrap();
        let mut registry = HandoffRegistry::default();
        let record = registry
            .transfer(
                predecessor,
                successor,
                "auditor",
                HandoffTrigger::HumanReassignment,
                payload,
            )
            .unwrap();
        assert_eq!(record.payload.artifacts, [artifact]);
        assert!(!record.payload.artifact_body().unwrap().contains("body"));
    }
}
