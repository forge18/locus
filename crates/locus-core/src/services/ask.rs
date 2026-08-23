//! Human escalation owned by the core rather than the in-container CLI.

use crate::ids::{ProjectId, RunId, SessionId};
use anyhow::{bail, Result};
use uuid::Uuid;

/// The durable context required to deliver an agent's question to a human.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskRequest {
    pub project_id: ProjectId,
    pub session_id: SessionId,
    pub run_id: RunId,
    pub question: String,
}

/// The persisted result returned to the caller after it is blocked for an answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskReceipt {
    pub delivery_id: Uuid,
    pub session_id: SessionId,
    pub waiting: bool,
}

/// Storage boundary for the human inbox and the run's active wait.
///
/// Implementations persist both changes in one transaction, so a failed wait-state update cannot
/// leave an unanswered question filed for a run that is still active.
pub trait HumanInbox {
    fn deliver_and_mark_waiting(&self, request: &AskRequest) -> Result<Uuid>;
}

/// Atomically files an inbox item attached to the originating session and blocks its run.
pub fn ask(inbox: &impl HumanInbox, request: AskRequest) -> Result<AskReceipt> {
    if request.question.trim().is_empty() {
        bail!("ask question must not be empty")
    }

    let delivery_id = inbox.deliver_and_mark_waiting(&request)?;
    Ok(AskReceipt {
        delivery_id,
        session_id: request.session_id,
        waiting: true,
    })
}

#[cfg(test)]
mod reaches_inbox {
    use crate::ids::{ProjectId, RunId, SessionId};
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct RecordingInbox {
        delivered: Mutex<Vec<AskRequest>>,
        waits: Mutex<Vec<(RunId, String)>>,
    }

    impl HumanInbox for RecordingInbox {
        fn deliver_and_mark_waiting(&self, request: &AskRequest) -> Result<Uuid> {
            self.delivered.lock().unwrap().push(request.clone());
            self.waits
                .lock()
                .unwrap()
                .push((request.run_id, "ask".into()));
            Ok(Uuid::nil())
        }
    }

    #[test]
    fn files_the_question_in_the_human_inbox_with_its_session_then_blocks() {
        let inbox = RecordingInbox::default();
        let request = AskRequest {
            project_id: ProjectId::generate(),
            session_id: SessionId::generate(),
            run_id: RunId::generate(),
            question: "Which deployment window should I use?".into(),
        };

        let receipt = ask(&inbox, request.clone()).expect("ask is delivered");

        assert_eq!(receipt.delivery_id, Uuid::nil());
        assert_eq!(receipt.session_id, request.session_id);
        assert!(receipt.waiting);
        assert_eq!(*inbox.delivered.lock().unwrap(), vec![request.clone()]);
        assert_eq!(
            *inbox.waits.lock().unwrap(),
            vec![(request.run_id, "ask".into())]
        );
    }
}
