//! Human escalation owned by the core rather than the in-container CLI.

use anyhow::{bail, Result};
use uuid::Uuid;

/// The durable context required to deliver an agent's question to a human.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskRequest {
    pub project_id: Uuid,
    pub session_id: Uuid,
    pub run_id: Uuid,
    pub question: String,
}

/// The persisted result returned to the caller after it is blocked for an answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskReceipt {
    pub delivery_id: Uuid,
    pub session_id: Uuid,
    pub waiting: bool,
}

/// Storage boundary for the human inbox and the run's active wait.
pub trait HumanInbox {
    fn deliver_question(&self, request: &AskRequest) -> Result<Uuid>;
    fn mark_waiting(&self, run_id: Uuid, reason: &str) -> Result<()>;
}

/// Files an inbox item attached to the originating session, then blocks its run.
pub fn ask(inbox: &impl HumanInbox, request: AskRequest) -> Result<AskReceipt> {
    if request.question.trim().is_empty() {
        bail!("ask question must not be empty")
    }

    let delivery_id = inbox.deliver_question(&request)?;
    inbox.mark_waiting(request.run_id, "ask")?;
    Ok(AskReceipt {
        delivery_id,
        session_id: request.session_id,
        waiting: true,
    })
}

#[cfg(test)]
mod reaches_inbox {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct RecordingInbox {
        delivered: Mutex<Vec<AskRequest>>,
        waits: Mutex<Vec<(Uuid, String)>>,
    }

    impl HumanInbox for RecordingInbox {
        fn deliver_question(&self, request: &AskRequest) -> Result<Uuid> {
            self.delivered.lock().unwrap().push(request.clone());
            Ok(Uuid::nil())
        }

        fn mark_waiting(&self, run_id: Uuid, reason: &str) -> Result<()> {
            self.waits.lock().unwrap().push((run_id, reason.into()));
            Ok(())
        }
    }

    #[test]
    fn files_the_question_in_the_human_inbox_with_its_session_then_blocks() {
        let inbox = RecordingInbox::default();
        let request = AskRequest {
            project_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
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
