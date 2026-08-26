//! Headless daemon lifetime and the authenticated agent socket.

use crate::ids::RunId;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};
use uuid::Uuid;

use crate::{
    runtime::{
        container::ContainerRuntime,
        dap::DebugSessionRegistry,
        run::{self, SpawnRequest, SpawnedRun},
        session::Run,
    },
    store::Store,
};

const MAX_AGENT_SOCKET_FRAME_BYTES: u32 = 1_048_576;

/// Every agent-facing daemon operation. Unknown verbs are rejected while decoding the socket frame.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AgentSocketVerb {
    #[serde(rename = "memory.note.add")]
    MemoryNoteAdd,
    #[serde(rename = "memory.note.replace")]
    MemoryNoteReplace,
    #[serde(rename = "memory.note.remove")]
    MemoryNoteRemove,
    #[serde(rename = "memory.recall")]
    MemoryRecall,
    #[serde(rename = "memory.write")]
    MemoryWrite,
    #[serde(rename = "memory.forget")]
    MemoryForget,
    #[serde(rename = "memory.adjudicate")]
    MemoryAdjudicate,
    #[serde(rename = "memory.explain")]
    MemoryExplain,
    #[serde(rename = "mail.send")]
    MailSend,
    #[serde(rename = "mail.list")]
    MailList,
    #[serde(rename = "mail.read")]
    MailRead,
    #[serde(rename = "mail.reply")]
    MailReply,
    #[serde(rename = "mail.drain")]
    MailDrain,
    #[serde(rename = "mail.wait")]
    MailWait,
    #[serde(rename = "task.list")]
    TaskList,
    #[serde(rename = "task.show")]
    TaskShow,
    #[serde(rename = "task.move")]
    TaskMove,
    #[serde(rename = "task.assign")]
    TaskAssign,
    #[serde(rename = "task.comment")]
    TaskComment,
    #[serde(rename = "wiki.search")]
    WikiSearch,
    #[serde(rename = "wiki.read")]
    WikiRead,
    #[serde(rename = "wiki.write")]
    WikiWrite,
    #[serde(rename = "wiki.history")]
    WikiHistory,
    #[serde(rename = "wiki.ingest")]
    WikiIngest,
    #[serde(rename = "wiki.query")]
    WikiQuery,
    #[serde(rename = "wiki.lint")]
    WikiLint,
    #[serde(rename = "lsp.def")]
    LspDef,
    #[serde(rename = "lsp.refs")]
    LspRefs,
    #[serde(rename = "lsp.hover")]
    LspHover,
    #[serde(rename = "lsp.symbols")]
    LspSymbols,
    #[serde(rename = "lsp.diagnostics")]
    LspDiagnostics,
    #[serde(rename = "lsp.rename")]
    LspRename,
    /// Internal host authorization used by the container-local LSP client. It is not exposed
    /// through the agent CLI verb allow-list.
    #[serde(rename = "lsp.lease")]
    LspLease,
    #[serde(rename = "debug.start")]
    DebugStart,
    #[serde(rename = "debug.break")]
    DebugBreak,
    #[serde(rename = "debug.step")]
    DebugStep,
    #[serde(rename = "debug.run")]
    DebugRun,
    #[serde(rename = "debug.next")]
    DebugNext,
    #[serde(rename = "debug.finish")]
    DebugFinish,
    #[serde(rename = "debug.continue")]
    DebugContinue,
    #[serde(rename = "debug.stop")]
    DebugStop,
    #[serde(rename = "debug.stack")]
    DebugStack,
    #[serde(rename = "debug.vars")]
    DebugVars,
    #[serde(rename = "debug.eval")]
    DebugEval,
    #[serde(rename = "browse.open")]
    BrowseOpen,
    #[serde(rename = "browse.click")]
    BrowseClick,
    #[serde(rename = "browse.fill")]
    BrowseFill,
    #[serde(rename = "browse.press")]
    BrowsePress,
    #[serde(rename = "browse.assert")]
    BrowseAssert,
    #[serde(rename = "browse.screenshot")]
    BrowseScreenshot,
    #[serde(rename = "browse.record")]
    BrowseRecord,
    #[serde(rename = "browse.console")]
    BrowseConsole,
    #[serde(rename = "browse.network")]
    BrowseNetwork,
    #[serde(rename = "agent.invoke")]
    AgentInvoke,
    #[serde(rename = "svc.up")]
    SvcUp,
    #[serde(rename = "svc.down")]
    SvcDown,
    #[serde(rename = "ask")]
    Ask,
    #[serde(rename = "run.status")]
    RunStatus,
    #[serde(rename = "run.artifacts")]
    RunArtifacts,
    #[serde(rename = "handoff")]
    Handoff,
    #[serde(rename = "artifact.put")]
    ArtifactPut,
    #[serde(rename = "artifact.get")]
    ArtifactGet,
    #[serde(rename = "artifact.comments")]
    ArtifactComments,
    #[serde(rename = "tools.list")]
    ToolsList,
    #[serde(rename = "tools.docs")]
    ToolsDocs,
    #[serde(rename = "lint")]
    Lint,
}

