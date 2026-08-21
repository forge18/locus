//! Spawn one configured agent container for a queued run.

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use serde_json::Value;
use tokio::sync::broadcast;

use crate::{
    materialize::{materialize, ExtensionSet, MaterializationReport, MaterializedTree, PluginHost},
    registry::HarnessDefinition,
    sandbox::{
        agent_image_tag, agent_mounts, project_network, Mount, PortAllocator, PtyAttachment,
        ToolPin, AGENT_PTY,
    },
    session::{Run, RunStatus},
    store::Store,
    telemetry::{Adapter, CapturedEvent, Event, EventCollector},
};
use uuid::Uuid;

/// Normalize captured source records through the adapter selected for this run's telemetry source.
pub fn normalize(
    adapter: &dyn Adapter,
    records: impl IntoIterator<Item = Value>,
) -> Result<Vec<CapturedEvent>> {
    records
        .into_iter()
        .try_fold(Vec::new(), |mut events, record| {
            events.extend(adapter.normalize(record)?);
            Ok(events)
        })
}

/// Assign run-owned ordering and durably store every normalized event before returning it.
pub async fn persist_normalized_events(
    store: &Store,
    collector: &EventCollector,
    run_id: impl Into<String>,
    captured: impl IntoIterator<Item = CapturedEvent>,
) -> Result<Vec<Event>> {
    let run_id = run_id.into();
    let events = captured
        .into_iter()
        .map(|event| collector.capture(run_id.clone(), event))
        .collect::<Vec<_>>();

    for event in &events {
        store
            .persist_event(&Uuid::new_v4().to_string(), event)
            .await?;
    }

    Ok(events)
}

/// Normalize two live sources through the same collector without exposing their harness dialects
/// after the capture boundary.
pub async fn normalize_two_harnesses(
    collector: &EventCollector,
    first_run_id: impl Into<String>,
    first_adapter: &dyn Adapter,
    first_records: Vec<Value>,
    second_run_id: impl Into<String>,
    second_adapter: &dyn Adapter,
    second_records: Vec<Value>,
) -> Result<Vec<Event>> {
    let (first, second) = tokio::join!(async { normalize(first_adapter, first_records) }, async {
        normalize(second_adapter, second_records)
    },);
    let first_run_id = first_run_id.into();
    let second_run_id = second_run_id.into();
    let first = first?;
    let second = second?;
    Ok(first
        .into_iter()
        .map(|event| collector.capture(first_run_id.clone(), event))
        .chain(
            second
                .into_iter()
                .map(|event| collector.capture(second_run_id.clone(), event)),
        )
        .collect())
}

/// Core-owned context for a request received through one run's socket endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunContext {
    run_id: Uuid,
}

impl RunContext {
    pub fn new(run_id: Uuid) -> Self {
        Self { run_id }
    }
}

/// Read boundary for the state that a connected run may observe.
pub trait RunStateStore {
    fn read_run(&self, run_id: Uuid) -> Result<Run>;
}

/// Returns the state belonging to the socket's run context, never a caller-selected run.
pub fn own_state(store: &impl RunStateStore, context: RunContext) -> Result<Run> {
    store.read_run(context.run_id)
}

#[cfg(test)]
mod own_state_only {
    use std::collections::BTreeMap;

    use super::*;

    struct Runs(BTreeMap<Uuid, Run>);

    impl RunStateStore for Runs {
        fn read_run(&self, run_id: Uuid) -> Result<Run> {
            self.0
                .get(&run_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("run not found"))
        }
    }

    fn run(id: Uuid, model: &str) -> Run {
        Run {
            id,
            session_id: Uuid::new_v4(),
            resolved_model_id: model.into(),
            status: RunStatus::Running,
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            artifacts: vec![],
            native_session_id: None,
        }
    }

