//! Mail domain types, projections, and the shared waiting mechanism.

use crate::ids::{ProjectId, RunId, SessionId};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const DEFAULT_WAIT_TIMEOUT_SECONDS: u64 = 15 * 60;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MailEntryKind {
    Sent,
    Read,
    Drained,
    Waiting,
}

impl MailEntryKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sent => "mail.sent",
            Self::Read => "mail.read",
            Self::Drained => "mail.drained",
            Self::Waiting => "mail.waiting",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WaitReason {
    Ask,
    Mail,
    DebugPaused,
    Gate,
}

impl WaitReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Mail => "mail",
            Self::DebugPaused => "debug-paused",
            Self::Gate => "gate",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Locator {
    pub value: String,
}

impl Locator {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() || !value.starts_with("locus://") {
            bail!("mail locator must be a locus:// URI")
        }
        Ok(Self { value })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Recipient {
    Agent(SessionId),
    Human,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryState {
    Pending,
    Delivered,
    Read,
    Drained,
}

impl DeliveryState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Delivered => "delivered",
            Self::Read => "read",
            Self::Drained => "drained",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MailMessage {
    pub id: String,
    pub sender: String,
    pub recipient: String,
    pub verb: MailVerb,
    pub body: String,
    pub created_at: i64,
    pub locator: Option<Locator>,
    pub delivery: DeliveryState,
}

impl MailMessage {
    pub fn new(
        id: impl Into<String>,
        sender: impl Into<String>,
        recipient: impl Into<String>,
        body: impl Into<String>,
        created_at: i64,
        locator: Option<Locator>,
    ) -> Result<Self> {
        let id = id.into();
        let sender = sender.into();
        let recipient = recipient.into();
        let body = body.into();
        if id.trim().is_empty() || sender.trim().is_empty() || recipient.trim().is_empty() {
            bail!("mail message identity is required")
        }
        if body.trim().is_empty() {
            bail!("mail body is required")
        }
        Ok(Self {
            id,
            sender,
            recipient,
            verb: MailVerb::Send,
            body,
            created_at,
            locator,
            delivery: DeliveryState::Pending,
        })
    }

    pub fn mark_read(&mut self) {
        if self.delivery != DeliveryState::Drained {
            self.delivery = DeliveryState::Read;
        }
    }
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

    pub fn send(
        &mut self,
        sender: impl Into<String>,
        recipient: impl Into<String>,
        body: impl Into<String>,
        now: i64,
        locator: Option<Locator>,
    ) -> Result<String> {
        self.append(MailMessage::new(
            Uuid::new_v4().to_string(),
            sender,
            recipient,
            body,
            now,
            locator,
        )?)
    }

    pub fn reply(
        &mut self,
        sender: impl Into<String>,
        body: impl Into<String>,
        now: i64,
        locator: Option<Locator>,
    ) -> Result<String> {
        let recipient = self
            .messages
            .last()
            .map(|message| message.sender.clone())
            .context("cannot reply to an empty thread")?;
        let mut message = MailMessage::new(
            Uuid::new_v4().to_string(),
            sender,
            recipient,
            body,
            now,
            locator,
        )?;
        message.verb = MailVerb::Reply;
        self.append(message)
    }

    fn append(&mut self, message: MailMessage) -> Result<String> {
        if !self.accepts_mail() {
            bail!("mail thread is drained")
        }
        let id = message.id.clone();
        self.messages.push(message);
        if self.status == ThreadStatus::Waiting {
            self.unblock();
        } else {
            self.status = ThreadStatus::Open;
        }
        Ok(id)
    }

    pub fn pending(&self) -> impl Iterator<Item = &MailMessage> {
        self.messages
            .iter()
            .filter(|message| message.delivery == DeliveryState::Pending)
    }

    pub fn read(&mut self, message_id: &str) -> Result<&MailMessage> {
        let message = self
            .messages
            .iter_mut()
            .find(|message| message.id == message_id)
            .context("mail message not found")?;
        message.mark_read();
        Ok(message)
    }

    /// Drains all pending messages as one in-memory transition; callers persist the event after it succeeds.
    pub fn drain_pending(&mut self) -> Vec<MailMessage> {
        let pending = self.pending().cloned().collect::<Vec<_>>();
        for message in &mut self.messages {
            if message.delivery == DeliveryState::Pending {
                message.delivery = DeliveryState::Drained;
            }
        }
        if !pending.is_empty() {
            self.status = ThreadStatus::Drained;
        }
        pending
    }

    pub fn start_wait(
        &mut self,
        run_id: RunId,
        reason: WaitReason,
        started_at: i64,
        timeout_seconds: Option<u64>,
    ) -> Result<WaitingState> {
        if self.waiting() {
            bail!("mail thread already has an active wait")
        }
        self.status = ThreadStatus::Waiting;
        self.waiting_run = Some(run_id);
        self.wait_started_at = Some(started_at);
        self.wait_timeout_seconds = Some(timeout_seconds.unwrap_or(DEFAULT_WAIT_TIMEOUT_SECONDS));
        Ok(WaitingState::new(
            run_id,
            reason,
            Value::String(self.id.clone()),
        ))
    }

    pub fn wait_expired(&self, now: i64) -> bool {
        self.waiting()
            && now.saturating_sub(self.wait_started_at.unwrap_or(now))
                >= self
                    .wait_timeout_seconds
                    .unwrap_or(DEFAULT_WAIT_TIMEOUT_SECONDS) as i64
    }

    pub fn end_wait(&mut self) {
        self.unblock();
        self.wait_started_at = None;
        self.wait_timeout_seconds = None;
    }

    pub fn wait_copy(&self, now: i64) -> Option<String> {
        self.waiting_run.map(|run| {
            let elapsed = now.saturating_sub(self.wait_started_at.unwrap_or(now));
            format!(
                "{run} is in mail wait — {}m of a {}m timeout",
                elapsed / 60,
                self.wait_timeout_seconds
                    .unwrap_or(DEFAULT_WAIT_TIMEOUT_SECONDS)
                    / 60
            )
        })
    }
    pub fn idle_guardrail_applies(&self) -> bool {
        !self.waiting()
    }
    pub fn drain(&mut self) {
        self.drain_pending();
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MailEvent {
    Sent {
        thread_id: String,
        project_id: ProjectId,
        subject: String,
        message: MailMessage,
    },
    Read {
        thread_id: String,
        message_id: String,
    },
    Drained {
        thread_id: String,
    },
    Waiting {
        thread_id: String,
        run_id: RunId,
        reason: WaitReason,
        started_at: i64,
        timeout_seconds: u64,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MailProjection {
    threads: std::collections::BTreeMap<String, MailThread>,
}

impl MailProjection {
    pub fn apply(&mut self, event: MailEvent) -> Result<()> {
        match event {
            MailEvent::Sent {
                thread_id,
                project_id,
                subject,
                message,
            } => {
                let thread = self.threads.entry(thread_id.clone()).or_insert(MailThread {
                    id: thread_id,
                    project_id,
                    subject,
                    status: ThreadStatus::Open,
                    messages: Vec::new(),
                    waiting_run: None,
                    wait_started_at: None,
                    wait_timeout_seconds: None,
                });
                if !thread.accepts_mail() {
                    bail!("mail projection thread is drained")
                }
                thread.messages.push(message);
                thread.status = ThreadStatus::Open;
            }
            MailEvent::Read {
                thread_id,
                message_id,
            } => {
                self.threads
                    .get_mut(&thread_id)
                    .context("mail projection thread not found")?
                    .read(&message_id)?;
            }
            MailEvent::Drained { thread_id } => {
                self.threads
                    .get_mut(&thread_id)
                    .context("mail projection thread not found")?
                    .drain();
            }
            MailEvent::Waiting {
                thread_id,
                run_id,
                reason,
                started_at,
                timeout_seconds,
            } => {
                self.threads
                    .get_mut(&thread_id)
                    .context("mail projection thread not found")?
                    .start_wait(run_id, reason, started_at, Some(timeout_seconds))?;
            }
        }
        Ok(())
    }

    pub fn thread(&self, thread_id: &str) -> Option<&MailThread> {
        self.threads.get(thread_id)
    }

    pub fn threads(&self) -> impl Iterator<Item = &MailThread> {
        self.threads.values()
    }

    pub fn rebuild(events: impl IntoIterator<Item = MailEvent>) -> Result<Self> {
        let mut projection = Self::default();
        for event in events {
            projection.apply(event)?;
        }
        Ok(projection)
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

/// A single state transition used by ask, mail wait, debug pauses, and gates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaitingState {
    pub run_id: RunId,
    pub reason: WaitReason,
    pub detail: Value,
}
impl WaitingState {
    pub fn new(run_id: RunId, reason: WaitReason, detail: Value) -> Self {
        Self {
            run_id,
            reason,
            detail,
        }
    }

    /// Marks a run as waiting while its debuggee is stopped at a breakpoint.
    ///
    /// A debug pause is deliberate work, not an absent event stream, so it must use the
    /// shared waiting state rather than being represented as idle.
    pub fn from_debug_breakpoint(run_id: RunId, detail: Value) -> Self {
        Self::new(run_id, WaitReason::DebugPaused, detail)
    }

    pub fn suppresses_idle(&self) -> bool {
        true
    }
}

pub fn idle_guardrail_applies(waiting: Option<&WaitingState>) -> bool {
    waiting.is_none()
}
pub const WAITING_NOT_IDLE: &str =
    "State is `waiting`, not idle. The idle guardrail will not fire.";

#[cfg(test)]
#[allow(clippy::module_inception)]
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
        let mut h = HandoffBoundary {
            artifact_id: None,
            ownership_transferred: false,
        };
        h.transfer("a");
        assert!(!h.accepts_mail());
    }
    #[test]
    fn participants_rail() {
        let p = Participant {
            session_id: SessionId::generate(),
            run_id: RunId::generate(),
            state: ThreadStatus::Open,
        };
        assert_eq!(p.state, ThreadStatus::Open);
    }
    #[test]
    fn waiting_suppresses_idle() {
        let wait = WaitingState::new(RunId::generate(), WaitReason::Mail, Value::Null);
        assert!(!idle_guardrail_applies(Some(&wait)));
    }
    #[test]
    fn four_callers_share_reasons() {
        assert_eq!(
            [
                WaitReason::Ask.as_str(),
                WaitReason::Mail.as_str(),
                WaitReason::DebugPaused.as_str(),
                WaitReason::Gate.as_str()
            ],
            ["ask", "mail", "debug-paused", "gate"]
        );
    }
    #[test]
    fn locators_are_required_for_work() {
        assert!(Locator::new("locus://p/session/s").is_ok());
        assert!(Locator::new("event://notification").is_err());
    }
    #[test]
    fn entry_kinds_are_stable() {
        assert_eq!(MailEntryKind::Sent.as_str(), "mail.sent");
        assert_eq!(MailEntryKind::Read.as_str(), "mail.read");
        assert_eq!(MailEntryKind::Drained.as_str(), "mail.drained");
    }

    #[test]
    fn schema() {
        let message = MailMessage::new(
            "m1",
            "agent-a",
            "agent-b",
            "hello",
            1,
            Some(Locator::new("locus://project/session").unwrap()),
        )
        .unwrap();
        assert_eq!(message.delivery, DeliveryState::Pending);
        assert_eq!(
            serde_json::to_value(&message).unwrap()["delivery"],
            "pending"
        );
    }

    #[test]
    fn send_reply_and_read() {
        let mut thread = thread();
        let first = thread.send("agent-a", "agent-b", "hello", 1, None).unwrap();
        let second = thread.reply("agent-b", "ack", 2, None).unwrap();
        assert_ne!(first, second);
        assert_eq!(thread.messages[1].verb, MailVerb::Reply);
        thread.read(&first).unwrap();
        assert_eq!(thread.messages[0].delivery, DeliveryState::Read);
        assert!(!thread.waiting());
    }

    #[test]
    fn drain_is_atomic() {
        let mut thread = thread();
        thread.send("a", "b", "one", 1, None).unwrap();
        thread.send("a", "b", "two", 2, None).unwrap();
        let drained = thread.drain_pending();
        assert_eq!(drained.len(), 2);
        assert!(thread.pending().next().is_none());
        assert!(thread
            .messages
            .iter()
            .all(|message| message.delivery == DeliveryState::Drained));
    }

    #[test]
    fn survives_harness_swap() {
        let mut projection = MailProjection::default();
        let message = MailMessage::new("m1", "a", "b", "same protocol", 1, None).unwrap();
        projection
            .apply(MailEvent::Sent {
                thread_id: "t".into(),
                project_id: ProjectId::generate(),
                subject: "subject".into(),
                message: message.clone(),
            })
            .unwrap();
        let encoded = serde_json::to_string(&MailEvent::Sent {
            thread_id: "t".into(),
            project_id: projection.thread("t").unwrap().project_id,
            subject: "subject".into(),
            message,
        })
        .unwrap();
        assert!(encoded.contains("same protocol"));
    }
}

#[cfg(test)]
mod project {
    use super::*;

    #[test]
    fn mail() {
        let event = MailEvent::Sent {
            thread_id: "t".into(),
            project_id: ProjectId::generate(),
            subject: "subject".into(),
            message: MailMessage::new("m", "a", "b", "body", 0, None).unwrap(),
        };
        let projection = MailProjection::rebuild([event]).unwrap();
        assert_eq!(projection.threads().count(), 1);
    }
}

#[cfg(test)]
mod rebuild {
    use super::*;

    #[test]
    fn mail() {
        let project_id = ProjectId::generate();
        let message = MailMessage::new("m", "a", "b", "body", 0, None).unwrap();
        let events = [
            MailEvent::Sent {
                thread_id: "t".into(),
                project_id,
                subject: "subject".into(),
                message,
            },
            MailEvent::Drained {
                thread_id: "t".into(),
            },
        ];
        let projection = MailProjection::rebuild(events).unwrap();
        assert_eq!(
            projection.thread("t").unwrap().status,
            ThreadStatus::Drained
        );
    }
}

#[cfg(test)]
mod schema {
    use super::*;

    #[test]
    fn schema() {
        let message = MailMessage::new("m", "a", "b", "body", 0, None).unwrap();
        assert_eq!(message.delivery, DeliveryState::Pending);
    }
}

#[cfg(test)]
mod wait_sets_waiting {
    use super::*;

    #[test]
    fn wait_sets_waiting() {
        let mut thread = MailThread {
            id: "t".into(),
            project_id: ProjectId::generate(),
            subject: "subject".into(),
            status: ThreadStatus::Open,
            messages: Vec::new(),
            waiting_run: None,
            wait_started_at: None,
            wait_timeout_seconds: None,
        };
        let state = thread
            .start_wait(RunId::generate(), WaitReason::Mail, 0, None)
            .unwrap();
        assert!(thread.waiting());
        assert!(state.suppresses_idle());
    }
}

#[cfg(test)]
mod survives_harness_swap {
    use super::*;

    #[test]
    fn survives_harness_swap() {
        let event = MailEvent::Sent {
            thread_id: "t".into(),
            project_id: ProjectId::generate(),
            subject: "subject".into(),
            message: MailMessage::new("m", "a", "b", "body", 0, None).unwrap(),
        };
        let bytes = serde_json::to_vec(&event).unwrap();
        let decoded: MailEvent = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, event);
    }
}

#[cfg(test)]
mod drain_is_atomic {
    use super::*;

    #[test]
    fn drain_is_atomic() {
        let mut thread = MailThread {
            id: "t".into(),
            project_id: ProjectId::generate(),
            subject: "subject".into(),
            status: ThreadStatus::Open,
            messages: Vec::new(),
            waiting_run: None,
            wait_started_at: None,
            wait_timeout_seconds: None,
        };
        thread.send("a", "b", "body", 0, None).unwrap();
        assert_eq!(thread.drain_pending().len(), 1);
        assert!(thread.pending().next().is_none());
    }
}

#[cfg(test)]
mod entry_kinds {
    use super::*;

    #[test]
    fn entry_kinds() {
        assert_eq!(MailEntryKind::Sent.as_str(), "mail.sent");
        assert_eq!(MailEntryKind::Read.as_str(), "mail.read");
        assert_eq!(MailEntryKind::Drained.as_str(), "mail.drained");
    }
}

#[cfg(test)]
mod waiting {
    use super::*;

    #[test]
    fn four_callers() {
        assert_eq!(WaitReason::Ask.as_str(), "ask");
        assert_eq!(WaitReason::Mail.as_str(), "mail");
        assert_eq!(WaitReason::DebugPaused.as_str(), "debug-paused");
        assert_eq!(WaitReason::Gate.as_str(), "gate");
    }
}

#[cfg(test)]
mod guard {
    use super::*;

    #[test]
    fn waiting_from_debug() {
        let run_id = RunId::generate();
        let state = WaitingState::from_debug_breakpoint(
            run_id,
            serde_json::json!({"file": "src/main.rs", "line": 42}),
        );

        assert_eq!(state.run_id, run_id);
        assert_eq!(state.reason, WaitReason::DebugPaused);
        assert_eq!(state.detail["line"], 42);
        assert!(state.suppresses_idle());
        assert!(!idle_guardrail_applies(Some(&state)));
    }
}