impl std::fmt::Display for AgentSocketVerb {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let encoded = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        formatter.write_str(&encoded[1..encoded.len() - 1])
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSocketErrorKind {
    InvalidRequest,
    PermissionDenied,
    Unavailable,
    Internal,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSocketError {
    pub kind: AgentSocketErrorKind,
    pub message: String,
}

impl AgentSocketError {
    fn permission_denied(message: impl Into<String>) -> Self {
        Self {
            kind: AgentSocketErrorKind::PermissionDenied,
            message: message.into(),
        }
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            kind: AgentSocketErrorKind::Unavailable,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AgentSocketError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for AgentSocketError {}

/// `locusd` owns active runs. Desktop windows attach and detach without owning them.
#[derive(Default)]
pub struct Daemon {
    active_runs: BTreeSet<RunId>,
    attached_windows: usize,
    debug: DebugSessionRegistry,
}

impl Daemon {
    pub fn with_debug(debug: DebugSessionRegistry) -> Self {
        Self {
            active_runs: BTreeSet::new(),
            attached_windows: 0,
            debug,
        }
    }

    pub fn attach_window(&mut self) {
        self.attached_windows += 1;
    }
    pub fn detach_window(&mut self) {
        self.attached_windows = self.attached_windows.saturating_sub(1);
    }
    pub fn begin_run(&mut self, run_id: RunId) {
        self.active_runs.insert(run_id);
    }
    pub fn finish_run(&mut self, run_id: RunId) {
        self.active_runs.remove(&run_id);
        self.debug.end_run(run_id);
    }
    pub fn tracks(&self, run_id: RunId) -> bool {
        self.active_runs.contains(&run_id)
    }

    pub fn debug(&self) -> &DebugSessionRegistry {
        &self.debug
    }
    pub fn attached_windows(&self) -> usize {
        self.attached_windows
    }

    /// Starts a persisted run through the daemon-owned credential-proxy path.
    pub async fn spawn_run(
        &mut self,
        store: &Store,
        run: &mut Run,
        request: SpawnRequest<'_>,
        runtime: &mut impl ContainerRuntime,
    ) -> Result<SpawnedRun> {
        let spawned = run::spawn_persisted(store, run, request, runtime).await?;
        self.begin_run(run.id);
        Ok(spawned)
    }
}

/// Framed request sent by an agent container. The nonce identifies the one run permitted to make
/// the request; container peer credentials cannot provide that identity reliably on macOS relays.
#[derive(Debug, Deserialize)]
pub struct AgentSocketRequest {
    pub nonce: String,
    pub verb: AgentSocketVerb,
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSocketResponse {
    pub result: Option<Value>,
    pub error: Option<AgentSocketError>,
}

/// Domain routing remains in the core, never in the container CLI.
///
/// `authorize` is deliberately mandatory rather than a permissive default: every new route,
/// including one with a caller-supplied ID, must prove the authenticated run owns its target
/// project/session before the daemon invokes it.
pub trait AgentSocketRouter: Send + Sync {
    fn authorize(
        &self,
        run_id: RunId,
        verb: AgentSocketVerb,
        args: &[String],
    ) -> std::result::Result<(), AgentSocketError>;

    fn route(
        &self,
        run_id: RunId,
        verb: AgentSocketVerb,
        args: &[String],
    ) -> std::result::Result<Value, AgentSocketError>;
}

/// Bind a daemon-owned socket. Its parent must be host-owned and inaccessible to agents.
#[derive(Clone, Default)]
pub struct AgentSocketCapabilities {
    runs: Arc<RwLock<BTreeMap<String, RunId>>>,
    lsp_runs: Arc<RwLock<BTreeSet<RunId>>>,
    debug_adapters: Arc<RwLock<BTreeMap<RunId, BTreeSet<String>>>>,
    debug_configs:
        Arc<RwLock<BTreeMap<RunId, BTreeMap<String, crate::services::project::DebugRunConfig>>>>,
}

impl AgentSocketCapabilities {
    pub fn register(&self, nonce: impl Into<String>, run_id: RunId) -> Result<()> {
        self.runs
            .write()
            .map_err(|_| anyhow::anyhow!("agent capability lock is poisoned"))?
            .insert(nonce.into(), run_id);
        self.debug_adapters
            .write()
            .map_err(|_| anyhow::anyhow!("agent debug capability lock is poisoned"))?
            .entry(run_id)
            .or_default();
        self.debug_configs
            .write()
            .map_err(|_| anyhow::anyhow!("agent debug config lock is poisoned"))?
            .entry(run_id)
            .or_default();
        Ok(())
    }

    /// Register a run with the optional LSP capability already checked by the run supervisor.
    pub fn register_lsp(&self, nonce: impl Into<String>, run_id: RunId) -> Result<()> {
        self.register(nonce, run_id)?;
        self.lsp_runs
            .write()
            .map_err(|_| anyhow::anyhow!("agent LSP capability lock is poisoned"))?
            .insert(run_id);
        Ok(())
    }

    pub fn revoke(&self, nonce: &str) -> Result<()> {
        let run_id = self
            .runs
            .write()
            .map_err(|_| anyhow::anyhow!("agent capability lock is poisoned"))?
            .remove(nonce);
        if let Some(run_id) = run_id {
            self.lsp_runs
                .write()
                .map_err(|_| anyhow::anyhow!("agent LSP capability lock is poisoned"))?
                .remove(&run_id);
            self.debug_adapters
                .write()
                .map_err(|_| anyhow::anyhow!("agent debug capability lock is poisoned"))?
                .remove(&run_id);
            self.debug_configs
                .write()
                .map_err(|_| anyhow::anyhow!("agent debug config lock is poisoned"))?
                .remove(&run_id);
        }
        Ok(())
    }

    /// Atomically replace registrations discovered by the run supervisor.
    pub fn replace(&self, registrations: &[AgentRunRegistration]) -> Result<()> {
        let mut runs = BTreeMap::new();
        let mut run_ids = BTreeSet::new();
        let mut lsp_runs = BTreeSet::new();
        let mut debug_adapters = BTreeMap::new();
        let mut debug_configs = BTreeMap::new();
        for registration in registrations {
            if registration.nonce.trim().is_empty() {
                bail!("agent registration nonce must not be empty")
            }
            if !run_ids.insert(registration.run_id) {
                bail!("duplicate agent registration run id")
            }
            if runs
                .insert(registration.nonce.clone(), registration.run_id)
                .is_some()
            {
                bail!("duplicate agent registration nonce")
            }
            if registration.lsp_enabled {
                lsp_runs.insert(registration.run_id);
            }
            debug_adapters.insert(
                registration.run_id,
                registration.debug_adapters.iter().cloned().collect(),
            );
            debug_configs.insert(registration.run_id, registration.debug_configs.clone());
        }
        *self
            .runs
            .write()
            .map_err(|_| anyhow::anyhow!("agent capability lock is poisoned"))? = runs;
        *self
            .lsp_runs
            .write()
            .map_err(|_| anyhow::anyhow!("agent LSP capability lock is poisoned"))? = lsp_runs;
        *self
            .debug_adapters
            .write()
            .map_err(|_| anyhow::anyhow!("agent debug capability lock is poisoned"))? =
            debug_adapters;
        *self
            .debug_configs
            .write()
            .map_err(|_| anyhow::anyhow!("agent debug config lock is poisoned"))? = debug_configs;
        Ok(())
    }

    pub fn set_debug_adapters(
        &self,
        run_id: RunId,
        adapters: impl IntoIterator<Item = String>,
    ) -> Result<()> {
        self.debug_adapters
            .write()
            .map_err(|_| anyhow::anyhow!("agent debug capability lock is poisoned"))?
            .insert(run_id, adapters.into_iter().collect());
        Ok(())
    }

    pub fn run_ids(&self) -> Result<BTreeSet<RunId>> {
        Ok(self
            .runs
            .read()
            .map_err(|_| anyhow::anyhow!("agent capability lock is poisoned"))?
            .values()
            .copied()
            .collect())
    }

    pub fn debug_adapters(&self, run_id: RunId) -> Result<BTreeSet<String>> {
        Ok(self
            .debug_adapters
            .read()
            .map_err(|_| anyhow::anyhow!("agent debug capability lock is poisoned"))?
            .get(&run_id)
            .cloned()
            .unwrap_or_default())
    }

    pub fn debug_config(
        &self,
        run_id: RunId,
        name: &str,
    ) -> Result<Option<crate::services::project::DebugRunConfig>> {
        Ok(self
            .debug_configs
            .read()
            .map_err(|_| anyhow::anyhow!("agent debug config lock is poisoned"))?
            .get(&run_id)
            .and_then(|configs| configs.get(name).cloned()))
    }

    fn snapshot(&self) -> Result<(BTreeMap<String, RunId>, BTreeSet<RunId>)> {
        Ok((
            self.runs
                .read()
                .map_err(|_| anyhow::anyhow!("agent capability lock is poisoned"))?
                .clone(),
            self.lsp_runs
                .read()
                .map_err(|_| anyhow::anyhow!("agent LSP capability lock is poisoned"))?
                .clone(),
        ))
    }
}

/// Host-written registration consumed by `locusd`; the agent cannot create one because the
/// registration directory is outside the container mounts.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentRunRegistration {
    pub run_id: RunId,
    pub nonce: String,
    pub lsp_enabled: bool,
    #[serde(default)]
    pub debug_adapters: Vec<String>,
    #[serde(default)]
    pub debug_configs: BTreeMap<String, crate::services::project::DebugRunConfig>,
}

pub fn agent_registration_root(socket_source: impl AsRef<Path>) -> PathBuf {
    socket_source
        .as_ref()
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("locus-runs")
}

fn secure_registration_root(root: &Path) -> Result<()> {
    fs::create_dir_all(root).with_context(|| format!("create {}", root.display()))?;
    if fs::symlink_metadata(root)?.file_type().is_symlink() {
        bail!("agent registration root must not be a symlink")
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root, fs::Permissions::from_mode(0o700))
            .with_context(|| format!("restrict agent registration root {}", root.display()))?;
    }
    Ok(())
}

/// Atomically publish one host-owned run registration beside the daemon socket.
pub fn write_agent_registration(
    socket_source: impl AsRef<Path>,
    registration: &AgentRunRegistration,
) -> Result<PathBuf> {
    if registration.nonce.trim().is_empty() {
        bail!("agent registration nonce must not be empty")
    }
    let root = std::env::var_os("LOCUS_RUN_REGISTRY")
        .map(PathBuf::from)
        .unwrap_or_else(|| agent_registration_root(socket_source));
    write_agent_registration_to(root, registration)
}

pub fn write_agent_registration_to(
    root: impl AsRef<Path>,
    registration: &AgentRunRegistration,
) -> Result<PathBuf> {
    let root = root.as_ref();
    if registration.nonce.trim().is_empty() {
        bail!("agent registration nonce must not be empty")
    }
    secure_registration_root(root)?;
    let destination = root.join(format!("{}.json", registration.run_id));
    let temporary = root.join(format!(".{}.{}.tmp", registration.run_id, Uuid::new_v4()));
    let bytes = serde_json::to_vec(registration)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .with_context(|| format!("create {}", temporary.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(&temporary, &destination)?;
    Ok(destination)
}

pub fn remove_agent_registration(path: impl AsRef<Path>) {
    let _ = fs::remove_file(path);
}

pub fn read_agent_registrations(root: impl AsRef<Path>) -> Result<Vec<AgentRunRegistration>> {
    let root = root.as_ref();
    if !root.exists() {
        return Ok(Vec::new());
    }
    secure_registration_root(root)?;
    let mut registrations = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_file() || entry.path().extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        registrations.push(serde_json::from_slice(&fs::read(entry.path())?)?);
    }
    registrations.sort_by_key(|registration: &AgentRunRegistration| registration.run_id);
    Ok(registrations)
}

pub fn bind_agent_socket(path: impl AsRef<Path>) -> Result<UnixListener> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if fs::symlink_metadata(parent)
            .with_context(|| format!("inspect socket parent {}", parent.display()))?
            .file_type()
            .is_symlink()
        {
            bail!("agent socket parent must not be a symlink")
        }
    }
    if path.exists() {
        std::fs::remove_file(path)
            .with_context(|| format!("remove stale socket {}", path.display()))?;
    }
    let listener = UnixListener::bind(path)
        .with_context(|| format!("bind agent socket {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("restrict agent socket {}", path.display()))?;
    }
    Ok(listener)
}

/// Serve a single connection, which makes the boundary independently testable. A daemon calls
/// this in its accept loop and retains the capability map for the run's lifetime.
pub async fn serve_agent_socket_once(
    listener: &UnixListener,
    capabilities: &BTreeMap<String, RunId>,
    router: &impl AgentSocketRouter,
) -> Result<()> {
    let (stream, _) = listener
        .accept()
        .await
        .context("accept agent socket client")?;
    // The legacy single-snapshot API has no way to prove an LSP capability. Keep it
    // non-LSP-only rather than silently authorizing every authenticated run.
    let no_lsp_runs = BTreeSet::new();
    serve_agent_socket_stream(stream, capabilities, Some(&no_lsp_runs), router).await
}

fn is_lsp_socket_verb(verb: AgentSocketVerb) -> bool {
    matches!(
        verb,
        AgentSocketVerb::LspDef
            | AgentSocketVerb::LspRefs
            | AgentSocketVerb::LspHover
            | AgentSocketVerb::LspSymbols
            | AgentSocketVerb::LspDiagnostics
            | AgentSocketVerb::LspRename
            | AgentSocketVerb::LspLease
    )
}

async fn serve_agent_socket_stream(
    mut stream: UnixStream,
    capabilities: &BTreeMap<String, RunId>,
    lsp_runs: Option<&BTreeSet<RunId>>,
    router: &impl AgentSocketRouter,
) -> Result<()> {
    let request: AgentSocketRequest = read_frame(&mut stream).await?;
    let response = match capabilities.get(&request.nonce) {
        Some(run_id)
            if lsp_runs
                .is_some_and(|runs| is_lsp_socket_verb(request.verb) && !runs.contains(run_id)) =>
        {
            AgentSocketResponse {
                result: None,
                error: Some(AgentSocketError::permission_denied(
                    "the authenticated run has no LSP capability",
                )),
            }
        }
        Some(run_id) => match router.authorize(*run_id, request.verb, &request.args) {
            Err(error) => AgentSocketResponse {
                result: None,
                error: Some(error),
            },
            Ok(()) => match router.route(*run_id, request.verb, &request.args) {
                Ok(result) => AgentSocketResponse {
                    result: Some(result),
                    error: None,
                },
                Err(error) => AgentSocketResponse {
                    result: None,
                    error: Some(error),
                },
            },
        },
        None => AgentSocketResponse {
            result: None,
            error: Some(AgentSocketError::permission_denied(
                "agent socket capability refused",
            )),
        },
    };
    write_frame(&mut stream, &response).await
}

/// Concurrent accept loop for the long-lived daemon. Capability registration is shared so a
/// supervisor can add and revoke runs without replacing the listener.
pub async fn serve_agent_socket_shared<R>(
    listener: &UnixListener,
    capabilities: AgentSocketCapabilities,
    router: Arc<R>,
) -> Result<()>
where
    R: AgentSocketRouter + 'static,
{
    loop {
        let (stream, _) = listener
            .accept()
            .await
            .context("accept agent socket client")?;
        let capabilities = capabilities.clone();
        let router = Arc::clone(&router);
        tokio::spawn(async move {
            match capabilities.snapshot() {
                Ok((snapshot, lsp_runs)) => {
                    if let Err(error) = serve_agent_socket_stream(
                        stream,
                        &snapshot,
                        Some(&lsp_runs),
                        router.as_ref(),
                    )
                    .await
                    {
                        tracing::warn!(%error, "agent socket request failed");
                    }
                }
                Err(error) => tracing::warn!(%error, "agent socket capability snapshot failed"),
            }
        });
    }
}

/// The accept loop `serve_agent_socket_once` was always meant to sit inside.
///
/// One connection at a time: the agent CLI is request/response and a run makes one call
/// at a time, so concurrency here would buy nothing and would let one agent's slow verb
/// hide another's. A connection that fails is logged and dropped — a malformed frame from
/// one container must not take the daemon down for every other run.
pub async fn serve_agent_socket(
    listener: &UnixListener,
    capabilities: &BTreeMap<String, RunId>,
    router: &impl AgentSocketRouter,
) -> Result<()> {
    loop {
        if let Err(error) = serve_agent_socket_once(listener, capabilities, router).await {
            tracing::warn!(%error, "agent socket request failed");
        }
    }
}

async fn read_frame<T: serde::de::DeserializeOwned>(stream: &mut UnixStream) -> Result<T> {
    let length = stream
        .read_u32()
        .await
        .context("read socket frame length")?;
    if length > MAX_AGENT_SOCKET_FRAME_BYTES {
        bail!("agent socket frame exceeds {MAX_AGENT_SOCKET_FRAME_BYTES} bytes")
    }
    let mut bytes = vec![0; length as usize];
    stream
        .read_exact(&mut bytes)
        .await
        .context("read socket frame body")?;
    serde_json::from_slice(&bytes).context("decode socket request")
}

async fn write_frame<T: Serialize>(stream: &mut UnixStream, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec(value).context("encode socket response")?;
    let length = u32::try_from(bytes.len()).context("socket response exceeds u32 length")?;
    stream
        .write_u32(length)
        .await
        .context("write socket frame length")?;
    stream
        .write_all(&bytes)
        .await
        .context("write socket frame body")?;
    stream.flush().await.context("flush socket response")
}

#[cfg(test)]
mod outlives_window {
    use super::Daemon;
    use crate::ids::RunId;
    #[test]
    fn background_daemon_keeps_runs_when_the_last_window_closes() {
        let run_id = RunId::generate();
        let mut daemon = Daemon::default();
        daemon.attach_window();
        daemon.begin_run(run_id);
        daemon.detach_window();
        assert_eq!(daemon.attached_windows(), 0);
        assert!(daemon.tracks(run_id));
    }
}

#[cfg(test)]
mod agent_socket {
    use super::{
        bind_agent_socket, read_frame, serve_agent_socket_once, serve_agent_socket_stream,
        AgentSocketRouter, MAX_AGENT_SOCKET_FRAME_BYTES,
    };
    use crate::ids::RunId;
    use serde_json::json;
    use std::{
        collections::{BTreeMap, BTreeSet},
        path::PathBuf,
    };
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::UnixStream,
    };
    use uuid::Uuid;

    struct Router;
    impl AgentSocketRouter for Router {
        fn authorize(
            &self,
            run_id: RunId,
            _: super::AgentSocketVerb,
            args: &[String],
        ) -> std::result::Result<(), super::AgentSocketError> {
            if args
                .first()
                .is_some_and(|target| target != &run_id.to_string())
            {
                return Err(super::AgentSocketError::permission_denied(
                    "socket target is outside the authenticated run",
                ));
            }
            Ok(())
        }

        fn route(
            &self,
            run_id: RunId,
            verb: super::AgentSocketVerb,
            _: &[String],
        ) -> std::result::Result<serde_json::Value, super::AgentSocketError> {
            Ok(json!({"run_id": run_id, "verb": verb}))
        }
    }
    fn path() -> PathBuf {
        std::env::temp_dir().join(format!("locus-daemon-{}.sock", Uuid::new_v4()))
    }
    async fn request(path: &PathBuf, value: serde_json::Value) -> serde_json::Value {
        let mut stream = UnixStream::connect(path).await.unwrap();
        let bytes = serde_json::to_vec(&value).unwrap();
        stream.write_u32(bytes.len() as u32).await.unwrap();
        stream.write_all(&bytes).await.unwrap();
        let length = stream.read_u32().await.unwrap();
        let mut response = vec![0; length as usize];
        stream.read_exact(&mut response).await.unwrap();
        serde_json::from_slice(&response).unwrap()
    }
    #[tokio::test]
    async fn refuses_lsp_before_routing_when_the_run_lacks_the_capability() {
        let path = path();
        let listener = bind_agent_socket(&path).unwrap();
        let run_id = RunId::generate();
        let capabilities = BTreeMap::from([(String::from("nonce"), run_id)]);
        let no_lsp_runs = BTreeSet::new();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            serve_agent_socket_stream(stream, &capabilities, Some(&no_lsp_runs), &Router)
                .await
                .unwrap();
        });
        let response = request(
            &path,
            json!({
                "nonce": "nonce",
                "verb": "lsp.lease",
                "args": ["lsp.def", "main.rs", "1", "0"]
            }),
        )
        .await;
        server.await.unwrap();
        assert_eq!(response["error"]["kind"], "permission_denied");
        assert_eq!(
            response["error"]["message"],
            "the authenticated run has no LSP capability"
        );
        std::fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn refuses_an_oversized_frame_before_allocating_it() {
        let path = path();
        let listener = bind_agent_socket(&path).unwrap();
        let client = tokio::spawn({
            let path = path.clone();
            async move {
                let mut stream = UnixStream::connect(path).await.unwrap();
                stream
                    .write_u32(MAX_AGENT_SOCKET_FRAME_BYTES + 1)
                    .await
                    .unwrap();
            }
        });
        let (mut stream, _) = listener.accept().await.unwrap();
        client.await.unwrap();
        let error = read_frame::<serde_json::Value>(&mut stream)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("exceeds"));
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn routes_only_the_run_bound_to_its_capability() {
        let path = path();
        let listener = bind_agent_socket(&path).unwrap();
        let run_id = RunId::generate();
        let capabilities = BTreeMap::from([("nonce".into(), run_id)]);
        let server = tokio::spawn(async move {
            serve_agent_socket_once(&listener, &capabilities, &Router)
                .await
                .unwrap();
        });
        let response = request(
            &path,
            json!({"nonce":"nonce","verb":"run.status","args":[]}),
        )
        .await;
        server.await.unwrap();
        assert_eq!(response["result"]["run_id"], run_id.to_string());
        let _ = std::fs::remove_file(path);
    }
    #[tokio::test]
    async fn refuses_a_cross_run_target_before_routing() {
        let path = path();
        let listener = bind_agent_socket(&path).unwrap();
        let run_a = RunId::generate();
        let run_b = RunId::generate();
        let capabilities = BTreeMap::from([("nonce".into(), run_a)]);
        let server = tokio::spawn(async move {
            serve_agent_socket_once(&listener, &capabilities, &Router)
                .await
                .unwrap();
        });
        let response = request(
            &path,
            json!({"nonce":"nonce","verb":"artifact.get","args":[run_b.to_string()]}),
        )
        .await;
        server.await.unwrap();
        assert_eq!(response["error"]["kind"], "permission_denied");
        assert_eq!(
            response["error"]["message"],
            "socket target is outside the authenticated run"
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn rejects_unknown_verbs_at_the_socket_boundary() {
        let error = serde_json::from_value::<super::AgentSocketRequest>(json!({
            "nonce": "nonce",
            "verb": "run.typo",
            "args": []
        }))
        .expect_err("unknown verb is rejected");
        assert!(error.to_string().contains("run.typo"));
    }

    #[tokio::test]
    async fn refuses_a_missing_or_wrong_capability_before_routing() {
        let path = path();
        let listener = bind_agent_socket(&path).unwrap();
        let server = tokio::spawn(async move {
            serve_agent_socket_once(&listener, &BTreeMap::new(), &Router)
                .await
                .unwrap();
        });
        let response = request(
            &path,
            json!({"nonce":"wrong","verb":"run.status","args":[]}),
        )
        .await;
        server.await.unwrap();
        assert_eq!(response["error"]["kind"], "permission_denied");
        assert_eq!(
            response["error"]["message"],
            "agent socket capability refused"
        );
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(test)]
mod registration {
    use super::*;

    #[test]
    fn host_registration_round_trips_and_replaces_capabilities() {
        let root = std::env::temp_dir().join(format!("locus-run-registry-{}", Uuid::new_v4()));
        let socket = root.join("locus.sock");
        let registration = AgentRunRegistration {
            run_id: RunId::generate(),
            nonce: "nonce".into(),
            lsp_enabled: true,
            debug_adapters: vec!["debugpy".into()],
            debug_configs: BTreeMap::from([(
                "python".into(),
                crate::services::project::DebugRunConfig::new("debugpy", "python -m app").unwrap(),
            )]),
        };
        let path = write_agent_registration(&socket, &registration).unwrap();
        let registrations = read_agent_registrations(agent_registration_root(&socket)).unwrap();
        assert_eq!(
            registrations.as_slice(),
            std::slice::from_ref(&registration)
        );
        let capabilities = AgentSocketCapabilities::default();
        capabilities.replace(&registrations).unwrap();
        let (runs, lsp_runs) = capabilities.snapshot().unwrap();
        assert_eq!(runs["nonce"], registration.run_id);
        assert!(lsp_runs.contains(&registration.run_id));
        assert!(capabilities
            .debug_adapters(registration.run_id)
            .unwrap()
            .contains("debugpy"));
        assert_eq!(
            capabilities
                .debug_config(registration.run_id, "python")
                .unwrap()
                .unwrap()
                .command(),
            "python -m app"
        );
        remove_agent_registration(path);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reconciliation_rejects_multiple_nonces_for_one_run() {
        let run_id = RunId::generate();
        let registrations = vec![
            AgentRunRegistration {
                run_id,
                nonce: "first".into(),
                lsp_enabled: false,
                debug_adapters: Vec::new(),
                debug_configs: BTreeMap::new(),
            },
            AgentRunRegistration {
                run_id,
                nonce: "second".into(),
                lsp_enabled: true,
                debug_adapters: Vec::new(),
                debug_configs: BTreeMap::new(),
            },
        ];
        let capabilities = AgentSocketCapabilities::default();
        let error = capabilities.replace(&registrations).unwrap_err();
        assert!(error
            .to_string()
            .contains("duplicate agent registration run id"));
    }
}