    #[test]
    fn reads_only_the_run_bound_to_the_socket_context() {
        let own_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let own = run(own_id, "own-model");
        let other = run(other_id, "other-model");
        let store = Runs(BTreeMap::from([(own_id, own.clone()), (other_id, other)]));

        assert_eq!(
            own_state(&store, RunContext::new(own_id)).expect("read own run"),
            own
        );
    }
}

/// Whether the container runtime built the image or reused its existing cache entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageDisposition {
    Built,
    Reused,
}

const PTY_STREAM_CAPACITY: usize = 1_024;

/// Broadcasts raw PTY bytes from a run's container runtime to its UI subscribers.
#[derive(Clone, Debug)]
pub struct PtyStream {
    sender: broadcast::Sender<Vec<u8>>,
}

impl PtyStream {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Registers one UI consumer. The desktop forwards each received buffer through
    /// its `Channel<&[u8]>` transport.
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<u8>> {
        self.sender.subscribe()
    }

    /// Delivers one byte buffer read from the attached PTY.
    pub fn write(&self, bytes: &[u8]) -> usize {
        self.sender.send(bytes.to_vec()).unwrap_or(0)
    }
}

impl PartialEq for PtyStream {
    fn eq(&self, other: &Self) -> bool {
        self.sender.same_channel(&other.sender)
    }
}

impl Eq for PtyStream {}

/// The complete, harness-agnostic request made to the container runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerLaunch {
    pub name: String,
    pub image: String,
    pub command: Vec<String>,
    pub entrypoint: String,
    pub environment: Vec<String>,
    pub mounts: Vec<Mount>,
    pub network: String,
}

/// The narrow container boundary required by run spawning.
///
/// The supplied container adapter owns image caching, container creation, and PTY plumbing; this
/// supervisor owns their ordering and the run state transition.
pub trait ContainerRuntime {
    fn build_or_reuse_image(&mut self, image: &str) -> Result<ImageDisposition>;
    fn start_container(&mut self, container: &ContainerLaunch) -> Result<()>;
    fn attach_pty(
        &mut self,
        container: &str,
        attachment: PtyAttachment,
        stream: PtyStream,
    ) -> Result<()>;
    fn stop_container(&mut self, container: &str) -> Result<()>;
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
        run.id.to_string(),
        CapturedEvent {
            verb: crate::telemetry::EventVerb::Aborted,
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

/// An inbox notification generated by run supervision rather than by a harness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InboxItem {
    pub session_id: Uuid,
    pub run_id: Uuid,
    pub body: String,
}

/// The small persistence boundary the supervisor needs from the inbox service.
pub trait Inbox {
    fn file(&mut self, item: InboxItem) -> Result<()>;
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

/// A human-owned shell shares the pane plumbing but deliberately has no run identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanTerminal {
    pub pty: PtyStream,
}

impl HumanTerminal {
    pub fn open() -> Self {
        Self {
            pty: PtyStream::new(PTY_STREAM_CAPACITY),
        }
    }
}

/// Inputs owned by the caller for one queued run.
pub struct SpawnRequest<'a> {
    pub project_id: &'a str,
    pub harness: &'a HarnessDefinition,
    pub extensions: &'a ExtensionSet,
    pub config_root: PathBuf,
    pub socket_source: PathBuf,
    /// Per-run capability validated by the daemon socket before it routes any agent request.
    pub run_nonce: String,
    pub base_image_digest: String,
    pub tools: Vec<ToolPin>,
    pub plugin: Option<&'a PluginHost>,
}

/// The started container and the materialized configuration used for its prompt prefix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnedRun {
    pub container: ContainerLaunch,
    pub config: MaterializedTree,
    pub materialization: MaterializationReport,
    pub image: String,
    pub image_disposition: ImageDisposition,
    pub port: u16,
    pub pty_stream: PtyStream,
}

