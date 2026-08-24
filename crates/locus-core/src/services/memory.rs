//! Project-scoped memory folds and curated fact revisions.
//!
//! The Postgres schema is the durable source; these types are the service boundary used by the
//! CLI, store adapters, and desktop projections.  Editing is append-only: revision one remains
//! the agent's assertion and revision two is the human curation returned by recall.

use crate::ids::{ProjectId, RunId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceState {
    Verified,
    Asserted,
    Decaying,
    Contradicted,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FactRevision {
    pub rev: u32,
    pub value: String,
    pub written_by_run: Option<RunId>,
    pub curated_by: Option<String>,
    pub written_at: i64,
    pub score: Option<f32>,
}

impl FactRevision {
    pub fn new(
        rev: u32,
        value: impl Into<String>,
        written_by_run: Option<RunId>,
        written_at: i64,
    ) -> Result<Self, MemoryError> {
        let value = value.into();
        if rev == 0 || value.trim().is_empty() {
            return Err(MemoryError::InvalidFact);
        }
        Ok(Self {
            rev,
            value,
            written_by_run,
            curated_by: None,
            written_at,
            score: None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fact {
    pub id: String,
    pub project_id: ProjectId,
    pub subject: String,
    pub confidence: ConfidenceState,
    pub revisions: Vec<FactRevision>,
    pub recall_count: u32,
}

impl Fact {
    pub fn new(
        id: impl Into<String>,
        project_id: ProjectId,
        subject: impl Into<String>,
        value: impl Into<String>,
        run: Option<RunId>,
        now: i64,
    ) -> Result<Self, MemoryError> {
        let id = id.into();
        let subject = subject.into();
        if id.trim().is_empty() || subject.trim().is_empty() {
            return Err(MemoryError::InvalidFact);
        }
        Ok(Self {
            id,
            project_id,
            subject,
            confidence: ConfidenceState::Asserted,
            revisions: vec![FactRevision::new(1, value, run, now)?],
            recall_count: 0,
        })
    }

    pub fn latest(&self) -> &FactRevision {
        self.revisions
            .last()
            .expect("a Fact always has revision one")
    }
    pub fn score(&self) -> Option<f32> {
        (self.confidence != ConfidenceState::Contradicted)
            .then(|| self.latest().score.unwrap_or(0.0))
    }
    pub fn edit(
        &mut self,
        value: impl Into<String>,
        human: impl Into<String>,
        now: i64,
    ) -> Result<&FactRevision, MemoryError> {
        let value = value.into();
        let human = human.into();
        if value.trim().is_empty() || human.trim().is_empty() {
            return Err(MemoryError::InvalidFact);
        }
        let revision = FactRevision {
            rev: self.latest().rev + 1,
            value,
            written_by_run: self
                .revisions
                .first()
                .and_then(|revision| revision.written_by_run),
            curated_by: Some(human),
            written_at: now,
            score: self.latest().score,
        };
        self.revisions.push(revision);
        Ok(self.latest())
    }
    pub fn recall(&mut self) -> &FactRevision {
        self.recall_count += 1;
        self.latest()
    }
    pub fn contradict(&mut self) {
        self.confidence = ConfidenceState::Contradicted;
        for revision in &mut self.revisions {
            revision.score = None;
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MemoryStore {
    facts: BTreeMap<String, Fact>,
}

impl MemoryStore {
    pub fn insert(&mut self, fact: Fact) -> Result<(), MemoryError> {
        if self.facts.insert(fact.id.clone(), fact).is_some() {
            return Err(MemoryError::DuplicateFact);
        }
        Ok(())
    }
    pub fn get(&self, id: &str) -> Option<&Fact> {
        self.facts.get(id)
    }
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Fact> {
        self.facts.get_mut(id)
    }
    pub fn recall(&mut self, id: &str) -> Result<FactRevision, MemoryError> {
        self.facts
            .get_mut(id)
            .map(|fact| fact.recall().clone())
            .ok_or(MemoryError::UnknownFact)
    }
    pub fn project_facts(&self, project: ProjectId) -> Vec<&Fact> {
        self.facts
            .values()
            .filter(|fact| fact.project_id == project)
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contradiction {
    pub fact_id: String,
    pub existing_value: String,
    pub incoming_value: String,
    pub existing_source: Option<RunId>,
    pub incoming_source: Option<RunId>,
    pub resolved: bool,
}

impl Contradiction {
    pub fn adjudicate(&mut self) {
        self.resolved = true;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceGroup {
    ShortTerm,
    LongTerm,
    Artifacts,
}

impl PersistenceGroup {
    pub fn can_delete(self) -> bool {
        matches!(self, Self::LongTerm | Self::Artifacts)
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum MemoryError {
    #[error("invalid memory fact")]
    InvalidFact,
    #[error("memory fact already exists")]
    DuplicateFact,
    #[error("memory fact does not exist")]
    UnknownFact,
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod memory {
    use super::*;
    #[test]
    fn fact_revision_schema() {
        let fact = Fact::new(
            "f",
            ProjectId::generate(),
            "subject",
            "one",
            Some(RunId::generate()),
            1,
        )
        .unwrap();
        assert_eq!(fact.latest().rev, 1);
    }
    #[test]
    fn edit_appends_revision() {
        let run = RunId::generate();
        let mut fact =
            Fact::new("f", ProjectId::generate(), "subject", "one", Some(run), 1).unwrap();
        fact.edit("two", "human", 2).unwrap();
        assert_eq!(fact.revisions[0].value, "one");
        assert_eq!(fact.latest().rev, 2);
    }
    #[test]
    fn recall_returns_curated() {
        let run = RunId::generate();
        let mut fact =
            Fact::new("f", ProjectId::generate(), "subject", "one", Some(run), 1).unwrap();
        fact.edit("two", "human", 2).unwrap();
        let revision = fact.recall().clone();
        assert_eq!(revision.value, "two");
        assert_eq!(revision.written_by_run, Some(run));
    }
    #[test]
    fn confidence_state_enum() {
        assert_eq!(
            serde_json::to_string(&ConfidenceState::Verified).unwrap(),
            "\"verified\""
        );
    }
    #[test]
    fn contradicted_has_no_score() {
        let mut fact = Fact::new("f", ProjectId::generate(), "subject", "one", None, 1).unwrap();
        fact.contradict();
        assert_eq!(fact.score(), None);
    }
    #[test]
    fn persistence_groups() {
        assert!(!PersistenceGroup::ShortTerm.can_delete());
        assert!(PersistenceGroup::LongTerm.can_delete());
    }
    #[test]
    fn delete_scoped_to_long_term_and_artifacts() {
        assert!(!PersistenceGroup::ShortTerm.can_delete());
        assert!(PersistenceGroup::Artifacts.can_delete());
    }
}
