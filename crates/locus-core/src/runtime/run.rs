//! Spawn one configured agent container for a queued run.

use crate::ids::{ProjectId, RunId, SessionId};
use std::{path::PathBuf, sync::Arc};

use crate::ipc::PtyChannel;

use anyhow::{bail, Context, Result};

use crate::runtime::{
    acp::{AgentSession, UpdateStream},
    backend::RuntimeBackend,
    container::{ContainerLaunch, ContainerRuntime, ImageDisposition},
};
use crate::{
    harness::{
        materialize::{
            context::assemble_frozen_head, extensions::ExtensionSet,
            extensions::ProjectExtensionScope, materialize, plugin::PluginHost,
            report::MaterializationReport, tree::MaterializedTree,
        },
        registry::HarnessDefinition,
    },
    runtime::session::{Run, RunStatus},
    sandbox::{
        credential_proxy::CredentialProxy,
        egress::{DestinationAllowlists, EgressTier},
        forward_proxy::{ForwardProxyLaunch, ForwardProxyPolicy},
        image::agent_image_tag,
        image::ToolPin,
        mounts::agent_mounts,
        ports::project_network,
        ports::PortAllocator,
    },
    services::telemetry::EventCollector,
    services::{
        handoff::HandoffContext,
        memory::{
            recall_with_settings, ContextBudget, DurableMemoryStore, FrozenCatalog, RecallSettings,
            TaskClass,
        },
        project::ProjectSettings,
        tools::{ProjectToolScope, RoleToolScope},
    },
    store::{audits::StoreAuditSink, Store},
};
use url::Url;

/// Core-owned context for a request received through one run's socket endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunContext {
    run_id: RunId,
}

impl RunContext {
    pub fn new(run_id: RunId) -> Self {
        Self { run_id }
    }
}

/// The small deterministic state recited at the mutable end of a run's context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecitationState {
    pub plan_active: bool,
    pub objective: String,
    pub step: String,
    pub unresolved_errors: u32,
}

impl RecitationState {
    pub fn planned(
        objective: impl Into<String>,
        step: impl Into<String>,
        unresolved_errors: u32,
    ) -> Self {
        Self {
            plan_active: true,
            objective: objective.into(),
            step: step.into(),
            unresolved_errors,
        }
    }

    pub fn without_plan() -> Self {
        Self {
            plan_active: false,
            objective: String::new(),
            step: String::new(),
            unresolved_errors: 0,
        }
    }
}

/// A rendered recitation is plain text so it can travel through the existing hook
/// `additionalContext` field without adding a telemetry verb or a model call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecitationBlock(String);

impl RecitationBlock {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn hook_context(&self) -> serde_json::Value {
        serde_json::json!({"additionalContext": self.0})
    }
}

pub fn recitation_block(state: &RecitationState) -> Option<RecitationBlock> {
    if !state.plan_active {
        return None;
    }
    let objective = state.objective.trim();
    let step = state.step.trim();
    if objective.is_empty() || step.is_empty() {
        return None;
    }
    Some(RecitationBlock(format!(
        "Objective: {objective}\nStep: {step}\nUnresolved errors: {}",
        state.unresolved_errors
    )))
}

/// Emits only when task state changes. The comparison is local and deterministic;
/// the hook path can therefore retain its 100ms/exit-0 guarantees.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecitationEmitter {
    last_state: Option<RecitationState>,
}

impl RecitationEmitter {
    pub fn on_task_state_change(&mut self, state: RecitationState) -> Option<RecitationBlock> {
        if self.last_state.as_ref() == Some(&state) {
            return None;
        }
        self.last_state = Some(state.clone());
        recitation_block(&state)
    }

    pub fn reset(&mut self) {
        self.last_state = None;
    }
}

/// Place recitation after the caller's frozen context head. It never rewrites the
/// head and returns the head byte-for-byte when no plan is active.
pub fn append_recitation_tail(head: &str, block: Option<&RecitationBlock>) -> String {
    let Some(block) = block else {
        return head.to_owned();
    };
    if head.is_empty() {
        block.as_str().to_owned()
    } else {
        format!("{head}\n\n{}", block.as_str())
    }
}

/// The run-local context assembled from the frozen head, bounded recall, and
/// mutable recitation tail. The head and tail are kept separate so callers can
/// cache the former without rebuilding it on every turn.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunContextAssembly {
    pub frozen_head: String,
    pub mutable_tail: String,
    pub recitation: Option<RecitationBlock>,
    pub recalled_ids: Vec<String>,
}

pub struct RunContextRequest<'a> {
    pub base_context: &'a str,
    pub memory: &'a mut DurableMemoryStore,
    pub project_id: ProjectId,
    pub query: &'a str,
    pub embedding: &'a [f32],
    pub task_class: TaskClass,
    pub effective_window_tokens: usize,
    pub state: RecitationState,
}