/// Materialize the run configuration, ensure its agent image, then start and attach its PTY.
pub fn spawn(
    run: &mut Run,
    request: SpawnRequest<'_>,
    ports: &PortAllocator,
    runtime: &mut impl ContainerRuntime,
) -> Result<SpawnedRun> {
    if run.status != RunStatus::Queued {
        bail!("only queued runs may be spawned")
    }

    let (config, materialization) = materialize(
        request.harness,
        request.extensions,
        &request.config_root,
        request.plugin,
    )
    .context("materialize run configuration")?;
    config
        .write_to(&request.config_root)
        .context("write run configuration")?;

    if request.run_nonce.trim().is_empty() {
        bail!("run socket capability nonce is required")
    }

    let image = agent_image_tag(&request.base_image_digest, &request.tools);
    let image_disposition = runtime
        .build_or_reuse_image(&image)
        .context("build or reuse agent image")?;
    let port = ports.allocate()?;
    let container = ContainerLaunch {
        name: format!("locus-agent-{}", run.id),
        image: image.clone(),
        command: std::iter::once(request.harness.binary.clone())
            .chain(request.harness.launch.argv.iter().cloned())
            .collect(),
        entrypoint: crate::sandbox::entrypoint_setup().into(),
        environment: vec![
            format!("LOCUS_PORT={port}"),
            format!("LOCUS_RUN_NONCE={}", request.run_nonce),
        ],
        mounts: agent_mounts(
            request.socket_source.display().to_string(),
            request.config_root.display().to_string(),
        )
        .to_vec(),
        network: project_network(request.project_id),
    };
    if let Err(error) = runtime.start_container(&container) {
        ports.release(port);
        return Err(error).context("start agent container");
    }
    let pty_stream = PtyStream::new(PTY_STREAM_CAPACITY);
    if let Err(error) = runtime.attach_pty(&container.name, AGENT_PTY, pty_stream.clone()) {
        ports.release(port);
        return Err(error).context("attach agent PTY");
    }

    run.status = RunStatus::Running;
    Ok(SpawnedRun {
        container,
        config,
        materialization,
        image,
        image_disposition,
        port,
        pty_stream,
    })
}

/// Stop a running agent container and retain the caller's cancellation reason on its run.
/// Tracks a cooperative pause request. A pause is applied only between turns; the
/// container remains running so its state can be inspected and resumed safely.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PauseController {
    requested: bool,
}

impl PauseController {
    pub fn request(&mut self, run: &Run) -> Result<()> {
        if run.status != RunStatus::Running {
            bail!("only running runs may be paused")
        }
        self.requested = true;
        Ok(())
    }

    /// Returns true when the just-finished turn put the run on hold.
    pub fn after_turn(&mut self, run: &mut Run) -> bool {
        if !self.requested {
            return false;
        }
        self.requested = false;
        run.status = RunStatus::Paused;
        true
    }
}

/// Store the optional identifier the active harness uses for its native conversation.
/// Locus resume remains event-based, so callers may omit this for harnesses without one.
pub fn record_native_session_id(run: &mut Run, native_session_id: impl Into<String>) -> Result<()> {
    let native_session_id = native_session_id.into();
    if native_session_id.trim().is_empty() {
        bail!("native session id must not be empty")
    }
    run.native_session_id = Some(native_session_id);
    Ok(())
}

pub fn cancel(
    run: &mut Run,
    reason: impl AsRef<str>,
    runtime: &mut impl ContainerRuntime,
) -> Result<()> {
    if run.status != RunStatus::Running {
        bail!("only running runs may be cancelled")
    }

    let reason = reason.as_ref();
    if reason.trim().is_empty() {
        bail!("cancellation reason must not be empty")
    }

    runtime
        .stop_container(&format!("locus-agent-{}", run.id))
        .context("stop agent container")?;
    run.status = RunStatus::Cancelled;
    run.cancel_reason = Some(reason.into());
    Ok(())
}

#[cfg(test)]
mod two_harnesses_concurrent {
    use serde_json::json;

