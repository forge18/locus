//! Human inbox projection for actionable mail and run blockers.

use super::mail::Locator;
use crate::ids::{ProjectId, RunId, SessionId};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InboxKind {
    Ask,
    Gate,
    Mail,
    Finding,
    Aborted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InboxItem {
    pub id: String,
    pub project_id: ProjectId,
    pub session_id: SessionId,
    pub run_id: Option<RunId>,
    pub kind: InboxKind,
    pub title: String,
    pub body: String,
    pub locator: Locator,
}

impl InboxItem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        project_id: ProjectId,
        session_id: SessionId,
        run_id: Option<RunId>,
        kind: InboxKind,
        title: impl Into<String>,
        body: impl Into<String>,
        locator: Locator,
    ) -> Result<Self> {
        let id = id.into();
        let title = title.into();
        let body = body.into();
        if id.trim().is_empty() || title.trim().is_empty() || body.trim().is_empty() {
            bail!("inbox item identity and body are required")
        }
        Ok(Self {
            id,
            project_id,
            session_id,
            run_id,
            kind,
            title,
            body,
            locator,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Notification {
    pub title: String,
    pub body: String,
}

/// Items without a locator are informational notifications, never actionable inbox work.
pub fn actionable_or_notification(
    id: impl Into<String>,
    project_id: ProjectId,
    session_id: SessionId,
    kind: InboxKind,
    title: impl Into<String>,
    body: impl Into<String>,
    locator: Option<Locator>,
) -> Result<std::result::Result<InboxItem, Notification>> {
    let title = title.into();
    let body = body.into();
    let Some(locator) = locator else {
        return Ok(Err(Notification { title, body }));
    };
    Ok(Ok(InboxItem::new(
        id, project_id, session_id, None, kind, title, body, locator,
    )?))
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InboxProjection {
    pending: Vec<InboxItem>,
}

impl InboxProjection {
    pub fn add(&mut self, item: InboxItem) {
        self.pending.push(item);
    }

    pub fn pending(&self) -> &[InboxItem] {
        &self.pending
    }

    pub fn for_session(&self, session_id: SessionId) -> Vec<&InboxItem> {
        self.pending
            .iter()
            .filter(|item| item.session_id == session_id)
            .collect()
    }
}

#[cfg(test)]
mod human_is_a_participant {
    use super::*;

    #[test]
    fn human_is_a_participant() {
        let item = InboxItem::new(
            "ask-1",
            ProjectId::generate(),
            SessionId::generate(),
            Some(RunId::generate()),
            InboxKind::Ask,
            "Question",
            "Which target?",
            Locator::new("locus://project/session").unwrap(),
        )
        .unwrap();
        assert_eq!(item.kind, InboxKind::Ask);
    }
}

#[cfg(test)]
mod items_resolve {
    use super::*;

    #[test]
    fn items_resolve() {
        let item = InboxItem::new(
            "gate-1",
            ProjectId::generate(),
            SessionId::generate(),
            None,
            InboxKind::Gate,
            "Review",
            "Review the diff",
            Locator::new("locus://project/artifact/gate-1").unwrap(),
        )
        .unwrap();
        assert!(item.locator.value.starts_with("locus://"));
    }
}

#[cfg(test)]
mod notifications_are_not_inbox_work {
    use super::*;

    #[test]
    fn notifications_are_not_inbox_work() {
        let result = actionable_or_notification(
            "notice-1",
            ProjectId::generate(),
            SessionId::generate(),
            InboxKind::Finding,
            "Observed",
            "Nothing needs action",
            None,
        )
        .unwrap();
        assert!(matches!(result, Err(Notification { .. })));
    }
}

#[cfg(test)]
mod silence_is_the_default {
    use super::*;

    #[test]
    fn silence_is_the_default() {
        assert!(InboxProjection::default().pending().is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locator() -> Locator {
        Locator::new("locus://project/panel/session").unwrap()
    }

    #[test]
    fn human_is_a_participant() {
        let item = InboxItem::new(
            "ask-1",
            ProjectId::generate(),
            SessionId::generate(),
            Some(RunId::generate()),
            InboxKind::Ask,
            "Question",
            "Which target?",
            locator(),
        )
        .unwrap();
        assert_eq!(item.kind, InboxKind::Ask);
        assert!(item.locator.value.starts_with("locus://"));
    }

    #[test]
    fn items_resolve() {
        let item = InboxItem::new(
            "gate-1",
            ProjectId::generate(),
            SessionId::generate(),
            None,
            InboxKind::Gate,
            "Review",
            "Review the diff",
            locator(),
        )
        .unwrap();
        assert!(!item.locator.value.is_empty());
    }

    #[test]
    fn notifications_are_not_inbox_work() {
        let result = actionable_or_notification(
            "notice-1",
            ProjectId::generate(),
            SessionId::generate(),
            InboxKind::Finding,
            "Observed",
            "Nothing needs action",
            None,
        )
        .unwrap();
        assert!(matches!(result, Err(Notification { .. })));
    }

    #[test]
    fn silence_is_the_default() {
        let projection = InboxProjection::default();
        assert!(projection.pending().is_empty());
        assert!(projection.for_session(SessionId::generate()).is_empty());
    }
}