pub fn assemble_run_context(request: RunContextRequest<'_>) -> RunContextAssembly {
    let RunContextRequest {
        base_context,
        memory,
        project_id,
        query,
        embedding,
        task_class,
        effective_window_tokens,
        state,
    } = request;
    let recalled_ids = recall_with_settings(
        memory,
        project_id,
        None,
        query,
        embedding,
        task_class,
        RecallSettings::default(),
    )
    .into_iter()
    .map(|(id, _)| id)
    .collect::<Vec<_>>();
    let entries = memory.entries().cloned().collect::<Vec<_>>();
    let frozen = FrozenCatalog::start(&entries);
    let budget = ContextBudget::from_effective_window(effective_window_tokens);
    let mut tail = frozen.tail(budget);
    for id in &recalled_ids {
        if let Some(entry) = memory.get(id) {
            tail.append(entry);
        }
    }
    let frozen_head = assemble_frozen_head([
        ("base-context", base_context),
        ("memory-catalog", frozen.snapshot().text.as_str()),
    ]);
    let mut emitter = RecitationEmitter::default();
    let recitation = emitter.on_task_state_change(state);
    let mutable_tail = append_recitation_tail(tail.text(), recitation.as_ref());
    RunContextAssembly {
        frozen_head,
        mutable_tail,
        recitation,
        recalled_ids,
    }
}

/// Read boundary for the state that a connected run may observe.
pub trait RunStateStore {
    fn read_run(&self, run_id: RunId) -> Result<Run>;
}

/// Returns the state belonging to the socket's run context, never a caller-selected run.
pub fn own_state(store: &impl RunStateStore, context: RunContext) -> Result<Run> {
    store.read_run(context.run_id)
}

#[cfg(test)]
mod own_state_only {
    use crate::ids::{RunId, SessionId};
    use std::collections::BTreeMap;

    use super::*;

    struct Runs(BTreeMap<RunId, Run>);

    impl RunStateStore for Runs {
        fn read_run(&self, run_id: RunId) -> Result<Run> {
            self.0
                .get(&run_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("run not found"))
        }
    }

    fn run(id: RunId, model: &str) -> Run {
        Run {
            id,
            session_id: SessionId::generate(),
            resolved_model_id: model.into(),
            status: RunStatus::Running,
            permission_posture: Default::default(),
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
        let own_id = RunId::generate();
        let other_id = RunId::generate();
        let own = run(own_id, "own-model");
        let other = run(other_id, "other-model");
        let store = Runs(BTreeMap::from([(own_id, own.clone()), (other_id, other)]));

        assert_eq!(
            own_state(&store, RunContext::new(own_id)).expect("read own run"),
            own
        );
    }
}

/// An inbox notification generated by run supervision rather than by a harness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InboxItem {
    pub session_id: SessionId,
    pub run_id: RunId,
    pub body: String,
}

/// The small persistence boundary the supervisor needs from the inbox service.
pub trait Inbox {
    fn file(&mut self, item: InboxItem) -> Result<()>;
}

/// A human-owned shell shares the pane plumbing but deliberately has no run identity.
#[derive(Clone, Debug)]
pub struct HumanTerminal {
    pub pty: PtyChannel,
}