    use super::normalize_two_harnesses;
    use crate::telemetry::{EventCollector, EventVerb, StreamJsonAdapter};

    #[tokio::test]
    async fn concurrent_harnesses_emit_the_same_downstream_event_shape() {
        let first = StreamJsonAdapter::new("type", [("message", EventVerb::Assistant)]);
        let second = StreamJsonAdapter::new("kind", [("reply", EventVerb::Assistant)]);
        let collector = EventCollector::new(4);

        let events = normalize_two_harnesses(
            &collector,
            "first-run",
            &first,
            vec![json!({"type": "message", "text": "one"})],
            "second-run",
            &second,
            vec![json!({"kind": "reply", "text": "two"})],
        )
        .await
        .expect("both sources normalize");

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].verb, EventVerb::Assistant);
        assert_eq!(events[1].verb, EventVerb::Assistant);
        assert_eq!(events[0].tool, events[1].tool);
        assert_eq!(events[0].args, events[1].args);
    }
}

#[cfg(test)]
mod human_terminal_is_not_a_session {
    use super::HumanTerminal;

    #[tokio::test]
    async fn human_shell_has_pty_bytes_but_no_run_or_cost_state() {
        let terminal = HumanTerminal::open();
        let mut ui = terminal.pty.subscribe();

        terminal.pty.write(b"human command output");

        assert_eq!(
            ui.recv().await.expect("terminal bytes"),
            b"human command output"
        );
        // HumanTerminal deliberately contains no Session, Run, Event, or Usage fields.
    }
}

#[cfg(test)]
mod abort_reaches_inbox {
    use anyhow::Result;
    use uuid::Uuid;

    use super::{file_aborted_run_inbox_item, Inbox, InboxItem};
    use crate::session::{Artifact, Run, RunStatus};

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
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
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
    use uuid::Uuid;

    use super::abort_missing_on_boot;
    use crate::{
        session::{Artifact, Run, RunStatus},
        telemetry::{EventCollector, EventVerb},
    };

