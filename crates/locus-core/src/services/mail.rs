//! Stored agent-to-agent mail projections and the waiting distinction.

use crate::ids::{ProjectId, RunId, SessionId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThreadStatus {
    Waiting,
    Open,
    Replied,
    You,
    Drained,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MailVerb {
    Send,
    Read,
    Reply,
    Wait,
    Drain,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MailMessage {
    pub id: String,
    pub sender: String,
    pub recipient: String,
    pub verb: MailVerb,
    pub body: String,
    pub created_at: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MailThread {
    pub id: String,
    pub project_id: ProjectId,
    pub subject: String,
    pub status: ThreadStatus,
    pub messages: Vec<MailMessage>,
    pub waiting_run: Option<RunId>,
    pub wait_started_at: Option<i64>,
    pub wait_timeout_seconds: Option<u64>,
}

impl MailThread {
    pub fn waiting(&self) -> bool {
        self.status == ThreadStatus::Waiting && self.waiting_run.is_some()
    }
    pub fn wait_copy(&self, now: i64) -> Option<String> {
        self.waiting_run.map(|run| {
            let elapsed = now.saturating_sub(self.wait_started_at.unwrap_or(now));
            format!(
                "{run} is in mail wait — {}m of a {}m timeout",
                elapsed / 60,
                self.wait_timeout_seconds.unwrap_or(900) / 60
            )
        })
    }
    pub fn idle_guardrail_applies(&self) -> bool {
        !self.waiting()
    }
    pub fn drain(&mut self) {
        self.status = ThreadStatus::Drained;
    }
    pub fn unblock(&mut self) {
        if self.status == ThreadStatus::Waiting {
            self.status = ThreadStatus::Open;
            self.waiting_run = None;
        }
    }
    pub fn accepts_mail(&self) -> bool {
        self.status != ThreadStatus::Drained
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Participant {
    pub session_id: SessionId,
    pub run_id: RunId,
    pub state: ThreadStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffBoundary {
    pub artifact_id: Option<String>,
    pub ownership_transferred: bool,
}

impl HandoffBoundary {
    pub fn transfer(&mut self, artifact_id: impl Into<String>) {
        self.artifact_id = Some(artifact_id.into());
        self.ownership_transferred = true;
    }
    pub fn accepts_mail(&self) -> bool {
        !self.ownership_transferred
    }
}

pub const WAITING_NOT_IDLE: &str =
    "State is `waiting`, not idle. The idle guardrail will not fire.";

#[cfg(test)]
mod mail {
    use super::*;
    fn thread() -> MailThread {
        MailThread {
            id: "t".into(),
            project_id: ProjectId::generate(),
            subject: "subject".into(),
            status: ThreadStatus::Waiting,
            messages: vec![],
            waiting_run: Some(RunId::generate()),
            wait_started_at: Some(0),
            wait_timeout_seconds: Some(900),
        }
    }
    #[test]
    fn three_pane_tabs() {
        assert_eq!(thread().status, ThreadStatus::Waiting);
    }
    #[test]
    fn thread_status_vocabulary() {
        assert_eq!(
            serde_json::to_string(&ThreadStatus::Drained).unwrap(),
            "\"drained\""
        );
    }
    #[test]
    fn wait_banner_copy() {
        assert!(thread().wait_copy(480).unwrap().contains("mail wait"));
        assert_eq!(
            WAITING_NOT_IDLE,
            "State is `waiting`, not idle. The idle guardrail will not fire."
        );
    }
    #[test]
    fn verb_tags() {
        assert_eq!(
            serde_json::to_string(&MailVerb::Reply).unwrap(),
            "\"reply\""
        );
    }
    #[test]
    fn drain_unblock() {
        let mut t = thread();
        t.unblock();
        assert_eq!(t.status, ThreadStatus::Open);
        t.drain();
        assert!(!t.accepts_mail());
    }
    #[test]
    fn handoff_boundary() {
        let mut handoff = HandoffBoundary {
            artifact_id: None,
            ownership_transferred: false,
        };
        handoff.transfer("a");
        assert!(!handoff.accepts_mail());
    }
    #[test]
    fn participants_rail() {
        let participant = Participant {
            session_id: SessionId::generate(),
            run_id: RunId::generate(),
            state: ThreadStatus::Open,
        };
        assert_eq!(participant.state, ThreadStatus::Open);
    }
}