impl HumanTerminal {
    pub fn open() -> Self {
        Self {
            pty: PtyChannel::new(1_024),
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
    pub workspace_remote: String,
    /// Agent-visible credential broker endpoint. The credential remains host-only.
    pub credential_proxy: CredentialProxyConfig,
    /// Host-only proxy that injects the sentinel and records the egress authorization.
    pub credential_proxy_authorizer: &'a CredentialProxy,
    /// Durable audit sink for every credential-proxy request made by this run.
    pub audit_store: Store,
    /// Network capability permitted for this run.
    pub egress_tier: EgressTier,
    /// Provider-derived model hosts and project-declared package registry hosts. Package
    /// registries intentionally default to empty; no implicit package endpoint is trusted.
    pub egress_allowlists: DestinationAllowlists,
    /// Host-only directory mounted into the forwarding sidecar, never into the agent.
    pub egress_policy_root: PathBuf,
    /// Per-run capability validated by the daemon socket before it routes any agent request.
    pub run_nonce: String,
    /// Whether this run's resolved tool allow-list includes clone-local LSP.
    pub lsp_enabled: bool,
    pub base_image_digest: String,
    /// The catalog-resolved baseline; project and role scopes can only remove from it.
    pub tools: Vec<ToolPin>,
    pub project_extension_scope: ProjectExtensionScope,
    pub project_tool_scope: ProjectToolScope,
    pub role_tool_scope: RoleToolScope,
    /// Project settings for this run. The persisted spawn path refreshes this from `core.settings`
    /// before writing the host-owned registration; it is never supplied by the CLI.
    pub project_settings: ProjectSettings,
    /// Optional session context used by the ownership-transfer route.
    pub handoff_context: Option<HandoffContext>,
    pub plugin: Option<&'a PluginHost>,
    /// The shared telemetry collector this run's events flow through — the same
    /// one the UI subscribes to — so persisted events and the live stream stay
    /// the same sequence.
    pub collector: &'a EventCollector,
}

/// Secret-free endpoint through which an agent can request host-brokered egress.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CredentialProxyConfig {
    endpoint: String,
}

const CREDENTIAL_PROXY_ENDPOINT: &str = "http://host.docker.internal:44000/";

/// Capacity of the run's streamed `session/update` bus, matching the PTY stream's
/// buffering so a slow consumer drops at the same order of lag.
const ACP_UPDATE_CAPACITY: usize = 1024;

/// The workspace clone target inside the agent container; the ACP session's cwd
/// (see `sandbox::workspace`, which clones into this path).
const WORKSPACE_CWD: &str = "/workspace";

impl CredentialProxyConfig {
    pub fn new(endpoint: impl Into<String>) -> Result<Self> {
        let endpoint = endpoint.into();
        let parsed = Url::parse(&endpoint).context("credential proxy endpoint must be a URL")?;
        if parsed.scheme() != "http"
            || parsed.host_str() != Some("host.docker.internal")
            || parsed.port() != Some(44000)
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.path() != "/"
            || parsed.query().is_some()
            || parsed.fragment().is_some()
            || endpoint != CREDENTIAL_PROXY_ENDPOINT
        {
            bail!("credential proxy endpoint must be the host proxy root")
        }
        Ok(Self { endpoint })
    }
}

#[cfg(test)]
#[test]
fn credential_proxy_rejects_credential_bearing_values() {
    assert!(CredentialProxyConfig::new("https://token@example.test").is_err());
}

#[derive(Debug, PartialEq, Eq)]
struct RegistrationLease(Arc<PathBuf>);

impl Drop for RegistrationLease {
    fn drop(&mut self) {
        crate::runtime::daemon::remove_agent_registration(self.0.as_path());
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AgentRunRegistrationGuard(Arc<RegistrationLease>);

/// The started container and the materialized configuration used for its prompt prefix.
#[derive(Clone, Debug)]
pub struct SpawnedRun {
    pub container: ContainerLaunch,
    pub config: MaterializedTree,
    pub context: RunContextAssembly,
    pub materialization: MaterializationReport,
    pub image: String,
    pub image_disposition: ImageDisposition,
    pub port: u16,
    /// Bus carrying the run's streamed ACP `session/update` notifications. The
    /// conversation itself is established by `start_acp_session` on the persisted
    /// spawn path and stored in `acp_session`.
    pub acp_updates: UpdateStream,
    pub acp_session: Option<AgentSession>,
    /// Kept for its `Drop`: the guard removes the agent registration file when the
    /// spawned run is dropped. It is constructed but never read.
    #[allow(dead_code)]
    registration: AgentRunRegistrationGuard,
}

/// Materialize the run configuration, ensure its agent image, then start its ACP container.
pub fn spawn(
    run: &mut Run,
    request: SpawnRequest<'_>,
    ports: &PortAllocator,
    runtime: &mut dyn ContainerRuntime,
) -> Result<SpawnedRun> {
    let forwarding =
        ForwardProxyLaunch::for_project(request.project_id, request.egress_policy_root.clone())?;
    let port = ports.allocate()?;
    let run_id = run.id.to_string();
    let proxy = request.credential_proxy_authorizer;
    match spawn_at_port(run, request, port, runtime) {
        Ok(spawned) => Ok(spawned),
        Err(error) => {
            proxy.release_run(&run_id);
            let _ = ForwardProxyPolicy::remove_from(&forwarding.policy_root, &run_id);
            let _ = runtime.release_egress_proxy(&forwarding, &run_id);
            ports.release(port);
            Err(error)
        }
    }
}

fn spawn_at_port(
    run: &mut Run,
    request: SpawnRequest<'_>,
    port: u16,
    runtime: &mut dyn ContainerRuntime,
) -> Result<SpawnedRun> {
    if !matches!(run.status, RunStatus::Queued | RunStatus::Running) {
        bail!("only queued or claimed runs may be spawned")
    }

    let extensions = request
        .extensions
        .project_scoped(&request.project_extension_scope);
    let mut memory = DurableMemoryStore::default();
    let context = assemble_run_context(RunContextRequest {
        base_context: request.project_settings.base_context().unwrap_or_default(),
        memory: &mut memory,
        project_id: request.project_id.parse().unwrap_or_default(),
        query: "",
        embedding: &[],
        task_class: TaskClass::Code,
        effective_window_tokens: request
            .project_settings
            .base_context_token_budget()
            .map(|value| value as usize)
            .unwrap_or(128_000),
        state: RecitationState::without_plan(),
    });
    let (config, materialization) = materialize(
        request.harness,
        &extensions,
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
    let lsp_enabled = request.lsp_enabled
        && request.tools.iter().any(|tool| {
            tool.name == "lsp"
                && request.project_tool_scope.permits(&tool.name)
                && request.role_tool_scope.permits(&tool.name)
        });
    let debug_adapters = request
        .project_settings
        .debug_configs()
        .values()
        .map(|config| config.adapter())
        .filter(|adapter| {
            request.tools.iter().any(|tool| {
                tool.name == *adapter
                    && request.project_tool_scope.permits(&tool.name)
                    && request.role_tool_scope.permits(&tool.name)
            })
        })
        .map(str::to_owned)
        .collect();
    let debug_configs = request.project_settings.debug_configs().clone();
    let registration_path = crate::runtime::daemon::write_agent_registration(
        &request.socket_source,
        &crate::runtime::daemon::AgentRunRegistration {
            run_id: run.id,
            nonce: request.run_nonce.clone(),
            lsp_enabled,
            debug_adapters,
            debug_configs,
            handoff_context: request.handoff_context.clone(),
        },
    )?;
    let registration =
        AgentRunRegistrationGuard(Arc::new(RegistrationLease(Arc::new(registration_path))));
    let audit_sink = StoreAuditSink::new(request.audit_store)?;
    request
        .credential_proxy_authorizer
        .attach_audit_sink(audit_sink.clone());
    runtime
        .attach_audit_sink(audit_sink)
        .context("attach runtime egress audit sink")?;
    request
        .credential_proxy_authorizer
        .configure_run(&run.id.to_string(), &request.run_nonce, request.egress_tier)
        .context("configure credential proxy for run")?;
    request
        .credential_proxy_authorizer
        .listen_configured()
        .context("start credential proxy listener")?;

    let forwarding =
        ForwardProxyLaunch::for_project(request.project_id, request.egress_policy_root.clone())?;
    let allowlists = request
        .egress_allowlists
        .clone()
        // The host credential gateway is reached only through this sidecar. It performs a
        // second nonce+sentinel check before it sends to the provider-derived model endpoint.
        .with_model_host(CredentialProxy::gateway_host());
    let forwarding_policy = ForwardProxyPolicy::new(
        run.id.to_string(),
        request.run_nonce.clone(),
        request.egress_tier,
        &allowlists,
    )?;
    runtime
        .ensure_agent_network(&forwarding.internal_network)
        .context("ensure internal agent network")?;
    if forwarding_policy.enabled() {
        forwarding_policy
            .write_to(&forwarding.policy_root)
            .context("deliver forwarding proxy policy")?;
        runtime
            .ensure_egress_proxy(&forwarding)
            .context("start forwarding proxy sidecar")?;
    }

    let tools = request
        .tools
        .into_iter()
        .filter(|tool| {
            request.project_tool_scope.permits(&tool.name)
                && request.role_tool_scope.permits(&tool.name)
        })
        .collect::<Vec<_>>();
    let image = agent_image_tag(&request.base_image_digest, &tools);
    let image_disposition = runtime
        .build_or_reuse_image(&image)
        .context("build or reuse agent image")?;
    let setup = crate::sandbox::workspace::workspace_clone_command(
        &request.workspace_remote,
        &run.id.to_string(),
    )?;
    let mut environment = request
        .credential_proxy_authorizer
        .container_environment_for_run(&run.id.to_string(), &request.run_nonce)
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>();
    environment.push(format!(
        "LOCUS_CREDENTIAL_PROXY={}",
        request.credential_proxy.endpoint
    ));
    environment.extend(
        forwarding_policy
            .agent_environment()
            .into_iter()
            .map(|(key, value)| format!("{key}={value}")),
    );
    environment.push(format!("LOCUS_PORT={port}"));
    if runtime.backend() == RuntimeBackend::Sbx {
        environment.push(format!(
            "LOCUS_SBX_EGRESS_TIER={}",
            request.egress_tier.as_str()
        ));
        environment.push(format!(
            "LOCUS_SBX_MODEL_HOSTS={}",
            allowlists
                .model_hosts()
                .cloned()
                .collect::<Vec<_>>()
                .join(",")
        ));
        environment.push(format!(
            "LOCUS_SBX_PACKAGE_HOSTS={}",
            allowlists
                .package_hosts()
                .cloned()
                .collect::<Vec<_>>()
                .join(",")
        ));
        environment.push(format!(
            "LOCUS_SBX_WORKSPACE_REMOTE={}",
            request.workspace_remote
        ));
        environment.push(format!("LOCUS_SBX_WORKSPACE_BRANCH=agent/{}", run.id));
    }
    environment.push(format!(
        "LOCUS_LSP_ENABLED={}",
        if lsp_enabled { "1" } else { "0" }
    ));
    let mut container = ContainerLaunch {
        name: format!("locus-agent-{}", run.id),
        image: image.clone(),
        command: std::iter::once(request.harness.binary.clone())
            .chain(request.harness.launch.argv.iter().cloned())
            .collect(),
        entrypoint: format!(
            "{} && {}",
            crate::sandbox::mounts::entrypoint_setup(),
            setup
        ),
        environment,
        mounts: agent_mounts(
            request.socket_source.display().to_string(),
            request.config_root.display().to_string(),
        )
        .to_vec(),
        network: project_network(request.project_id),
    };
    runtime
        .prepare_container(&mut container)
        .context("prepare agent container for its runtime backend")?;
    runtime
        .start_container(&container)
        .context("start agent container")?;
    // ACP owns the agent's only I/O path. The container is intentionally started
    // without a TTY; `start_acp_session` opens its stdin/stdout transport explicitly.
    run.status = RunStatus::Running;
    Ok(SpawnedRun {
        container,
        config,
        context,
        materialization,
        image,
        image_disposition,
        port,
        acp_updates: UpdateStream::new(ACP_UPDATE_CAPACITY),
        acp_session: None,
        registration,
    })
}

/// Reserve a run port in Postgres before starting the container. A failed synchronous start
/// releases the durable reservation, while successful runs retain it until terminal cleanup.
pub async fn spawn_persisted(
    store: &Store,
    run: &mut Run,
    mut request: SpawnRequest<'_>,
    runtime: &mut dyn ContainerRuntime,
) -> Result<SpawnedRun> {
    let project_id: ProjectId = request
        .project_id
        .parse()
        .context("spawn project id must be a UUID")?;
    request.project_settings = store.project_settings(project_id).await?;
    let forwarding =
        ForwardProxyLaunch::for_project(request.project_id, request.egress_policy_root.clone())?;
    let backend = runtime.backend();
    let port = store.allocate_run_port(run.id).await?;
    if let Err(error) = store.record_runtime_backend(run.id, backend).await {
        let _ = store.release_run_port(run.id).await;
        return Err(error).context("record run runtime backend");
    }
    let run_id = run.id.to_string();
    let proxy = request.credential_proxy_authorizer;
    let backend = runtime.backend();
    let agent_command = request.harness.binary.clone();
    let agent_args: Vec<String> = request.harness.launch.argv.clone();
    let collector = request.collector.clone();
    match spawn_at_port(run, request, port, runtime) {
        Ok(mut spawned) => {
            // Subscribe before the handshake so updates streamed during
            // establishment reach the durable pump.
            let updates = spawned.acp_updates.subscribe();
            // The conversation is the run: a container that spawns but hosts no ACP
            // session is a failed start, handled like a failed PTY attach.
            if let Err(error) =
                start_acp_session(&mut spawned, backend, agent_command, agent_args).await
            {
                if let Err(stop_error) = runtime.stop_container(&spawned.container.name) {
                    return Err(error).context(format!(
                        "establish the run's ACP session; failed to stop the started container: {stop_error}"
                    ));
                }
                proxy.release_run(&run_id);
                let _ = ForwardProxyPolicy::remove_from(&forwarding.policy_root, &run_id);
                let _ = runtime.release_egress_proxy(&forwarding, &run_id);
                store.release_run_port(run.id).await?;
                return Err(error).context("establish the run's ACP session");
            }
            // Detached: the pump ends on its own when the session's bus closes.
            crate::runtime::normalize::spawn_acp_event_pump(
                store.clone(),
                collector,
                run.id,
                updates,
            );
            Ok(spawned)
        }
        Err(error) => {
            proxy.release_run(&run_id);
            let _ = ForwardProxyPolicy::remove_from(&forwarding.policy_root, &run_id);
            let _ = runtime.release_egress_proxy(&forwarding, &run_id);
            store.release_run_port(run.id).await?;
            Err(error)
        }
    }
}

/// Establish the run's ACP conversation inside its container: exec the harness
/// binary over the container runtime's stdio transport and answer `session/new`
/// with the workspace cwd, storing the session on the spawned run. Streamed
/// `session/update` notifications flow on the run's `acp_updates` bus. ACP is the
/// agent's only I/O path; no terminal stream is attached.
async fn start_acp_session(
    spawned: &mut SpawnedRun,
    backend: RuntimeBackend,
    agent_command: String,
    agent_args: impl IntoIterator<Item = String>,
) -> Result<()> {
    let transport = crate::runtime::acp::container_stdio_transport_for_backend(
        backend,
        spawned.container.name.clone(),
        agent_command,
        agent_args,
    );
    let session = crate::runtime::acp::establish_session(
        transport,
        WORKSPACE_CWD,
        spawned.acp_updates.clone(),
    )
    .await
    .context("open the run's ACP conversation")?;
    spawned.acp_session = Some(session);
    Ok(())
}

/// Cancel the container and release its durable port reservation after it reaches a terminal state.
pub async fn cancel_persisted(
    store: &Store,
    proxy: &CredentialProxy,
    forwarding: &ForwardProxyLaunch,
    run: &mut Run,
    reason: impl AsRef<str>,
    runtime: &mut impl ContainerRuntime,
) -> Result<()> {
    cancel(run, reason, runtime)?;
    proxy.release_run(&run.id.to_string());
    runtime.release_egress_proxy(forwarding, &run.id.to_string())?;
    store.release_run_port(run.id).await
}

/// Release a run's resources after any terminal outcome that did not use cancellation.
pub async fn release_terminal_port(
    store: &Store,
    proxy: &CredentialProxy,
    forwarding: &ForwardProxyLaunch,
    run: &Run,
    runtime: &mut impl ContainerRuntime,
) -> Result<()> {
    if matches!(
        run.status,
        RunStatus::Queued | RunStatus::Running | RunStatus::Paused
    ) {
        bail!("only terminal runs release durable ports")
    }
    proxy.release_run(&run.id.to_string());
    runtime.release_egress_proxy(forwarding, &run.id.to_string())?;
    store.release_run_port(run.id).await
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
mod run_supervisor {
    use super::*;

    #[test]
    fn recitation_block() {
        let state = RecitationState::planned("ship the context layer", "verify the tail", 2);
        let block = super::recitation_block(&state).expect("active plan recites");
        assert!(block.as_str().lines().count() <= 3);
        assert!(block.as_str().contains("Objective: ship the context layer"));
        assert!(block.as_str().contains("Step: verify the tail"));
        assert!(block.as_str().contains("Unresolved errors: 2"));

        let head = "frozen base context";
        let tail = append_recitation_tail(head, Some(&block));
        assert!(tail.starts_with(head));
        assert_eq!(
            &tail[head.len()..],
            "\n\nObjective: ship the context layer\nStep: verify the tail\nUnresolved errors: 2"
        );
        assert!(super::recitation_block(&RecitationState::without_plan()).is_none());
        assert!(crate::services::workflow::orchestration_model_invocation_hook().is_none());
    }

    #[test]
    fn recites_only_on_state_change() {
        let state = RecitationState::planned("objective", "step", 0);
        let mut emitter = RecitationEmitter::default();
        assert!(emitter.on_task_state_change(state.clone()).is_some());
        assert!(emitter.on_task_state_change(state).is_none());
        assert!(emitter
            .on_task_state_change(RecitationState::planned("objective", "next", 0))
            .is_some());
    }
}

#[cfg(test)]
mod human_terminal_is_not_a_session {
    use super::HumanTerminal;

    #[tokio::test]
    async fn human_shell_has_pty_bytes_but_no_run_or_cost_state() {
        let terminal = HumanTerminal::open();
        let mut ui = terminal.pty.subscribe();

        terminal.pty.send(b"human command output");

        assert_eq!(
            ui.recv().await.expect("terminal bytes"),
            b"human command output"
        );
        // HumanTerminal deliberately contains no Session, Run, Event, or Usage fields.
    }
}

#[cfg(test)]
mod native_session_id {
    use crate::ids::{RunId, SessionId};

    use super::record_native_session_id;
    use crate::runtime::session::{Artifact, Run, RunStatus};

    #[test]
    fn retains_a_harness_session_id_only_on_the_run_that_received_it() {
        let mut run = Run {
            id: RunId::generate(),
            session_id: SessionId::generate(),
            resolved_model_id: "test-model".into(),
            status: RunStatus::Running,
            permission_posture: Default::default(),
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
mod permission_posture {
    use super::*;
    use crate::runtime::session::{PermissionPosture, Run};

    #[test]
    fn dispatch_pins_bypass_or_gated_posture() {
        let mut run = Run {
            id: RunId::generate(),
            session_id: SessionId::generate(),
            resolved_model_id: "test-model".into(),
            status: RunStatus::Queued,
            permission_posture: PermissionPosture::default(),
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: vec![],
        };
        run.set_permission_posture(PermissionPosture::Gated)
            .unwrap();
        assert_eq!(run.permission_posture, PermissionPosture::Gated);
        run.status = RunStatus::Running;
        assert!(run
            .set_permission_posture(PermissionPosture::Bypass)
            .is_err());
        assert_eq!(run.permission_posture, PermissionPosture::Gated);
    }
}

#[cfg(test)]
mod permission_request_by_posture {
    use super::*;
    use crate::runtime::session::PermissionPosture;
    use crate::services::telemetry::{AcpAdapter, Adapter, EventCollector};
    use serde_json::json;

    #[test]
    fn bypass_alarms_but_gated_waits_for_a_human() {
        let captured = AcpAdapter
            .normalize(json!({"method": "session/request_permission", "id": "p1"}))
            .unwrap()
            .pop()
            .unwrap();
        let collector = EventCollector::new(4);
        let mut alarms = collector.subscribe_alarms();
        let mut gates = collector.subscribe_gates();
        collector.capture_with_posture(
            RunId::generate(),
            PermissionPosture::Bypass.is_gated(),
            captured.clone(),
        );
        collector.capture_with_posture(
            RunId::generate(),
            PermissionPosture::Gated.is_gated(),
            captured,
        );
        assert!(alarms.try_recv().is_ok());
        assert!(gates.try_recv().is_ok());
    }
}

#[cfg(test)]
mod checkpoints {
    use super::*;
    use crate::runtime::controls::{CheckpointLedger, WorkspaceSnapshot};
    use crate::services::telemetry::{Event, EventVerb};
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn restore_and_undo_keep_the_transcript() {
        let run_id = RunId::generate();
        let event = Event {
            run_id,
            seq: 0,
            ts: "2026-01-01T00:00:00Z".into(),
            verb: EventVerb::Assistant,
            text: Some("before edit".into()),
            tool: None,
            args: None,
            usage: None,
            raw: json!({"text": "before edit"}),
        };
        let mut files = BTreeMap::new();
        files.insert("src/lib.rs".into(), "old".into());
        let mut ledger = CheckpointLedger::default();
        let checkpoint = ledger.snapshot_before_edit(
            run_id,
            WorkspaceSnapshot {
                branch: "agent/test".into(),
                files,
            },
        );
        let restored = ledger
            .restore(checkpoint.id, std::slice::from_ref(&event))
            .unwrap();
        assert_eq!(restored.transcript, vec![event.clone()]);
        assert_eq!(restored.workspace.files["src/lib.rs"], "old");
        let undone = ledger.undo(std::slice::from_ref(&event)).unwrap();
        assert_eq!(undone.transcript, vec![event]);
    }
}

#[cfg(test)]
mod pause_holds_not_freezes {
    use crate::ids::{RunId, SessionId};

    use super::PauseController;
    use crate::runtime::session::{Artifact, Run, RunStatus};

    fn running_run() -> Run {
        Run {
            id: RunId::generate(),
            session_id: SessionId::generate(),
            resolved_model_id: "test-model".into(),
            status: RunStatus::Running,
            permission_posture: Default::default(),
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
    use crate::ids::{RunId, SessionId};
    use anyhow::Result;

    use super::{cancel, ContainerLaunch, ContainerRuntime, ImageDisposition};
    use crate::runtime::session::{Artifact, Run, RunStatus};

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

        fn stop_container(&mut self, container: &str) -> Result<()> {
            self.stopped.push(container.into());
            Ok(())
        }
    }

    #[test]
    fn stops_a_running_container_and_records_why() {
        let mut run = Run {
            id: RunId::generate(),
            session_id: SessionId::generate(),
            resolved_model_id: "test-model".into(),
            status: RunStatus::Running,
            permission_posture: Default::default(),
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
        harness::{
            materialize::{extensions::ExtensionEntry, extensions::ExtensionSet},
            registry::{load_from_directory, HarnessDefinition},
        },
        runtime::session::{Run, RunStatus},
        sandbox::{image::agent_image_tag, image::ToolPin, mounts::Mount, CONFIG_SOURCE},
    };

    #[derive(Default)]
    struct RecordingRuntime {
        calls: Vec<String>,
        started: Option<ContainerLaunch>,
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

        fn stop_container(&mut self, container: &str) -> Result<()> {
            self.calls.push(format!("stop:{container}"));
            Ok(())
        }

        fn ensure_agent_network(&mut self, network: &str) -> Result<()> {
            self.calls.push(format!("network:{network}"));
            Ok(())
        }

        fn ensure_egress_proxy(&mut self, proxy: &ForwardProxyLaunch) -> Result<()> {
            self.calls.push(format!("proxy:{}", proxy.name));
            Ok(())
        }
    }

    fn root() -> PathBuf {
        std::env::temp_dir().join(format!("locus-run-spawns-{}", Uuid::new_v4()))
    }

    fn spawn_request<'a>(
        harness: &'a HarnessDefinition,
        extensions: &'a ExtensionSet,
        config_root: PathBuf,
        credential_proxy_authorizer: &'a CredentialProxy,
        project_settings: ProjectSettings,
        collector: &'a EventCollector,
    ) -> SpawnRequest<'a> {
        let egress_policy_root = config_root.with_file_name("locus-forwarding-proxy-policies");
        SpawnRequest {
            project_id: "project-1",
            harness,
            extensions,
            config_root,
            socket_source: PathBuf::from("/tmp/locus.sock"),
            workspace_remote: "/var/lib/locus/repos/project.git".into(),
            credential_proxy: CredentialProxyConfig::new(CREDENTIAL_PROXY_ENDPOINT).unwrap(),
            credential_proxy_authorizer,
            audit_store: Store::connect_lazy("postgres://locus@127.0.0.1/locus").unwrap(),
            egress_tier: EgressTier::Model,
            egress_allowlists: DestinationAllowlists::new(
                ["api.anthropic.com"],
                std::iter::empty::<&str>(),
            ),
            egress_policy_root,
            run_nonce: "nonce".into(),
            lsp_enabled: false,
            base_image_digest: "sha256:base".into(),
            tools: vec![
                ToolPin {
                    name: "git".into(),
                    version: "2.49".into(),
                },
                ToolPin {
                    name: "rg".into(),
                    version: "14".into(),
                },
                ToolPin {
                    name: "sqlx".into(),
                    version: "0.8".into(),
                },
            ],
            project_extension_scope: {
                let mut scope = ProjectExtensionScope::default();
                scope.disable_extension("rules");
                scope
            },
            project_tool_scope: ProjectToolScope::new(["sqlx"]),
            role_tool_scope: RoleToolScope::new(["git"]),
            project_settings,
            handoff_context: None,
            plugin: None,
            collector,
        }
    }

    #[tokio::test]
    async fn persisted_spawn_uses_the_reserved_port() {
        let registry =
            load_from_directory(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../harnesses"))
                .expect("registry loads");
        let mut extensions = ExtensionSet::default();
        extensions.insert(
            "context",
            vec![ExtensionEntry::new("base.md", json!({}), "base context")],
        );
        extensions.insert(
            "rules",
            vec![ExtensionEntry::new(
                "no-secrets.md",
                json!({}),
                "never commit secrets",
            )],
        );
        let config_root = root();
        let run_id = RunId::generate();
        let mut run = Run {
            id: run_id,
            session_id: SessionId::generate(),
            resolved_model_id: "test-model".into(),
            status: RunStatus::Queued,
            permission_posture: Default::default(),
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: vec![],
        };
        let credential_proxy_authorizer = CredentialProxy::new("test-secret", "api_key");
        let collector = EventCollector::new(1024);
        let request = spawn_request(
            registry.by_name("claude").expect("claude harness"),
            &extensions,
            config_root.clone(),
            &credential_proxy_authorizer,
            ProjectSettings::default(),
            &collector,
        );
        let mut runtime = RecordingRuntime::default();

        let _fixed_port = crate::testkit::postgres::serialize_fixed_port();

        let spawned = spawn_at_port(&mut run, request, 43_210, &mut runtime)
            .expect("run spawns with its durable reservation");

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
        assert!(spawned.config.file("rules/no-secrets.md").is_none());
        assert!(fs::read_to_string(config_root.join("CLAUDE.md"))
            .unwrap()
            .starts_with("base context"));
        assert_eq!(
            runtime.calls,
            [
                "network:locus-project-1-internal".into(),
                "proxy:locus-egress-proxy-project-1".into(),
                format!("image:{image}"),
                format!("start:locus-agent-{run_id}")
            ]
        );
        assert_eq!(spawned.image_disposition, ImageDisposition::Built);
        assert_eq!(spawned.port, 43_210);
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
        assert_eq!(spawned.container.network, "locus-project-1-internal");
        assert!(spawned.container.entrypoint.contains("git clone"));
        assert!(spawned.container.entrypoint.contains("/workspace"));
        assert!(spawned.container.entrypoint.contains("checkout -b agent/"));
        assert!(spawned
            .container
            .mounts
            .iter()
            .all(|mount| mount.destination != "/workspace"));
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
        assert!(spawned
            .container
            .environment
            .iter()
            .any(|value| value == &format!("LOCUS_RUN_ID={run_id}")));
        assert_eq!(
            credential_proxy_authorizer
                .listener_address()
                .unwrap()
                .port(),
            44_000
        );
        assert!(spawned
            .container
            .environment
            .iter()
            .any(|value| value == "ANTHROPIC_API_KEY=sk-locus-sentinel"));
        assert!(spawned
            .container
            .environment
            .iter()
            .any(|value| value == "LOCUS_CREDENTIAL_PROXY=http://host.docker.internal:44000/"));
        assert!(spawned.container.environment.iter().any(|value| {
            value == &format!("HTTPS_PROXY=http://{run_id}:nonce@locus-egress-proxy:3128")
        }));
        assert!(!spawned
            .container
            .environment
            .iter()
            .any(|value| value == "HTTPS_PROXY=http://host.docker.internal:44000/"));
        assert!(!spawned
            .container
            .environment
            .iter()
            .any(|value| value.contains("test-secret")));
        assert!(credential_proxy_authorizer.audit_rows().is_empty());

        let _ = fs::remove_dir_all(config_root);
    }
}