    #[test]
    fn marks_a_missing_container_aborted_and_emits_a_terminal_event() {
        let mut run = Run {
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
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
    use anyhow::Result;
    use uuid::Uuid;

    use super::{reattach_on_boot, BootReconciliation, BootRuntime, PtyStream};
    use crate::session::{Artifact, Run, RunStatus};

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
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
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

#[cfg(test)]
mod native_session_id {
    use uuid::Uuid;

    use super::record_native_session_id;
    use crate::session::{Artifact, Run, RunStatus};

    #[test]
    fn retains_a_harness_session_id_only_on_the_run_that_received_it() {
        let mut run = Run {
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            resolved_model_id: "test-model".into(),
            status: RunStatus::Running,
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: Vec::<Artifact>::new(),
        };

        record_native_session_id(&mut run, "harness-session-42").expect("store harness id");

        assert_eq!(run.native_session_id.as_deref(), Some("harness-session-42"));
    }
}

#[cfg(test)]
mod pause_holds_not_freezes {
    use uuid::Uuid;

    use super::PauseController;
    use crate::session::{Artifact, Run, RunStatus};

    fn running_run() -> Run {
        Run {
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            resolved_model_id: "test-model".into(),
            status: RunStatus::Running,
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: Vec::<Artifact>::new(),
        }
    }

    #[test]
    fn pauses_after_the_current_turn_without_stopping_the_container() {
        let mut run = running_run();
        let mut pause = PauseController::default();

        pause.request(&run).expect("request pause");
        assert_eq!(run.status, RunStatus::Running, "the current turn continues");
        assert!(pause.after_turn(&mut run), "next turn is held");
        assert_eq!(run.status, RunStatus::Paused);
    }
}

#[cfg(test)]
mod cancels {
    use anyhow::Result;
    use uuid::Uuid;

    use super::{cancel, ContainerLaunch, ContainerRuntime, ImageDisposition, PtyStream};
    use crate::{
        sandbox::PtyAttachment,
        session::{Artifact, Run, RunStatus},
    };

    #[derive(Default)]
    struct RecordingRuntime {
        stopped: Vec<String>,
    }

    impl ContainerRuntime for RecordingRuntime {
        fn build_or_reuse_image(&mut self, _: &str) -> Result<ImageDisposition> {
            unreachable!("cancel does not build images")
        }

        fn start_container(&mut self, _: &ContainerLaunch) -> Result<()> {
            unreachable!("cancel does not start containers")
        }

        fn attach_pty(&mut self, _: &str, _: PtyAttachment, _: PtyStream) -> Result<()> {
            unreachable!("cancel does not attach PTYs")
        }

        fn stop_container(&mut self, container: &str) -> Result<()> {
            self.stopped.push(container.into());
            Ok(())
        }
    }

    #[test]
    fn stops_a_running_container_and_records_why() {
        let mut run = Run {
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            resolved_model_id: "test-model".into(),
            status: RunStatus::Running,
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: Vec::<Artifact>::new(),
        };
        let mut runtime = RecordingRuntime::default();

        cancel(
            &mut run,
            "superseded by a higher-priority task",
            &mut runtime,
        )
        .expect("running run cancels");

        assert_eq!(run.status, RunStatus::Cancelled);
        assert_eq!(
            run.cancel_reason.as_deref(),
            Some("superseded by a higher-priority task")
        );
        assert_eq!(runtime.stopped, [format!("locus-agent-{}", run.id)]);
    }
}

#[cfg(test)]
mod normalizes {
    use serde_json::json;

    use super::normalize;
    use crate::telemetry::{EventVerb, StreamJsonAdapter};

    #[test]
    fn hands_captured_records_to_the_source_adapter() {
        let adapter = StreamJsonAdapter::new("type", [("message", EventVerb::Assistant)]);
        let raw = json!({"type": "message", "text": "complete"});

        let events = normalize(&adapter, [raw.clone()]).expect("captured record normalizes");

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].verb, EventVerb::Assistant);
        assert_eq!(events[0].raw, raw);
    }
}

#[cfg(test)]
mod persists_events {
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use serde_json::json;
    use sqlx::{query, query_scalar};
    use uuid::Uuid;

    use super::{normalize, persist_normalized_events};
    use crate::{
        backup::{MigrationBackup, RetainedBackupConfig},
        store::{PostgresConfig, PostgresContainer, Store},
        telemetry::{EventCollector, EventVerb, StreamJsonAdapter},
    };

    struct NoopMigrationBackup;

    impl MigrationBackup for NoopMigrationBackup {
        fn create_retained(&self, _: &RetainedBackupConfig) -> anyhow::Result<()> {
            Ok(())
        }
    }

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }

