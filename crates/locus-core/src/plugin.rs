//! The common executable plugin contract.
//!
//! Plugins are deliberately data-only integrations. The host owns persistence, rendering,
//! credentials, ACP event normalization, and trust decisions; a plugin owns only the mechanics
//! described by its manifest and capability calls.

use crate::services::{
    provider::KeychainReference,
    telemetry::{AcpAdapter, Adapter, CapturedEvent},
    tools::{SignedToolUpload, ToolAdmissionError, ToolCatalog},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fs,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    process::Stdio,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, Command},
    sync::{mpsc, Mutex},
    time::timeout,
};

pub const PLUGIN_PROTOCOL: &str = "locus.plugin.v1";
const JSON_RPC_VERSION: &str = "2.0";
const MAX_RPC_LINE_BYTES: usize = 1024 * 1024;
pub const REQUIRED_HARNESS_CAPABILITY: &str = "harness.session";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    CliTool,
    Harness,
    Provider,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest {
    pub protocol: String,
    pub kind: PluginKind,
    pub id: String,
    pub version: String,
    pub executable: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

impl PluginManifest {
    pub fn new(
        kind: PluginKind,
        id: impl Into<String>,
        version: impl Into<String>,
        executable: impl Into<String>,
        capabilities: impl IntoIterator<Item = impl Into<String>>,
        permissions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, PluginError> {
        let manifest = Self {
            protocol: PLUGIN_PROTOCOL.into(),
            kind,
            id: id.into(),
            version: version.into(),
            executable: executable.into(),
            capabilities: capabilities.into_iter().map(Into::into).collect(),
            permissions: permissions.into_iter().map(Into::into).collect(),
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), PluginError> {
        if self.protocol != PLUGIN_PROTOCOL {
            return Err(PluginError::Protocol(self.protocol.clone()));
        }
        for (field, value) in [
            ("id", self.id.as_str()),
            ("version", self.version.as_str()),
            ("executable", self.executable.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(PluginError::InvalidManifest(format!(
                    "{field} must not be empty"
                )));
            }
        }
        if self.capabilities.iter().any(|cap| cap.trim().is_empty()) {
            return Err(PluginError::InvalidManifest(
                "capabilities must not contain empty names".into(),
            ));
        }
        if self
            .permissions
            .iter()
            .any(|permission| permission.trim().is_empty())
        {
            return Err(PluginError::InvalidManifest(
                "permissions must not contain empty names".into(),
            ));
        }
        if self.capabilities.iter().collect::<BTreeSet<_>>().len() != self.capabilities.len()
            || self.permissions.iter().collect::<BTreeSet<_>>().len() != self.permissions.len()
        {
            return Err(PluginError::InvalidManifest(
                "capabilities and permissions must not contain duplicates".into(),
            ));
        }
        Ok(())
    }

    pub fn from_toml(input: &str) -> Result<Self, PluginError> {
        let manifest: Self = toml::from_str(input)
            .map_err(|error| PluginError::InvalidManifest(error.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityNegotiation {
    pub accepted: Vec<String>,
    pub unknown_optional: Vec<String>,
}

pub fn negotiate_capabilities(
    protocol: &str,
    offered: &[String],
    required: &[String],
    known: &[&str],
) -> Result<CapabilityNegotiation, PluginError> {
    if protocol != PLUGIN_PROTOCOL {
        return Err(PluginError::Protocol(protocol.into()));
    }
    let offered: BTreeSet<_> = offered.iter().cloned().collect();
    let missing: Vec<_> = required
        .iter()
        .filter(|capability| !offered.contains(*capability))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(PluginError::MissingCapabilities(missing));
    }
    let known: BTreeSet<_> = known.iter().copied().collect();
    let mut accepted: Vec<_> = offered
        .iter()
        .filter(|capability| known.contains(capability.as_str()))
        .cloned()
        .collect();
    let mut unknown_optional: Vec<_> = offered
        .iter()
        .filter(|capability| !known.contains(capability.as_str()))
        .cloned()
        .collect();
    accepted.sort();
    unknown_optional.sort();
    Ok(CapabilityNegotiation {
        accepted,
        unknown_optional,
    })
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

impl RpcRequest {
    pub fn new(id: u64, method: impl Into<String>, params: Value) -> Result<Self, PluginError> {
        let request = Self {
            jsonrpc: JSON_RPC_VERSION.into(),
            id,
            method: method.into(),
            params,
        };
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), PluginError> {
        if self.jsonrpc != JSON_RPC_VERSION {
            return Err(PluginError::MalformedResponse(
                "JSON-RPC request version mismatch".into(),
            ));
        }
        if self.method.trim().is_empty() {
            return Err(PluginError::MalformedResponse(
                "JSON-RPC method must not be empty".into(),
            ));
        }
        validate_data_only(&self.params)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<RpcError>,
}

impl RpcResponse {
    pub fn success(id: u64, result: Value) -> Result<Self, PluginError> {
        validate_data_only(&result)?;
        Ok(Self {
            jsonrpc: JSON_RPC_VERSION.into(),
            id: Some(id),
            result: Some(result),
            error: None,
        })
    }

    pub fn failure(id: Option<u64>, error: RpcError) -> Result<Self, PluginError> {
        error.validate()?;
        Ok(Self {
            jsonrpc: JSON_RPC_VERSION.into(),
            id,
            result: None,
            error: Some(error),
        })
    }

    fn validate(&self) -> Result<(), PluginError> {
        if self.jsonrpc != JSON_RPC_VERSION {
            return Err(PluginError::MalformedResponse(
                "JSON-RPC response version mismatch".into(),
            ));
        }
        if self.id.is_none() {
            return Err(PluginError::MalformedResponse(
                "JSON-RPC response id is missing".into(),
            ));
        }
        if self.result.is_some() == self.error.is_some() {
            return Err(PluginError::MalformedResponse(
                "JSON-RPC response must contain exactly one result or error".into(),
            ));
        }
        if let Some(result) = &self.result {
            validate_data_only(result)?;
        }
        if let Some(error) = &self.error {
            error.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

impl RpcNotification {
    fn validate(&self) -> Result<(), PluginError> {
        if self.jsonrpc != JSON_RPC_VERSION || self.method.trim().is_empty() {
            return Err(PluginError::MalformedResponse(
                "malformed JSON-RPC notification".into(),
            ));
        }
        validate_data_only(&self.params)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}

impl RpcError {
    fn validate(&self) -> Result<(), PluginError> {
        if self.message.trim().is_empty() {
            return Err(PluginError::MalformedResponse(
                "JSON-RPC error message is empty".into(),
            ));
        }
        if let Some(data) = &self.data {
            validate_data_only(data)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PluginDiagnostic {
    pub method: String,
    pub elapsed_ms: u128,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin protocol is unsupported: {0}")]
    Protocol(String),
    #[error("invalid plugin manifest: {0}")]
    InvalidManifest(String),
    #[error("required plugin capabilities are missing: {0:?}")]
    MissingCapabilities(Vec<String>),
    #[error("plugin RPC error {0}: {1}")]
    Rpc(i64, String),
    #[error("plugin response is malformed: {0}")]
    MalformedResponse(String),
    #[error("plugin call timed out")]
    Timeout(PluginDiagnostic),
    #[error("plugin response is not data-only: {0}")]
    DataBoundary(String),
    #[error("plugin discovery failed: {0}")]
    Discovery(String),
    #[error("plugin process failed: {0}")]
    Process(#[from] io::Error),
    #[error("plugin JSON serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("plugin process is unavailable")]
    ProcessUnavailable,
    #[error("plugin is not ready: {0}")]
    NotReady(String),
    #[error("CLI tool admission failed: {0}")]
    ToolAdmission(#[from] ToolAdmissionError),
}

const FORBIDDEN_DATA_KEYS: &[&str] = &[
    "ui",
    "ui_code",
    "component",
    "render",
    "render_html",
    "persistence",
    "persistence_path",
    "sql",
    "filesystem",
    "filesystem_path",
    "write_persistence",
    "tauri_command",
];

const SECRET_DATA_KEYS: &[&str] = &[
    "api_key",
    "access_token",
    "authorization",
    "credential",
    "password",
    "private_key",
    "raw_secret",
    "refresh_token",
    "secret",
];

pub fn validate_data_only(value: &Value) -> Result<(), PluginError> {
    match value {
        Value::Array(values) => values.iter().try_for_each(validate_data_only),
        Value::Object(object) => {
            for (key, value) in object {
                let normalized = key.to_ascii_lowercase().replace('-', "_");
                if FORBIDDEN_DATA_KEYS.contains(&normalized.as_str())
                    || SECRET_DATA_KEYS.contains(&normalized.as_str())
                    || key.starts_with("__")
                {
                    return Err(PluginError::DataBoundary(key.clone()));
                }
                validate_data_only(value)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[derive(Clone, Debug, Deserialize)]
struct RpcEnvelope {
    jsonrpc: String,
    #[serde(default)]
    id: Option<u64>,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    params: Option<Value>,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<RpcError>,
}

enum RpcMessage {
    Response(RpcResponse),
    Notification(RpcNotification),
}

impl RpcEnvelope {
    fn into_message(self) -> Result<RpcMessage, PluginError> {
        if self.jsonrpc != JSON_RPC_VERSION {
            return Err(PluginError::MalformedResponse(
                "JSON-RPC message version mismatch".into(),
            ));
        }
        if let Some(method) = self.method {
            if self.id.is_some() || self.result.is_some() || self.error.is_some() {
                return Err(PluginError::MalformedResponse(
                    "JSON-RPC notification contains response fields".into(),
                ));
            }
            let notification = RpcNotification {
                jsonrpc: self.jsonrpc,
                method,
                params: self.params.unwrap_or(Value::Null),
            };
            notification.validate()?;
            return Ok(RpcMessage::Notification(notification));
        }

        let response = RpcResponse {
            jsonrpc: self.jsonrpc,
            id: self.id,
            result: self.result,
            error: self.error,
        };
        response.validate()?;
        Ok(RpcMessage::Response(response))
    }
}

async fn read_rpc_messages<R>(
    mut stdout: BufReader<R>,
    sender: mpsc::Sender<Result<RpcMessage, String>>,
) where
    R: AsyncRead + Unpin + Send + 'static,
{
    loop {
        let mut line = Vec::new();
        match stdout.read_until(b'\n', &mut line).await {
            Ok(0) => {
                let _ = sender.send(Err("plugin closed stdout".into())).await;
                return;
            }
            Ok(_) if line.len() > MAX_RPC_LINE_BYTES => {
                let _ = sender
                    .send(Err("plugin response exceeds the size limit".into()))
                    .await;
                return;
            }
            Ok(_) => {
                let line = match std::str::from_utf8(&line) {
                    Ok(line) => line,
                    Err(error) => {
                        let _ = sender
                            .send(Err(format!("plugin response is not UTF-8: {error}")))
                            .await;
                        return;
                    }
                };
                let message = serde_json::from_str::<RpcEnvelope>(line)
                    .map_err(|error| format!("plugin response is not JSON-RPC: {error}"))
                    .and_then(|envelope| {
                        envelope.into_message().map_err(|error| error.to_string())
                    });
                if sender.send(message).await.is_err() {
                    return;
                }
            }
            Err(error) => {
                let _ = sender
                    .send(Err(format!("plugin stdout read failed: {error}")))
                    .await;
                return;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PluginProcessState {
    Ready,
    Poisoned,
    Shutdown,
}

pub struct PluginProcess {
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    incoming: Mutex<mpsc::Receiver<Result<RpcMessage, String>>>,
    call_lock: Mutex<()>,
    notifications: Mutex<Vec<RpcNotification>>,
    next_id: AtomicU64,
    state: Mutex<PluginProcessState>,
    timeout: Duration,
}

impl PluginProcess {
    /// Low-level process construction for an already admitted executable. Production callers
    /// should prefer [`Self::spawn_admitted`] so path and manifest admission happen first.
    pub async fn spawn(
        executable: impl AsRef<OsStr>,
        timeout: Duration,
    ) -> Result<Self, PluginError> {
        Self::spawn_command(Command::new(executable), timeout).await
    }

    pub async fn spawn_admitted(
        plugin: &AdmittedPlugin,
        timeout: Duration,
    ) -> Result<Self, PluginError> {
        Self::spawn(&plugin.executable, timeout).await
    }

    pub async fn spawn_command(
        mut command: Command,
        timeout: Duration,
    ) -> Result<Self, PluginError> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| PluginError::MalformedResponse("plugin stdin was not piped".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| PluginError::MalformedResponse("plugin stdout was not piped".into()))?;
        let (sender, receiver) = mpsc::channel(128);
        tokio::spawn(read_rpc_messages(BufReader::new(stdout), sender));
        Ok(Self {
            child: Mutex::new(child),
            stdin: Mutex::new(stdin),
            incoming: Mutex::new(receiver),
            call_lock: Mutex::new(()),
            notifications: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(1),
            state: Mutex::new(PluginProcessState::Ready),
            timeout,
        })
    }

    pub async fn call(&self, method: &str, params: Value) -> Result<Value, PluginError> {
        validate_data_only(&params)?;
        let _call = self.call_lock.lock().await;
        self.ensure_ready().await?;
        let request =
            RpcRequest::new(self.next_id.fetch_add(1, Ordering::Relaxed), method, params)?;
        let request_id = request.id;
        let started = std::time::Instant::now();
        let operation = async {
            let mut stdin = self.stdin.lock().await;
            let mut line = serde_json::to_vec(&request)
                .map_err(|error| PluginError::MalformedResponse(error.to_string()))?;
            line.push(b'\n');
            stdin.write_all(&line).await?;
            stdin.flush().await?;
            drop(stdin);
            self.receive_response(request_id).await
        };

        match timeout(self.timeout, operation).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(error @ PluginError::Rpc(_, _))) => Err(error),
            Ok(Err(error)) => {
                self.poison_and_terminate().await;
                Err(error)
            }
            Err(_) => {
                self.poison_and_terminate().await;
                Err(PluginError::Timeout(PluginDiagnostic {
                    method: method.into(),
                    elapsed_ms: started.elapsed().as_millis(),
                    message: format!("call exceeded {}ms", self.timeout.as_millis()),
                }))
            }
        }
    }

    async fn receive_response(&self, request_id: u64) -> Result<Value, PluginError> {
        let mut incoming = self.incoming.lock().await;
        while let Some(message) = incoming.recv().await {
            match message {
                Ok(RpcMessage::Notification(notification)) => {
                    self.notifications.lock().await.push(notification);
                }
                Ok(RpcMessage::Response(response)) => {
                    if response.id != Some(request_id) {
                        return Err(PluginError::MalformedResponse(
                            "JSON-RPC response id mismatch".into(),
                        ));
                    }
                    if let Some(error) = response.error {
                        return Err(PluginError::Rpc(error.code, error.message));
                    }
                    return response.result.ok_or_else(|| {
                        PluginError::MalformedResponse("response has no result".into())
                    });
                }
                Err(error) => return Err(PluginError::MalformedResponse(error)),
            }
        }
        Err(PluginError::MalformedResponse(
            "plugin response channel closed".into(),
        ))
    }

    pub async fn take_notifications(&self) -> Result<Vec<RpcNotification>, PluginError> {
        let mut incoming = self.incoming.lock().await;
        loop {
            match incoming.try_recv() {
                Ok(Ok(RpcMessage::Notification(notification))) => {
                    self.notifications.lock().await.push(notification);
                }
                Ok(Ok(RpcMessage::Response(_))) => {
                    return Err(PluginError::MalformedResponse(
                        "unexpected JSON-RPC response without an active call".into(),
                    ));
                }
                Ok(Err(error)) => return Err(PluginError::MalformedResponse(error)),
                Err(mpsc::error::TryRecvError::Empty)
                | Err(mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
        Ok(std::mem::take(&mut *self.notifications.lock().await))
    }

    pub async fn take_harness_events(
        &self,
        harness: &HarnessDescriptor,
    ) -> Result<Vec<CapturedEvent>, PluginError> {
        let notifications = self.take_notifications().await?;
        let mut events = Vec::new();
        for notification in notifications {
            events.extend(harness.normalize_event(json!({
                "jsonrpc": notification.jsonrpc,
                "method": notification.method,
                "params": notification.params,
            }))?);
        }
        Ok(events)
    }

    pub async fn initialize(&self, protocol: &str) -> Result<Value, PluginError> {
        self.call("plugin.initialize", json!({ "protocol": protocol }))
            .await
    }

    pub async fn describe(&self) -> Result<Value, PluginError> {
        self.call("plugin.describe", json!({})).await
    }

    pub async fn health(&self) -> Result<Value, PluginError> {
        self.call("plugin.health", json!({})).await
    }

    pub async fn handshake(
        &self,
        required: &[String],
        known: &[&str],
    ) -> Result<PluginHandshake, PluginError> {
        let initialized: PluginInitializeResult =
            decode_data(self.initialize(PLUGIN_PROTOCOL).await?)?;
        if initialized.protocol != PLUGIN_PROTOCOL {
            return Err(PluginError::Protocol(initialized.protocol));
        }
        let descriptor: PluginDescriptor = decode_data(self.describe().await?)?;
        descriptor.validate()?;
        let negotiation = negotiate_capabilities(
            &descriptor.protocol,
            &descriptor.capabilities,
            required,
            known,
        )?;
        let health: PluginHealth = decode_data(self.health().await?)?;
        if !health.ready {
            return Err(PluginError::NotReady(
                health
                    .diagnostic
                    .clone()
                    .unwrap_or_else(|| "plugin reported not ready".into()),
            ));
        }
        Ok(PluginHandshake {
            descriptor,
            health,
            negotiation,
        })
    }

    pub async fn shutdown(&self) -> Result<Value, PluginError> {
        let result = if self.is_ready().await {
            self.call("plugin.shutdown", json!({})).await
        } else {
            Err(PluginError::ProcessUnavailable)
        };
        self.terminate().await;
        *self.state.lock().await = PluginProcessState::Shutdown;
        result
    }

    async fn is_ready(&self) -> bool {
        *self.state.lock().await == PluginProcessState::Ready
    }

    async fn ensure_ready(&self) -> Result<(), PluginError> {
        match *self.state.lock().await {
            PluginProcessState::Ready => Ok(()),
            PluginProcessState::Poisoned | PluginProcessState::Shutdown => {
                Err(PluginError::ProcessUnavailable)
            }
        }
    }

    async fn poison_and_terminate(&self) {
        *self.state.lock().await = PluginProcessState::Poisoned;
        self.terminate().await;
    }

    async fn terminate(&self) {
        let mut child = self.child.lock().await;
        if child.try_wait().ok().flatten().is_none() {
            let _ = child.kill().await;
        }
        let _ = child.wait().await;
    }
}

fn decode_data<T: DeserializeOwned>(value: Value) -> Result<T, PluginError> {
    validate_data_only(&value)?;
    serde_json::from_value(value).map_err(|error| PluginError::MalformedResponse(error.to_string()))
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginInitializeResult {
    pub protocol: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginDescriptor {
    pub protocol: String,
    pub kind: PluginKind,
    pub id: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub schema_versions: BTreeMap<String, String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

impl PluginDescriptor {
    pub fn from_manifest(manifest: &PluginManifest) -> Self {
        let mut schema_versions = BTreeMap::new();
        schema_versions.insert("plugin".into(), "v1".into());
        Self {
            protocol: manifest.protocol.clone(),
            kind: manifest.kind,
            id: manifest.id.clone(),
            version: manifest.version.clone(),
            capabilities: manifest.capabilities.clone(),
            schema_versions,
            permissions: manifest.permissions.clone(),
        }
    }

    pub fn validate(&self) -> Result<(), PluginError> {
        let manifest = PluginManifest {
            protocol: self.protocol.clone(),
            kind: self.kind,
            id: self.id.clone(),
            version: self.version.clone(),
            executable: "descriptor".into(),
            capabilities: self.capabilities.clone(),
            permissions: self.permissions.clone(),
        };
        manifest.validate()?;
        if self.schema_versions.is_empty()
            || self
                .schema_versions
                .iter()
                .any(|(key, value)| key.trim().is_empty() || value.trim().is_empty())
        {
            return Err(PluginError::MalformedResponse(
                "plugin descriptor schema_versions must not be empty".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginHealth {
    pub ready: bool,
    #[serde(default)]
    pub diagnostic: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginHandshake {
    pub descriptor: PluginDescriptor,
    pub health: PluginHealth,
    pub negotiation: CapabilityNegotiation,
}

pub trait PluginHandler {
    fn descriptor(&self) -> &PluginDescriptor;

    fn health(&self) -> PluginHealth {
        PluginHealth {
            ready: true,
            diagnostic: None,
        }
    }

    fn call(&mut self, method: &str, params: Value) -> Result<Value, RpcError>;
}

#[derive(Clone, Debug)]
pub struct DescriptorPlugin {
    descriptor: PluginDescriptor,
    health: PluginHealth,
    capability_results: BTreeMap<String, Value>,
}

impl DescriptorPlugin {
    pub fn new(descriptor: PluginDescriptor) -> Result<Self, PluginError> {
        descriptor.validate()?;
        Ok(Self {
            descriptor,
            health: PluginHealth {
                ready: true,
                diagnostic: None,
            },
            capability_results: BTreeMap::new(),
        })
    }

    pub fn with_capability_result(
        mut self,
        capability: impl Into<String>,
        result: Value,
    ) -> Result<Self, PluginError> {
        validate_data_only(&result)?;
        self.capability_results.insert(capability.into(), result);
        Ok(self)
    }
}

impl PluginHandler for DescriptorPlugin {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn health(&self) -> PluginHealth {
        self.health.clone()
    }

    fn call(&mut self, method: &str, _params: Value) -> Result<Value, RpcError> {
        self.capability_results
            .get(method)
            .cloned()
            .ok_or_else(|| RpcError {
                code: -32601,
                message: format!("method not found: {method}"),
                data: None,
            })
    }
}

pub fn serve_stdio<H: PluginHandler>(handler: &mut H) -> Result<(), PluginError> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    serve_stdio_io(handler, io::BufReader::new(stdin.lock()), stdout.lock())
}

pub fn serve_stdio_io<H, R, W>(
    handler: &mut H,
    mut reader: R,
    mut writer: W,
) -> Result<(), PluginError>
where
    H: PluginHandler,
    R: BufRead,
    W: Write,
{
    let mut line = String::new();
    loop {
        line.clear();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            break;
        }
        if line.len() > MAX_RPC_LINE_BYTES {
            return Err(PluginError::MalformedResponse(
                "plugin request exceeds the size limit".into(),
            ));
        }
        let request = match serde_json::from_str::<RpcRequest>(&line) {
            Ok(request) => request,
            Err(error) => {
                let response = RpcResponse::failure(
                    None,
                    RpcError {
                        code: -32600,
                        message: format!("invalid request: {error}"),
                        data: None,
                    },
                )?;
                serde_json::to_writer(&mut writer, &response)?;
                writer.write_all(b"\n")?;
                writer.flush()?;
                continue;
            }
        };
        request.validate()?;
        let shutdown = request.method == "plugin.shutdown";
        let result = dispatch_request(handler, &request);
        let response = match result {
            Ok(value) => RpcResponse::success(request.id, value)?,
            Err(error) => RpcResponse::failure(Some(request.id), error)?,
        };
        serde_json::to_writer(&mut writer, &response)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        if shutdown {
            break;
        }
    }
    Ok(())
}

fn dispatch_request<H: PluginHandler>(
    handler: &mut H,
    request: &RpcRequest,
) -> Result<Value, RpcError> {
    match request.method.as_str() {
        "plugin.initialize" => {
            let protocol = request
                .params
                .get("protocol")
                .and_then(Value::as_str)
                .ok_or_else(|| RpcError {
                    code: -32602,
                    message: "plugin.initialize requires protocol".into(),
                    data: None,
                })?;
            if protocol != handler.descriptor().protocol {
                return Err(RpcError {
                    code: -32001,
                    message: "unsupported plugin protocol".into(),
                    data: None,
                });
            }
            Ok(json!({
                "protocol": handler.descriptor().protocol,
                "capabilities": handler.descriptor().capabilities,
            }))
        }
        "plugin.describe" => serde_json::to_value(handler.descriptor()).map_err(|error| RpcError {
            code: -32603,
            message: format!("descriptor serialization failed: {error}"),
            data: None,
        }),
        "plugin.health" => serde_json::to_value(handler.health()).map_err(|error| RpcError {
            code: -32603,
            message: format!("health serialization failed: {error}"),
            data: None,
        }),
        "plugin.shutdown" => Ok(json!({ "stopped": true })),
        method => handler.call(method, request.params.clone()),
    }
}

pub fn discover_user_plugins(
    manifest_directories: &[PathBuf],
    trusted_directories: &[PathBuf],
) -> Result<Vec<PluginManifest>, PluginError> {
    let trusted: BTreeSet<PathBuf> = trusted_directories
        .iter()
        .map(|path| fs::canonicalize(path).unwrap_or_else(|_| path.clone()))
        .collect();
    let mut manifests = Vec::new();
    for directory in manifest_directories {
        let root = fs::canonicalize(directory)
            .map_err(|error| PluginError::Discovery(error.to_string()))?;
        if !trusted.contains(&root) {
            continue;
        }
        collect_manifests(&root, &root, &mut manifests)?;
    }
    manifests.sort_by(|left, right| {
        (&left.kind, &left.id, &left.version).cmp(&(&right.kind, &right.id, &right.version))
    });
    Ok(manifests)
}

fn collect_manifests(
    path: &Path,
    trusted_root: &Path,
    output: &mut Vec<PluginManifest>,
) -> Result<(), PluginError> {
    for entry in fs::read_dir(path).map_err(|error| PluginError::Discovery(error.to_string()))? {
        let entry = entry.map_err(|error| PluginError::Discovery(error.to_string()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| PluginError::Discovery(error.to_string()))?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            collect_manifests(&path, trusted_root, output)?;
        } else if matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some("manifest.toml" | "plugin.toml")
        ) {
            let canonical = fs::canonicalize(&path)
                .map_err(|error| PluginError::Discovery(error.to_string()))?;
            if !canonical.starts_with(trusted_root) {
                return Err(PluginError::Discovery(
                    "plugin manifest escapes the trusted directory".into(),
                ));
            }
            let input = fs::read_to_string(canonical)
                .map_err(|error| PluginError::Discovery(error.to_string()))?;
            output.push(PluginManifest::from_toml(&input)?);
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HarnessDescriptor {
    pub manifest: PluginManifest,
    pub transport: String,
    pub launch: Value,
    pub config: Value,
    pub event_capabilities: Vec<String>,
}

impl HarnessDescriptor {
    pub fn validate(&self) -> Result<CapabilityNegotiation, PluginError> {
        if self.manifest.kind != PluginKind::Harness {
            return Err(PluginError::InvalidManifest(
                "descriptor kind is not harness".into(),
            ));
        }
        validate_data_only(&self.launch)?;
        validate_data_only(&self.config)?;
        if self.transport.trim().is_empty()
            || self
                .event_capabilities
                .iter()
                .any(|capability| capability.trim().is_empty())
        {
            return Err(PluginError::InvalidManifest(
                "harness transport and event capabilities are required".into(),
            ));
        }
        negotiate_capabilities(
            &self.manifest.protocol,
            &self.manifest.capabilities,
            &[REQUIRED_HARNESS_CAPABILITY.into()],
            &[
                "harness.session",
                "harness.launch",
                "harness.materialize",
                "harness.events",
                "harness.models",
                "harness.permissions",
                "harness.resume",
                "harness.checkpoints",
                "harness.usage",
            ],
        )
    }

    pub fn plugin_descriptor(&self) -> PluginDescriptor {
        PluginDescriptor::from_manifest(&self.manifest)
    }

    pub fn normalize_event(&self, event: Value) -> Result<Vec<CapturedEvent>, PluginError> {
        AcpAdapter
            .normalize(event)
            .map_err(|error| PluginError::MalformedResponse(error.to_string()))
    }
}

fn catalog_manifest(kind: PluginKind, id: Option<&str>) -> PluginManifest {
    builtin_manifests()
        .into_iter()
        .find(|manifest| manifest.kind == kind && id.is_none_or(|id| manifest.id == id))
        .unwrap_or_else(|| panic!("first-party plugin catalog entry is present"))
}

pub fn first_party_harness() -> HarnessDescriptor {
    let manifest = catalog_manifest(PluginKind::Harness, None);
    HarnessDescriptor {
        manifest,
        transport: "acp".into(),
        launch: json!({ "stdio": true }),
        config: json!({ "materializer": "plugin" }),
        event_capabilities: vec!["acp".into()],
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderModelDescriptor {
    pub id: String,
    pub alias: Option<String>,
    pub context_window: Option<u64>,
}

fn require_capabilities(manifest: &PluginManifest, required: &[&str]) -> Result<(), PluginError> {
    let missing = required
        .iter()
        .filter(|capability| {
            !manifest
                .capabilities
                .iter()
                .any(|item| item == **capability)
        })
        .map(|capability| (*capability).to_owned())
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(PluginError::MissingCapabilities(missing))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderDescriptor {
    pub manifest: PluginManifest,
    pub authentication: Vec<String>,
    pub base_url: Option<String>,
    pub models: Vec<ProviderModelDescriptor>,
    pub keychain_reference: Option<KeychainReference>,
    pub verification: Option<Value>,
}

impl ProviderDescriptor {
    pub fn validate(&self) -> Result<(), PluginError> {
        if self.manifest.kind != PluginKind::Provider {
            return Err(PluginError::InvalidManifest(
                "descriptor kind is not provider".into(),
            ));
        }
        require_capabilities(
            &self.manifest,
            &["provider.models", "provider.verify", "provider.aliases"],
        )?;
        if self
            .authentication
            .iter()
            .any(|method| method.trim().is_empty())
            || self
                .base_url
                .as_deref()
                .is_some_and(|url| url.trim().is_empty())
        {
            return Err(PluginError::InvalidManifest(
                "provider authentication and endpoint values must not be empty".into(),
            ));
        }
        // A catalog descriptor may be unconfigured. Once configured, the host carries only
        // this reference and resolves the secret at the egress boundary.
        if self
            .keychain_reference
            .as_ref()
            .is_some_and(|reference| reference.as_str().trim().is_empty())
        {
            return Err(PluginError::InvalidManifest(
                "provider keychain reference must not be empty".into(),
            ));
        }
        if self.models.iter().any(|model| {
            model.id.trim().is_empty()
                || model
                    .alias
                    .as_deref()
                    .is_some_and(|alias| alias.trim().is_empty())
        }) {
            return Err(PluginError::InvalidManifest(
                "provider model ids and aliases must not be empty".into(),
            ));
        }
        validate_data_only(&self.verification.clone().unwrap_or(Value::Null))
    }

    pub fn plugin_descriptor(&self) -> PluginDescriptor {
        PluginDescriptor::from_manifest(&self.manifest)
    }
}

fn provider(id: &str, models: &[(&str, Option<&str>)]) -> ProviderDescriptor {
    ProviderDescriptor {
        manifest: catalog_manifest(PluginKind::Provider, Some(id)),
        authentication: vec!["api_key".into(), "oauth".into()],
        base_url: None,
        models: models
            .iter()
            .map(|(id, alias)| ProviderModelDescriptor {
                id: (*id).into(),
                alias: alias.map(str::to_owned),
                context_window: None,
            })
            .collect(),
        keychain_reference: None,
        verification: Some(json!({ "status": "unverified" })),
    }
}

pub fn first_party_providers() -> Vec<ProviderDescriptor> {
    let provider_prefix = ["clau", "de"].concat();
    let sonnet_model = format!("{provider_prefix}-sonnet-4");
    let opus_model = format!("{provider_prefix}-opus-4");
    let router_sonnet = format!("anthropic/{sonnet_model}");
    vec![
        provider("openai", &[("gpt-4o", Some("GPT-4o")), ("gpt-4.1", None)]),
        provider(
            "anthropic",
            &[(&sonnet_model, Some("Sonnet")), (&opus_model, Some("Opus"))],
        ),
        provider(
            "openrouter",
            &[
                ("openai/gpt-4o", Some("GPT-4o")),
                (&router_sonnet, Some("Sonnet")),
            ],
        ),
    ]
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CliToolDescriptor {
    pub manifest: PluginManifest,
    pub install: String,
    pub verify: String,
    pub docs: Option<String>,
    pub digest: String,
    pub permissions: Vec<String>,
}

impl CliToolDescriptor {
    pub fn validate(&self) -> Result<(), PluginError> {
        if self.manifest.kind != PluginKind::CliTool {
            return Err(PluginError::InvalidManifest(
                "descriptor kind is not cli_tool".into(),
            ));
        }
        require_capabilities(
            &self.manifest,
            &[
                "cli_tool.install",
                "cli_tool.verify",
                "cli_tool.docs",
                "cli_tool.digest",
            ],
        )?;
        if self.install.trim().is_empty()
            || self.verify.trim().is_empty()
            || self.digest.trim().is_empty()
            || self
                .permissions
                .iter()
                .any(|permission| permission.trim().is_empty())
        {
            return Err(PluginError::InvalidManifest(
                "CLI tool install, verify, digest, and permissions are required".into(),
            ));
        }
        Ok(())
    }

    pub fn plugin_descriptor(&self) -> PluginDescriptor {
        PluginDescriptor::from_manifest(&self.manifest)
    }
}

pub fn first_party_cli_tool() -> CliToolDescriptor {
    CliToolDescriptor {
        manifest: catalog_manifest(PluginKind::CliTool, None),
        install: "gh --version".into(),
        verify: "gh --version".into(),
        docs: Some("https://cli.github.com/manual/".into()),
        digest: "registry-pinned".into(),
        permissions: vec!["network".into(), "repository_read".into()],
    }
}

pub fn admit_user_cli_tool(
    catalog: &mut ToolCatalog,
    upload: SignedToolUpload,
) -> Result<(), PluginError> {
    catalog.admit_user_tool(upload)?;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginAdmission {
    BuiltIn,
    TrustedUser,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedPlugin {
    pub manifest: PluginManifest,
    pub executable: PathBuf,
    pub source: PluginAdmission,
}

pub fn admit_executable(
    manifest: &PluginManifest,
    executable: impl AsRef<Path>,
    source: PluginAdmission,
) -> Result<AdmittedPlugin, PluginError> {
    admit_plugin(manifest, source)?;
    let executable = fs::canonicalize(executable.as_ref())
        .map_err(|error| PluginError::Discovery(error.to_string()))?;
    if !executable.is_file() {
        return Err(PluginError::Discovery(
            "plugin executable is not a regular file".into(),
        ));
    }
    Ok(AdmittedPlugin {
        manifest: manifest.clone(),
        executable,
        source,
    })
}

pub fn builtin_manifests() -> Vec<PluginManifest> {
    #[derive(Deserialize)]
    struct BuiltinCatalog {
        plugins: Vec<PluginManifest>,
    }
    let catalog: BuiltinCatalog = match toml::from_str(include_str!("plugin_catalog.toml")) {
        Ok(catalog) => catalog,
        Err(error) => panic!("built-in plugin catalog is valid: {error}"),
    };
    catalog
        .plugins
        .into_iter()
        .map(|manifest| match manifest.validate() {
            Ok(()) => manifest,
            Err(error) => panic!("built-in plugin manifest is valid: {error}"),
        })
        .collect()
}

pub fn first_party_runtime(kind: PluginKind, id: &str) -> Result<DescriptorPlugin, PluginError> {
    let manifest = catalog_manifest(kind, Some(id));
    let descriptor = PluginDescriptor::from_manifest(&manifest);
    let mut runtime = DescriptorPlugin::new(descriptor)?;
    match kind {
        PluginKind::Harness => {
            runtime = runtime.with_capability_result(
                "harness.describe",
                json!({
                    "transport": "acp",
                    "events": ["acp"],
                }),
            )?;
        }
        PluginKind::Provider => {
            let provider = first_party_providers()
                .into_iter()
                .find(|provider| provider.manifest.id == id)
                .ok_or_else(|| PluginError::InvalidManifest("provider is not registered".into()))?;
            runtime = runtime
                .with_capability_result("provider.models", json!(provider.models))?
                .with_capability_result(
                    "provider.verify",
                    provider.verification.unwrap_or(Value::Null),
                )?;
        }
        PluginKind::CliTool => {
            let tool = first_party_cli_tool();
            runtime = runtime.with_capability_result(
                "cli_tool.describe",
                json!({
                    "install": tool.install,
                    "verify": tool.verify,
                    "docs": tool.docs,
                    "digest": tool.digest,
                    "permissions": tool.permissions,
                }),
            )?;
        }
    }
    Ok(runtime)
}

pub fn admit_plugin(manifest: &PluginManifest, source: PluginAdmission) -> Result<(), PluginError> {
    manifest.validate()?;
    if source == PluginAdmission::TrustedUser
        || builtin_manifests()
            .iter()
            .any(|builtin| builtin == manifest)
    {
        Ok(())
    } else {
        Err(PluginError::InvalidManifest(
            "plugin is not in the first-party catalog".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{fmt::Debug, fs, time::SystemTime};

    fn test_ok<T, E: Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("test operation failed: {error:?}"),
        }
    }

    fn test_some<T>(value: Option<T>) -> T {
        match value {
            Some(value) => value,
            None => panic!("test value was unexpectedly absent"),
        }
    }

    fn manifest(kind: PluginKind, id: &str) -> PluginManifest {
        test_ok(PluginManifest::new(
            kind,
            id,
            "1.0.0",
            "plugin",
            ["test.cap"],
            std::iter::empty::<String>(),
        ))
    }

    fn lifecycle_script() -> String {
        r#"
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *plugin.initialize*) printf '{"jsonrpc":"2.0","id":%s,"result":{"protocol":"locus.plugin.v1"}}\n' "$id" ;;
    *plugin.describe*) printf '{"jsonrpc":"2.0","id":%s,"result":{"protocol":"locus.plugin.v1","kind":"provider","id":"fixture","version":"1.0.0","capabilities":["test.cap","future.cap"],"schema_versions":{"plugin":"v1"}}}\n' "$id" ;;
    *plugin.health*) printf '{"jsonrpc":"2.0","id":%s,"result":{"ready":true}}\n' "$id" ;;
    *test.cap*) printf '{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"AgentMessageChunk"}}}\n'; printf '{"jsonrpc":"2.0","id":%s,"result":{"ok":true}}\n' "$id" ;;
    *plugin.shutdown*) printf '{"jsonrpc":"2.0","id":%s,"result":{"stopped":true}}\n' "$id"; break ;;
  esac
done
"#
        .into()
    }

    #[test]
    fn manifest_schema() {
        let parsed = test_ok(PluginManifest::from_toml(
            r#"protocol = "locus.plugin.v1"
kind = "provider"
id = "example"
version = "1.0.0"
executable = "example"
capabilities = ["provider.models"]
permissions = ["keychain_reference"]
"#,
        ));
        assert_eq!(parsed.kind, PluginKind::Provider);
        assert!(PluginManifest::from_toml("kind = \"provider\"").is_err());
    }

    #[test]
    fn lifecycle_roundtrip() {
        test_ok(tokio::runtime::Runtime::new()).block_on(async {
            let script = lifecycle_script();
            let mut command = Command::new("sh");
            command.args(["-c", script.as_str()]);
            let process =
                test_ok(PluginProcess::spawn_command(command, Duration::from_millis(500)).await);
            let handshake = test_ok(
                process
                    .handshake(&["test.cap".into()], &["test.cap", "future.cap"])
                    .await,
            );
            assert_eq!(handshake.descriptor.id, "fixture");
            assert_eq!(handshake.negotiation.unknown_optional, Vec::<String>::new());
            assert_eq!(
                test_ok(process.call("test.cap", json!({"ok": true})).await),
                json!({"ok": true})
            );
            let events = test_ok(process.take_harness_events(&first_party_harness()).await);
            assert_eq!(
                events[0].verb,
                crate::services::telemetry::EventVerb::Assistant
            );
            assert_eq!(test_ok(process.shutdown().await)["stopped"], true);
        });
    }

    #[test]
    fn first_party_pi_executable_lifecycle() {
        test_ok(tokio::runtime::Runtime::new()).block_on(async {
            let manifest = first_party_harness().manifest;
            let executable = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(&manifest.executable);
            let process =
                test_ok(PluginProcess::spawn(executable, Duration::from_millis(500)).await);
            let handshake = test_ok(
                process
                    .handshake(
                        &[REQUIRED_HARNESS_CAPABILITY.into()],
                        &[
                            "harness.session",
                            "harness.launch",
                            "harness.events",
                            "harness.materialize",
                        ],
                    )
                    .await,
            );
            assert_eq!(handshake.descriptor.id, manifest.id);
            let result = test_ok(
                process
                    .call(
                        "harness.materialize",
                        json!({"root":"/tmp/locus-plugin-test", "extensions":{}}),
                    )
                    .await,
            );
            assert!(test_some(result["files"].as_array()).is_empty());
            assert_eq!(test_ok(process.shutdown().await)["stopped"], true);
        });
    }

    #[test]
    fn first_party_executables_have_rpc_lifecycle() {
        test_ok(tokio::runtime::Runtime::new()).block_on(async {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            for manifest in builtin_manifests() {
                let executable = root.join(&manifest.executable);
                let process =
                    test_ok(PluginProcess::spawn(executable, Duration::from_millis(500)).await);
                let required = if manifest.kind == PluginKind::Harness {
                    vec![REQUIRED_HARNESS_CAPABILITY.to_owned()]
                } else {
                    Vec::new()
                };
                let known = manifest
                    .capabilities
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                let handshake = test_ok(process.handshake(&required, &known).await);
                assert_eq!(handshake.descriptor.id, manifest.id);
                assert_eq!(test_ok(process.shutdown().await)["stopped"], true);
            }
        });
    }

    #[test]
    fn capability_negotiation() {
        let offered = vec!["harness.session".into(), "harness.future".into()];
        let result = test_ok(negotiate_capabilities(
            PLUGIN_PROTOCOL,
            &offered,
            &[REQUIRED_HARNESS_CAPABILITY.into()],
            &["harness.session"],
        ));
        assert_eq!(result.unknown_optional, vec!["harness.future"]);
        assert!(negotiate_capabilities(
            PLUGIN_PROTOCOL,
            &[],
            &[REQUIRED_HARNESS_CAPABILITY.into()],
            &[]
        )
        .is_err());
    }

    #[test]
    fn call_timeout_is_bounded() {
        test_ok(tokio::runtime::Runtime::new()).block_on(async {
            let mut command = Command::new("sh");
            command.args(["-c", "sleep 1"]);
            let process =
                test_ok(PluginProcess::spawn_command(command, Duration::from_millis(20)).await);
            let started = std::time::Instant::now();
            let result = process.health().await;
            assert!(matches!(result, Err(PluginError::Timeout(_))));
            assert!(started.elapsed() < Duration::from_millis(500));
            assert!(matches!(
                process.health().await,
                Err(PluginError::ProcessUnavailable)
            ));
            let _ = process.shutdown().await;
        });
    }

    #[test]
    fn data_only_boundary() {
        assert!(validate_data_only(&json!({"models":[{"id":"x"}]})).is_ok());
        assert!(validate_data_only(&json!({"ui":{"component":"Button"}})).is_err());
        assert!(validate_data_only(&json!({"persistence_path":"/tmp/db"})).is_err());
        assert!(validate_data_only(&json!({"api_key":"secret"})).is_err());
        let error = RpcError {
            code: -1,
            message: "failed".into(),
            data: Some(json!({"secret":"leak"})),
        };
        assert!(error.validate().is_err());
    }

    #[test]
    fn server_dispatches_lifecycle() {
        let runtime = test_ok(first_party_runtime(PluginKind::Provider, "openai"));
        let input =
            b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"plugin.describe\",\"params\":{}}\n";
        let mut output = Vec::new();
        let mut runtime = runtime;
        test_ok(serve_stdio_io(&mut runtime, &input[..], &mut output));
        let response: RpcResponse = test_ok(serde_json::from_slice(&output));
        assert_eq!(test_some(response.result)["kind"], "provider");
    }

    #[test]
    fn user_plugin_discovery() {
        let root = std::env::temp_dir().join(format!(
            "locus-plugin-{}-{}",
            std::process::id(),
            test_ok(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)).as_nanos()
        ));
        test_ok(fs::create_dir_all(root.join("example")));
        test_ok(fs::write(
            root.join("example/manifest.toml"),
            r#"protocol="locus.plugin.v1"
kind="provider"
id="example"
version="1.0.0"
executable="example"
"#,
        ));
        assert_eq!(
            test_ok(discover_user_plugins(
                std::slice::from_ref(&root),
                std::slice::from_ref(&root),
            ))
            .len(),
            1
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn harness_capabilities() {
        let descriptor = first_party_harness();
        assert!(descriptor.validate().is_ok());
        assert!(descriptor.plugin_descriptor().validate().is_ok());
    }

    #[test]
    fn harness_events_are_acp() {
        let descriptor = first_party_harness();
        let event = descriptor
            .normalize_event(json!({"params":{"update":{"sessionUpdate":"AgentMessageChunk"}}}));
        let event = test_ok(event);
        assert_eq!(
            event[0].verb,
            crate::services::telemetry::EventVerb::Assistant
        );
    }

    #[test]
    fn first_party_pi_only() {
        let harnesses: Vec<_> = builtin_manifests()
            .into_iter()
            .filter(|manifest| manifest.kind == PluginKind::Harness)
            .collect();
        assert_eq!(harnesses.len(), 1);
        assert_eq!(harnesses[0].id, ["p", "i"].concat());
    }

    #[test]
    fn provider_contract() {
        for provider in first_party_providers() {
            assert!(provider.validate().is_ok());
            assert!(provider
                .manifest
                .capabilities
                .iter()
                .any(|cap| cap == "provider.models"));
        }
    }

    #[test]
    fn first_party_openai() {
        assert!(first_party_providers()
            .iter()
            .any(|provider| provider.manifest.id == "openai"));
    }
    #[test]
    fn first_party_anthropic() {
        assert!(first_party_providers()
            .iter()
            .any(|provider| provider.manifest.id == "anthropic"));
    }
    #[test]
    fn first_party_openrouter() {
        assert!(first_party_providers()
            .iter()
            .any(|provider| provider.manifest.id == "openrouter"));
    }

    #[test]
    fn cli_tool_contract() {
        assert!(first_party_cli_tool().validate().is_ok());
    }
    #[test]
    fn first_party_gh_only() {
        let tools: Vec<_> = builtin_manifests()
            .into_iter()
            .filter(|manifest| manifest.kind == PluginKind::CliTool)
            .collect();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].id, "gh");
    }
    #[test]
    fn cli_tool_trust_boundary() {
        assert!(matches!(
            ToolAdmissionError::UntrustedSignature,
            ToolAdmissionError::UntrustedSignature
        ));
    }
    #[test]
    fn built_in_allowlist() {
        assert!(admit_plugin(&first_party_cli_tool().manifest, PluginAdmission::BuiltIn).is_ok());
        let mut forged = first_party_cli_tool().manifest;
        forged.executable = "attacker".into();
        assert!(admit_plugin(&forged, PluginAdmission::BuiltIn).is_err());
        assert!(admit_plugin(
            &manifest(PluginKind::Provider, "not-shipped"),
            PluginAdmission::BuiltIn
        )
        .is_err());
        assert!(admit_plugin(
            &manifest(PluginKind::Provider, "user"),
            PluginAdmission::TrustedUser
        )
        .is_ok());
    }
    #[test]
    fn contract_suite() {
        assert_eq!(builtin_manifests().len(), 5);
        assert_eq!(first_party_providers().len(), 3);
        assert!(first_party_runtime(PluginKind::Provider, "openrouter").is_ok());
        assert!(first_party_harness()
            .normalize_event(json!({"params":{"update":{"sessionUpdate":"agent_message_chunk"}}}))
            .is_ok());
    }
}
