//! Headless daemon lifetime and the authenticated agent socket.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
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
    run::{self, ContainerRuntime, SpawnRequest, SpawnedRun},
    session::Run,
    store::Store,
};

const MAX_AGENT_SOCKET_FRAME_BYTES: u32 = 1_048_576;

/// `locusd` owns active runs. Desktop windows attach and detach without owning them.
#[derive(Default)]
pub struct Daemon {
    active_runs: BTreeSet<Uuid>,
    attached_windows: usize,
}

impl Daemon {
    pub fn attach_window(&mut self) {
        self.attached_windows += 1;
    }
    pub fn detach_window(&mut self) {
        self.attached_windows = self.attached_windows.saturating_sub(1);
    }
    pub fn begin_run(&mut self, run_id: Uuid) {
        self.active_runs.insert(run_id);
    }
    pub fn finish_run(&mut self, run_id: Uuid) {
        self.active_runs.remove(&run_id);
    }
    pub fn tracks(&self, run_id: Uuid) -> bool {
        self.active_runs.contains(&run_id)
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
    pub verb: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AgentSocketResponse {
    result: Option<Value>,
    error: Option<String>,
}

/// Domain routing remains in the core, never in the container CLI.
pub trait AgentSocketRouter: Send + Sync {
    fn route(&self, run_id: Uuid, verb: &str, args: &[String]) -> Result<Value>;
}

/// Bind a daemon-owned socket. Its parent must be host-owned and inaccessible to agents.
pub fn bind_agent_socket(path: impl AsRef<Path>) -> Result<UnixListener> {
    let path = path.as_ref();
    if path.exists() {
        std::fs::remove_file(path)
            .with_context(|| format!("remove stale socket {}", path.display()))?;
    }
    UnixListener::bind(path).with_context(|| format!("bind agent socket {}", path.display()))
}

/// Serve a single connection, which makes the boundary independently testable. A daemon calls
/// this in its accept loop and retains the capability map for the run's lifetime.
pub async fn serve_agent_socket_once(
    listener: &UnixListener,
    capabilities: &BTreeMap<String, Uuid>,
    router: &impl AgentSocketRouter,
) -> Result<()> {
    let (mut stream, _) = listener
        .accept()
        .await
        .context("accept agent socket client")?;
    let request: AgentSocketRequest = read_frame(&mut stream).await?;
    let response = match capabilities.get(&request.nonce) {
        Some(run_id) => match router.route(*run_id, &request.verb, &request.args) {
            Ok(result) => AgentSocketResponse {
                result: Some(result),
                error: None,
            },
            Err(error) => AgentSocketResponse {
                result: None,
                error: Some(error.to_string()),
            },
        },
        None => AgentSocketResponse {
            result: None,
            error: Some("agent socket capability refused".into()),
        },
    };
    write_frame(&mut stream, &response).await
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
    use uuid::Uuid;
    #[test]
    fn background_daemon_keeps_runs_when_the_last_window_closes() {
        let run_id = Uuid::new_v4();
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
        bind_agent_socket, read_frame, serve_agent_socket_once, AgentSocketRouter,
        MAX_AGENT_SOCKET_FRAME_BYTES,
    };
    use anyhow::Result;
    use serde_json::json;
    use std::{collections::BTreeMap, path::PathBuf};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::UnixStream,
    };
    use uuid::Uuid;

    struct Router;
    impl AgentSocketRouter for Router {
        fn route(&self, run_id: Uuid, verb: &str, _: &[String]) -> Result<serde_json::Value> {
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
        let run_id = Uuid::new_v4();
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
        assert_eq!(response["error"], "agent socket capability refused");
        let _ = std::fs::remove_file(path);
    }
}