    impl Drop for DockerCleanup {
        fn drop(&mut self) {
            let _ = Command::new("docker")
                .args(["rm", "--force", &self.container_name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let _ = Command::new("docker")
                .args(["volume", "rm", "--force", &self.volume_name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind unused local port");
        listener.local_addr().expect("read local port").port()
    }

    #[tokio::test]
    async fn persists_each_normalized_event_with_its_run_identity_and_source_record() {
        let port = unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-run-events-test-{suffix}");
        let volume_name = format!("locus-run-events-test-data-{suffix}");
        let _cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container =
            PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
        container.start().await.expect("start PostgreSQL");
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
                &NoopMigrationBackup,
                &RetainedBackupConfig::new(
                    "postgres://locus@localhost/locus",
                    "/var/lib/locus/artifacts",
                    "/var/lib/locus/backups",
                ),
            )
            .await
            .expect("run migrations");

        let project_id = Uuid::new_v4();
        let agent_def_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let run_id = Uuid::new_v4();
        query("INSERT INTO core.projects (id, name) VALUES ($1, 'event persistence')")
            .bind(project_id)
            .execute(store.pool())
            .await
            .expect("insert project");
        query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ($1, 'event persistence', 1, '{}'::jsonb, '')",
        )
        .bind(agent_def_id)
        .execute(store.pool())
        .await
        .expect("insert agent definition");
        query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch)
             VALUES ($1, $2, $3, 'event persistence', 'agent/event-persistence')",
        )
        .bind(session_id)
        .bind(project_id)
        .bind(agent_def_id)
        .execute(store.pool())
        .await
        .expect("insert session");
        query(
            "INSERT INTO agents.runs (id, session_id, resolved_model_id, status)
             VALUES ($1, $2, 'test-model', 'running')",
        )
        .bind(run_id)
        .bind(session_id)
        .execute(store.pool())
        .await
        .expect("insert run");

        let adapter = StreamJsonAdapter::new("type", [("message", EventVerb::Assistant)]);
        let first_raw = json!({
            "type": "message",
            "text": "first",
            "timestamp": "2026-01-01T00:00:00Z"
        });
        let second_raw = json!({
            "type": "message",
            "text": "second",
            "timestamp": "2026-01-01T00:00:01Z"
        });
        let captured = normalize(&adapter, [first_raw.clone(), second_raw.clone()])
            .expect("normalize source records");

        let persisted = persist_normalized_events(
            &store,
            &EventCollector::new(2),
            run_id.to_string(),
            captured,
        )
        .await
        .expect("persist normalized events");

        assert_eq!(persisted.len(), 2);
        let rows: serde_json::Value = query_scalar(
            "SELECT jsonb_agg(
                jsonb_build_object(
                    'run_id', run_id::text,
                    'seq', seq,
                    'ts', to_char(ts AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'),
                    'raw', raw
                )
                ORDER BY seq
            )
            FROM agents.events
            WHERE run_id = $1",
        )
        .bind(run_id)
        .fetch_one(store.pool())
        .await
        .expect("read persisted events");
        assert_eq!(
            rows,
            json!([
                {
                    "run_id": run_id.to_string(),
                    "seq": 0,
                    "ts": "2026-01-01T00:00:00Z",
                    "raw": first_raw,
                },
                {
                    "run_id": run_id.to_string(),
                    "seq": 1,
                    "ts": "2026-01-01T00:00:01Z",
                    "raw": second_raw,
                }
            ])
        );
    }
}

#[cfg(test)]
mod streams_pty {
    use super::PtyStream;

    #[tokio::test]
    async fn delivers_pty_bytes_to_each_ui_subscriber() {
        let stream = PtyStream::new(2);
        let mut first_ui = stream.subscribe();
        let mut second_ui = stream.subscribe();

        stream.write(b"agent output");

        assert_eq!(first_ui.recv().await.unwrap(), b"agent output");
        assert_eq!(second_ui.recv().await.unwrap(), b"agent output");
    }
}

#[cfg(test)]
mod spawns {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use anyhow::Result;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::{
        materialize::{ExtensionEntry, ExtensionSet},
        registry::load_from_directory,
        sandbox::{
            agent_image_tag, Mount, PortAllocator, PtyAttachment, ToolPin, AGENT_PTY, CONFIG_SOURCE,
        },
        session::{Run, RunStatus},
    };

    #[derive(Default)]
    struct RecordingRuntime {
        calls: Vec<String>,
        started: Option<ContainerLaunch>,
        attached: Option<(String, PtyAttachment)>,
        pty_stream: Option<PtyStream>,
    }

    impl ContainerRuntime for RecordingRuntime {
        fn build_or_reuse_image(&mut self, image: &str) -> Result<ImageDisposition> {
            self.calls.push(format!("image:{image}"));
            Ok(ImageDisposition::Built)
        }

        fn start_container(&mut self, container: &ContainerLaunch) -> Result<()> {
            self.calls.push(format!("start:{}", container.name));
            self.started = Some(container.clone());
            Ok(())
        }

