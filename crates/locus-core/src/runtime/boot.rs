//! Boot reconciliation: on start, runs marked `running` are checked against Docker
//! rather than trusted indefinitely. PLAN.md §Process topology, "Every start reconciles".
//!
//! Collapsed here from `sandbox`, which carried a second `RunStatus` and a second
//! reconciliation of its own.

use anyhow::{bail, Result};

use crate::{
    ids::RunId,
    runtime::{
        container::PtyStream,
        run::{Inbox, InboxItem},
        session::{Run, RunStatus},
    },
    services::telemetry::{CapturedEvent, Event, EventCollector},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunReconciliation {
    pub run_id: RunId,
    pub status: RunStatus,
    pub reattach: bool,
}

/// On boot, running rows are reconciled with Docker rather than trusted indefinitely.
pub fn reconcile_on_boot(
    running: impl IntoIterator<Item = (RunId, bool)>,
) -> Vec<RunReconciliation> {
    running
        .into_iter()
        .map(|(run_id, container_alive)| RunReconciliation {
            run_id,
            status: if container_alive {
                RunStatus::Running
            } else {
                RunStatus::Aborted
            },
            reattach: container_alive,
        })
        .collect()
}

/// File the human-visible follow-up for a run that was aborted during boot reconciliation.
pub fn file_aborted_run_inbox_item(run: &Run, inbox: &mut impl Inbox) -> Result<()> {
    if run.status != RunStatus::Aborted {
        bail!("only aborted runs file a boot reconciliation inbox item")
    }
    inbox.file(InboxItem {
        session_id: run.session_id,
        run_id: run.id,
        body: "Run aborted because its container was gone when locusd restarted.".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconciles_on_boot() {
        let alive = RunId::generate();
        let gone = RunId::generate();
        assert_eq!(
            reconcile_on_boot([(alive, true), (gone, false)]),
            [
                RunReconciliation {
                    run_id: alive,
                    status: RunStatus::Running,
                    reattach: true
                },
                RunReconciliation {
                    run_id: gone,
                    status: RunStatus::Aborted,
                    reattach: false
                },
            ]
        );
    }
}

/// The minimal runtime view needed to reconcile runs after `locusd` restarts.
pub trait BootRuntime {
    fn container_is_alive(&mut self, container: &str) -> Result<bool>;
    fn reattach_pty(&mut self, container: &str, stream: PtyStream) -> Result<()>;
}

/// Result of reconciling one persisted running run at daemon boot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootReconciliation {
    Reattached,
    Missing,
}

/// Reattach to containers that survived a daemon restart. Missing containers are handled by
/// `abort_missing_on_boot`, which additionally records their terminal event and inbox item.
pub fn reattach_on_boot(
    run: &Run,
    runtime: &mut impl BootRuntime,
    stream: PtyStream,
) -> Result<BootReconciliation> {
    if run.status != RunStatus::Running {
        bail!("only running runs are reconciled at boot")
    }
    let container = format!("locus-agent-{}", run.id);
    if !runtime.container_is_alive(&container)? {
        return Ok(BootReconciliation::Missing);
    }
    runtime.reattach_pty(&container, stream)?;
    Ok(BootReconciliation::Reattached)
}

/// Close a persisted running row whose container disappeared while the daemon was down.
pub fn abort_missing_on_boot(run: &mut Run, collector: &EventCollector) -> Result<Event> {
    if run.status != RunStatus::Running {
        bail!("only running runs may be aborted on boot")
    }
    run.status = RunStatus::Aborted;
    let event = collector.capture(
        run.id,
        CapturedEvent {
            verb: crate::services::telemetry::EventVerb::Aborted,
            ts: time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .expect("RFC3339 timestamp"),
            text: Some("container missing during boot reconciliation".into()),
            tool: None,
            args: None,
            usage: None,
            raw: serde_json::json!({"reason": "container_missing_on_boot"}),
        },
    );
    run.events.push(event.clone());
    Ok(event)
}

#[cfg(test)]
mod abort_reaches_inbox {
    use crate::ids::{RunId, SessionId};
    use anyhow::Result;

    use super::*;
    use crate::runtime::run::{Inbox, InboxItem};
    use crate::runtime::session::{Artifact, Run, RunStatus};

    #[derive(Default)]
    struct RecordingInbox(Vec<InboxItem>);

    impl Inbox for RecordingInbox {
        fn file(&mut self, item: InboxItem) -> Result<()> {
            self.0.push(item);
            Ok(())
        }
    }

    #[test]
    fn files_an_inbox_item_for_an_orphaned_run() {
        let run = Run {
            id: RunId::generate(),
            session_id: SessionId::generate(),
            resolved_model_id: "test-model".into(),
            status: RunStatus::Aborted,
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: Vec::<Artifact>::new(),
        };
        let mut inbox = RecordingInbox::default();

        file_aborted_run_inbox_item(&run, &mut inbox).expect("file notification");

        assert_eq!(inbox.0.len(), 1);
        assert_eq!(inbox.0[0].session_id, run.session_id);
        assert_eq!(inbox.0[0].run_id, run.id);
    }
}

#[cfg(test)]
mod aborts_orphans {
    use crate::ids::{RunId, SessionId};

    use super::abort_missing_on_boot;
    use crate::{
        runtime::session::{Artifact, Run, RunStatus},
        services::telemetry::{EventCollector, EventVerb},
    };

    #[test]
    fn marks_a_missing_container_aborted_and_emits_a_terminal_event() {
        let mut run = Run {
            id: RunId::generate(),
            session_id: SessionId::generate(),
            resolved_model_id: "test-model".into(),
            status: RunStatus::Running,
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: Vec::<Artifact>::new(),
        };
        let collector = EventCollector::new(1);

        let event = abort_missing_on_boot(&mut run, &collector).expect("abort orphan");

        assert_eq!(run.status, RunStatus::Aborted);
        assert_eq!(event.verb, EventVerb::Aborted);
        assert_eq!(run.events, vec![event]);
    }
}

#[cfg(test)]
mod reattach_on_boot {
    use crate::ids::{RunId, SessionId};
    use anyhow::Result;

    use super::{reattach_on_boot, BootReconciliation, BootRuntime, PtyStream};
    use crate::runtime::session::{Artifact, Run, RunStatus};

    struct RecordingRuntime {
        attached: Option<String>,
    }

    impl BootRuntime for RecordingRuntime {
        fn container_is_alive(&mut self, _: &str) -> Result<bool> {
            Ok(true)
        }

        fn reattach_pty(&mut self, container: &str, _: PtyStream) -> Result<()> {
            self.attached = Some(container.into());
            Ok(())
        }
    }

    #[test]
    fn reattaches_to_a_container_that_survived_daemon_boot() {
        let run = Run {
            id: RunId::generate(),
            session_id: SessionId::generate(),
            resolved_model_id: "test-model".into(),
            status: RunStatus::Running,
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: Vec::<Artifact>::new(),
        };
        let mut runtime = RecordingRuntime { attached: None };

        let result = reattach_on_boot(&run, &mut runtime, PtyStream::new(1)).expect("reconcile");

        assert_eq!(result, BootReconciliation::Reattached);
        assert_eq!(runtime.attached, Some(format!("locus-agent-{}", run.id)));
    }
}
