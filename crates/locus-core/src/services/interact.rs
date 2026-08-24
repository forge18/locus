//! Board-less Interact sessions.
//!
//! An Interact session is intentionally explicit rather than inferred from a nullable board task.
//! That keeps promotion and discard terminal, auditable transitions.

use crate::ids::{ProjectId, SessionId, TaskId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InteractState {
    Open,
    Promoted,
    Discarded,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InteractSession {
    pub id: SessionId,
    pub project_id: ProjectId,
    pub repo: String,
    pub branch: String,
    pub board_task_id: Option<TaskId>,
    pub container_id: Option<String>,
    pub state: InteractState,
}

impl InteractSession {
    pub fn open(id: SessionId, project_id: ProjectId, repo: impl Into<String>) -> Self {
        let id_text = id.to_string();
        Self {
            id,
            project_id,
            repo: repo.into(),
            branch: format!("interact/{id_text}"),
            board_task_id: None,
            container_id: None,
            state: InteractState::Open,
        }
    }

    pub fn promote(&mut self, task_id: TaskId) -> Result<(), InteractError> {
        if self.state != InteractState::Open || self.board_task_id.is_some() {
            return Err(InteractError::NotOpen);
        }
        self.board_task_id = Some(task_id);
        self.state = InteractState::Promoted;
        Ok(())
    }

    pub fn discard(&mut self) -> Result<DiscardPlan, InteractError> {
        if self.state != InteractState::Open {
            return Err(InteractError::NotOpen);
        }
        self.state = InteractState::Discarded;
        Ok(DiscardPlan {
            container_id: self.container_id.take(),
            branch: self.branch.clone(),
        })
    }

    pub fn can_commit(&self) -> bool {
        self.state == InteractState::Open
    }
    pub fn can_discard(&self) -> bool {
        self.state == InteractState::Open
    }
    pub fn meta_chip(&self, changed_files: usize) -> String {
        match self.state {
            InteractState::Open if changed_files == 0 => "clean".into(),
            InteractState::Open => format!("{changed_files} changed"),
            InteractState::Promoted => self
                .board_task_id
                .map_or_else(|| "promoted".into(), |id| format!("→ #{id}")),
            InteractState::Discarded => "discarded".into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscardPlan {
    pub container_id: Option<String>,
    pub branch: String,
}

pub fn reconciliation_skips_discarded_session(state: InteractState) -> bool {
    state == InteractState::Discarded
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum InteractError {
    #[error("interact session is not open")]
    NotOpen,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangedFile {
    pub path: String,
    pub marker: char,
    pub additions: u32,
    pub removals: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangedSessionNote {
    pub state: InteractState,
    pub text: &'static str,
}

impl ChangedSessionNote {
    pub fn for_state(state: InteractState) -> Self {
        let text = match state {
            InteractState::Open => "This session has no card, so no approval gate and nothing in your Inbox. This panel is the only account of what it touched.",
            InteractState::Promoted => "This session was promoted to a card, so its diff now takes the normal gate. What you see here is the record of what it touched before that.",
            InteractState::Discarded => "This session was discarded. The container and branch are gone; the transcript stays for the record.",
        };
        Self { state, text }
    }
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod interact {
    use super::*;

    #[test]
    fn session_state_enum() {
        assert_eq!(
            serde_json::to_string(&InteractState::Open).unwrap(),
            "\"open\""
        );
        assert_ne!(InteractState::Open, InteractState::Discarded);
    }

    #[test]
    fn opens_boardless_session_on_interact_branch() {
        let session = InteractSession::open(SessionId::generate(), ProjectId::generate(), "repo");
        assert_eq!(session.state, InteractState::Open);
        assert!(session.board_task_id.is_none());
        assert!(session.branch.starts_with("interact/"));
    }

    #[test]
    fn session_transitions_are_terminal() {
        let mut session =
            InteractSession::open(SessionId::generate(), ProjectId::generate(), "repo");
        session.promote(TaskId::generate()).unwrap();
        assert!(session.promote(TaskId::generate()).is_err());
        assert!(session.discard().is_err());
    }

    #[test]
    fn promote_attaches_board_task() {
        let mut session =
            InteractSession::open(SessionId::generate(), ProjectId::generate(), "repo");
        let task = TaskId::generate();
        session.promote(task).unwrap();
        assert_eq!(session.board_task_id, Some(task));
        assert_eq!(session.state, InteractState::Promoted);
    }

    #[test]
    fn promoted_session_has_no_interact_actions() {
        let mut session =
            InteractSession::open(SessionId::generate(), ProjectId::generate(), "repo");
        session.promote(TaskId::generate()).unwrap();
        assert!(!session.can_commit());
        assert!(!session.can_discard());
    }

    #[test]
    fn discard_kills_container_and_deletes_branch() {
        let mut session =
            InteractSession::open(SessionId::generate(), ProjectId::generate(), "repo");
        session.container_id = Some("container".into());
        let branch = session.branch.clone();
        let plan = session.discard().unwrap();
        assert_eq!(plan.container_id.as_deref(), Some("container"));
        assert_eq!(plan.branch, branch);
        assert_eq!(session.state, InteractState::Discarded);
    }

    #[test]
    fn discard_retains_history() {
        let mut session =
            InteractSession::open(SessionId::generate(), ProjectId::generate(), "repo");
        session.discard().unwrap();
        assert_eq!(session.state, InteractState::Discarded);
        assert!(session.branch.starts_with("interact/"));
    }

    #[test]
    fn reconciliation_skips_discarded_session() {
        assert!(super::reconciliation_skips_discarded_session(
            InteractState::Discarded
        ));
        assert!(!super::reconciliation_skips_discarded_session(
            InteractState::Open
        ));
    }

    #[test]
    fn commit_to_branch_preserves_open_state() {
        let session = InteractSession::open(SessionId::generate(), ProjectId::generate(), "repo");
        assert!(session.can_commit());
        assert_eq!(session.state, InteractState::Open);
    }

    #[test]
    fn session_meta_chip() {
        let mut session =
            InteractSession::open(SessionId::generate(), ProjectId::generate(), "repo");
        assert_eq!(session.meta_chip(0), "clean");
        assert_eq!(session.meta_chip(2), "2 changed");
        session.promote(TaskId::generate()).unwrap();
        assert!(session.meta_chip(2).starts_with("→ #"));
        let _ = session.discard();
    }
}