        fn attach_pty(
            &mut self,
            container: &str,
            attachment: PtyAttachment,
            stream: PtyStream,
        ) -> Result<()> {
            self.calls.push(format!("pty:{container}"));
            self.attached = Some((container.into(), attachment));
            self.pty_stream = Some(stream);
            Ok(())
        }

        fn stop_container(&mut self, container: &str) -> Result<()> {
            self.calls.push(format!("stop:{container}"));
            Ok(())
        }
    }

    fn root() -> PathBuf {
        std::env::temp_dir().join(format!("locus-run-spawns-{}", Uuid::new_v4()))
    }

    #[test]
    fn materializes_builds_starts_and_attaches_the_agent_pty() {
        let registry =
            load_from_directory(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../harnesses"))
                .expect("registry loads");
        let mut extensions = ExtensionSet::default();
        extensions.insert(
            "context",
            vec![ExtensionEntry::new("base.md", json!({}), "base context")],
        );
        let config_root = root();
        let run_id = Uuid::new_v4();
        let mut run = Run {
            id: run_id,
            session_id: Uuid::new_v4(),
            resolved_model_id: "test-model".into(),
            status: RunStatus::Queued,
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: vec![],
        };
        let request = SpawnRequest {
            project_id: "project-1",
            harness: registry.by_name("claude").expect("claude harness"),
            extensions: &extensions,
            config_root: config_root.clone(),
            socket_source: PathBuf::from("/tmp/locus.sock"),
            run_nonce: "nonce".into(),
            base_image_digest: "sha256:base".into(),
            tools: vec![ToolPin {
                name: "rg".into(),
                version: "14".into(),
            }],
            plugin: None,
        };
        let mut runtime = RecordingRuntime::default();
        let ports = PortAllocator::default();

        let spawned = spawn(&mut run, request, &ports, &mut runtime).expect("run spawns");

        let image = agent_image_tag(
            "sha256:base",
            &[ToolPin {
                name: "rg".into(),
                version: "14".into(),
            }],
        );
        assert_eq!(run.status, RunStatus::Running);
        assert_eq!(spawned.image, image);
        assert!(spawned.config.file("CLAUDE.md").is_some());
        assert_eq!(
            fs::read_to_string(config_root.join("CLAUDE.md")).unwrap(),
            "base context"
        );
        assert_eq!(
            runtime.calls,
            [
                format!("image:{image}"),
                format!("start:locus-agent-{run_id}"),
                format!("pty:locus-agent-{run_id}")
            ]
        );
        assert_eq!(spawned.image_disposition, ImageDisposition::Built);
        assert_eq!(spawned.container.mounts[1].destination, CONFIG_SOURCE);
        assert_eq!(
            spawned.container.mounts,
            vec![
                Mount {
                    source: "/tmp/locus.sock".into(),
                    destination: "/run/locus.sock".into(),
                    read_only: false
                },
                Mount {
                    source: config_root.display().to_string(),
                    destination: CONFIG_SOURCE.into(),
                    read_only: true
                }
            ]
        );
        assert_eq!(spawned.container.network, "locus-project-1");
        assert!(spawned
            .container
            .environment
            .iter()
            .any(|value| value == &format!("LOCUS_PORT={}", spawned.port)));
        assert!(spawned
            .container
            .environment
            .iter()
            .any(|value| value == "LOCUS_RUN_NONCE=nonce"));
        assert_eq!(
            runtime.attached,
            Some((spawned.container.name.clone(), AGENT_PTY))
        );
        let mut ui = spawned.pty_stream.subscribe();
        assert_eq!(
            runtime
                .pty_stream
                .as_ref()
                .expect("runtime received PTY stream")
                .write(b"agent output"),
            1
        );
        assert_eq!(
            ui.try_recv().expect("UI receives PTY bytes"),
            b"agent output"
        );

        let _ = fs::remove_dir_all(config_root);
    }
}
