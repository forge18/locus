//! Language-server protocol owned by Locus.
//!
//! The client is deliberately independent of a language name. Descriptors are data, while this
//! module owns the protocol framing, project supervision, clone boundary, and deterministic
//! semantic-token decoding used by both the host and the in-container CLI.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{self, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{mpsc, Mutex},
    time::Duration,
};
use thiserror::Error;
use tokio::sync::broadcast;
use url::Url;

use crate::bus::InProcessBus;

const JSON_RPC_VERSION: &str = "2.0";
const MAX_HEADER_BYTES: usize = 8 * 1024;
const MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
const MAX_SKIPPED_NOTIFICATIONS: usize = 1_024;
const LSP_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Error)]
pub enum LspError {
    #[error("invalid LSP request: {0}")]
    Invalid(String),
    #[error("LSP server is unsupported: {0}")]
    Unsupported(String),
    #[error("LSP server exited unexpectedly")]
    ServerExited,
    #[error("LSP request timed out")]
    Timeout,
    #[error("LSP protocol error: {0}")]
    Protocol(String),
    #[error("LSP I/O failed: {0}")]
    Io(#[from] io::Error),
    #[error("LSP JSON failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("LSP descriptor TOML failed: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("LSP descriptor TOML serialization failed: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Value,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: impl Into<String>, params: Value) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.into(),
            id,
            method: method.into(),
            params,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LspNotification {
    pub method: String,
    pub params: Value,
}

impl JsonRpcNotification {
    pub fn new(method: impl Into<String>, params: Value) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.into(),
            method: method.into(),
            params,
        }
    }
}

fn encode_frame(value: &Value) -> Result<Vec<u8>, LspError> {
    let body = serde_json::to_vec(value)?;
    if body.len() > MAX_MESSAGE_BYTES {
        return Err(LspError::Protocol("LSP message exceeds size limit".into()));
    }
    let mut frame = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    frame.extend(body);
    Ok(frame)
}

fn write_frame(writer: &mut impl Write, value: &Value) -> Result<(), LspError> {
    writer.write_all(&encode_frame(value)?)?;
    writer.flush()?;
    Ok(())
}

fn read_frame(reader: &mut impl Read) -> Result<Value, LspError> {
    let mut headers = Vec::new();
    let mut window = Vec::new();
    loop {
        let mut byte = [0; 1];
        reader.read_exact(&mut byte)?;
        headers.push(byte[0]);
        window.push(byte[0]);
        if window.len() > 4 {
            window.remove(0);
        }
        if headers.len() > MAX_HEADER_BYTES {
            return Err(LspError::Protocol("LSP headers exceed size limit".into()));
        }
        if window == b"\r\n\r\n" {
            break;
        }
    }

    let header_text = std::str::from_utf8(&headers)
        .map_err(|error| LspError::Protocol(format!("LSP headers are not UTF-8: {error}")))?;
    let length = header_text
        .lines()
        .find_map(|line| {
            line.strip_prefix("Content-Length:")
                .or_else(|| line.strip_prefix("content-length:"))
                .and_then(|value| value.trim().parse::<usize>().ok())
        })
        .ok_or_else(|| LspError::Protocol("LSP response omitted Content-Length".into()))?;
    if length > MAX_MESSAGE_BYTES {
        return Err(LspError::Protocol("LSP response exceeds size limit".into()));
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    Ok(serde_json::from_slice(&body)?)
}

/// A synchronous JSON-RPC client for one LSP process.
///
/// LSP servers are request/response streams, not line-delimited JSON. The client consumes
/// notifications while waiting for the matching response and bounds both message size and
/// notification starvation.
pub struct StdioLspServer {
    child: Child,
    stdin: ChildStdin,
    incoming: mpsc::Receiver<Result<Value, LspError>>,
    next_id: u64,
    capabilities: Value,
    initialize_result: Option<Value>,
    notifications: Vec<LspNotification>,
}

impl StdioLspServer {
    pub fn start(command: &[String], root: &Path) -> Result<Self, LspError> {
        let executable = command
            .first()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| LspError::Invalid("LSP descriptor command is empty".into()))?;
        let mut child = Command::new(executable)
            .args(&command[1..])
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| LspError::Protocol("LSP stdin was not piped".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| LspError::Protocol("LSP stdout was not piped".into()))?;
        let (sender, incoming) = mpsc::channel();
        std::thread::spawn(move || {
            let mut stdout = BufReader::new(stdout);
            loop {
                let message = read_frame(&mut stdout);
                let finished = message.is_err();
                if sender.send(message).is_err() || finished {
                    break;
                }
            }
        });
        let mut server = Self {
            child,
            stdin,
            incoming,
            next_id: 1,
            capabilities: Value::Null,
            initialize_result: None,
            notifications: Vec::new(),
        };
        let initialize = server.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "rootUri": file_uri(root)?,
                "capabilities": {
                    "workspace": {"workspaceFolders": true},
                    "textDocument": {
                        "definition": {},
                        "references": {},
                        "hover": {},
                        "rename": {},
                        "publishDiagnostics": {},
                        "semanticTokens": {"requests": {"full": {"delta": true}}}
                    }
                },
                "workspaceFolders": [{"uri": file_uri(root)?, "name": root.file_name().and_then(|name| name.to_str()).unwrap_or("workspace")}]
            }),
        )?;
        server.capabilities = initialize
            .get("capabilities")
            .cloned()
            .unwrap_or(Value::Null);
        server.initialize_result = Some(initialize);
        server.notify("initialized", json!({}))?;
        Ok(server)
    }

    pub fn capabilities(&self) -> &Value {
        &self.capabilities
    }

    pub fn supports(&self, capability: &str) -> bool {
        capability
            .split('.')
            .try_fold(&self.capabilities, |value, key| value.get(key))
            .is_some_and(|value| !value.is_null() && value != &json!(false))
    }

    fn supports_method(&self, method: &str) -> bool {
        if method == "initialize" {
            return true;
        }
        let capability = match method {
            "textDocument/definition" => "definitionProvider",
            "textDocument/declaration" => "declarationProvider",
            "textDocument/typeDefinition" => "typeDefinitionProvider",
            "textDocument/implementation" => "implementationProvider",
            "textDocument/references" => "referencesProvider",
            "textDocument/hover" => "hoverProvider",
            "textDocument/completion" => "completionProvider",
            "textDocument/signatureHelp" => "signatureHelpProvider",
            "textDocument/documentSymbol" => "documentSymbolProvider",
            "textDocument/documentHighlight" => "documentHighlightProvider",
            "textDocument/diagnostic" => "diagnosticProvider",
            "textDocument/rename" | "textDocument/prepareRename" => "renameProvider",
            "textDocument/formatting" => "documentFormattingProvider",
            "textDocument/rangeFormatting" => "documentRangeFormattingProvider",
            "textDocument/foldingRange" => "foldingRangeProvider",
            "textDocument/codeAction" => "codeActionProvider",
            "textDocument/documentLink" => "documentLinkProvider",
            "workspace/symbol" => "workspaceSymbolProvider",
            "textDocument/semanticTokens/full" | "textDocument/semanticTokens/full/delta" => {
                "semanticTokensProvider"
            }
            _ => return false,
        };
        self.supports(capability)
    }

    pub fn request(&mut self, method: &str, params: Value) -> Result<Value, LspError> {
        if method == "initialize" {
            if let Some(result) = &self.initialize_result {
                return Ok(result.clone());
            }
        }
        let id = self.next_id;
        self.next_id = self.next_id.checked_add(1).unwrap_or(1);
        let request = serde_json::to_value(JsonRpcRequest::new(id, method, params))?;
        write_frame(&mut self.stdin, &request)?;
        let mut skipped = 0;
        loop {
            let response = self
                .incoming
                .recv_timeout(LSP_REQUEST_TIMEOUT)
                .map_err(|error| match error {
                    mpsc::RecvTimeoutError::Timeout => LspError::Timeout,
                    mpsc::RecvTimeoutError::Disconnected => LspError::ServerExited,
                })??;
            if response.get("id") == Some(&json!(id)) {
                if let Some(error) = response.get("error") {
                    return Err(LspError::Protocol(error.to_string()));
                }
                return Ok(response.get("result").cloned().unwrap_or(Value::Null));
            }
            self.handle_interleaved(&response)?;
            skipped += 1;
            if skipped > MAX_SKIPPED_NOTIFICATIONS {
                return Err(LspError::Protocol(
                    "LSP server sent too many notifications while responding".into(),
                ));
            }
        }
    }

    fn handle_interleaved(&mut self, response: &Value) -> Result<(), LspError> {
        if let Some(method) = response.get("method").and_then(Value::as_str) {
            if let Some(server_request_id) = response.get("id") {
                write_frame(
                    &mut self.stdin,
                    &json!({"jsonrpc": JSON_RPC_VERSION, "id": server_request_id, "result": Value::Null}),
                )?;
            } else {
                self.notifications.push(LspNotification {
                    method: method.into(),
                    params: response.get("params").cloned().unwrap_or(Value::Null),
                });
            }
        }
        Ok(())
    }

    pub fn take_notifications(&mut self) -> Vec<LspNotification> {
        while let Ok(message) = self.incoming.try_recv() {
            let Ok(response) = message else { break };
            if self.handle_interleaved(&response).is_err() {
                break;
            }
        }
        std::mem::take(&mut self.notifications)
    }

    pub fn notify(&mut self, method: &str, params: Value) -> Result<(), LspError> {
        if method == "initialized" && self.initialize_result.is_some() {
            return Ok(());
        }
        write_frame(
            &mut self.stdin,
            &serde_json::to_value(JsonRpcNotification::new(method, params))?,
        )
    }

    pub fn shutdown(&mut self) -> Result<(), LspError> {
        if self.is_alive()? {
            let id = self.next_id;
            self.next_id = self.next_id.checked_add(1).unwrap_or(1);
            let _ = write_frame(
                &mut self.stdin,
                &serde_json::to_value(JsonRpcRequest::new(id, "shutdown", Value::Null))?,
            );
            let _ = self.notify("exit", Value::Null);
            // Do not wait for a potentially hung server response. The supervisor owns the
            // process and shutdown is bounded by the child kill/wait path.
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        Ok(())
    }

    pub fn is_alive(&mut self) -> Result<bool, LspError> {
        Ok(self.child.try_wait()?.is_none())
    }
}

impl Drop for StdioLspServer {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

pub trait LspServer: Send {
    fn request(&mut self, method: &str, params: Value) -> Result<Value, LspError>;
    fn supports_method(&self, _method: &str) -> bool {
        true
    }
    fn notify(&mut self, method: &str, params: Value) -> Result<(), LspError>;
    fn take_notifications(&mut self) -> Vec<LspNotification>;
    fn is_alive(&mut self) -> Result<bool, LspError>;
    fn shutdown(&mut self) -> Result<(), LspError>;
}

impl LspServer for StdioLspServer {
    fn request(&mut self, method: &str, params: Value) -> Result<Value, LspError> {
        Self::request(self, method, params)
    }

    fn supports_method(&self, method: &str) -> bool {
        Self::supports_method(self, method)
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<(), LspError> {
        Self::notify(self, method, params)
    }

    fn take_notifications(&mut self) -> Vec<LspNotification> {
        Self::take_notifications(self)
    }

    fn is_alive(&mut self) -> Result<bool, LspError> {
        Self::is_alive(self)
    }

    fn shutdown(&mut self) -> Result<(), LspError> {
        Self::shutdown(self)
    }
}

pub trait LspServerFactory: Send {
    fn start(
        &mut self,
        descriptor: &LanguageDescriptor,
        root: &Path,
    ) -> Result<Box<dyn LspServer>, LspError>;
}

#[derive(Default)]
pub struct StdioLspFactory;

impl LspServerFactory for StdioLspFactory {
    fn start(
        &mut self,
        descriptor: &LanguageDescriptor,
        root: &Path,
    ) -> Result<Box<dyn LspServer>, LspError> {
        Ok(Box::new(StdioLspServer::start(&descriptor.command, root)?))
    }
}

struct ProjectServer {
    root: PathBuf,
    descriptor: LanguageDescriptor,
    server: Box<dyn LspServer>,
    panes: BTreeSet<String>,
    open_documents: BTreeMap<String, Value>,
    restarts: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LspPane {
    pub project_root: PathBuf,
    pub pane_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectServerInfo {
    pub project_root: PathBuf,
    pub descriptor_id: String,
    pub pane_count: usize,
    pub restart_count: u32,
}

fn document_uri(params: &Value) -> Option<String> {
    params
        .get("textDocument")
        .and_then(Value::as_object)
        .and_then(|document| document.get("uri"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn remember_document(project: &mut ProjectServer, method: &str, params: &Value) {
    let Some(uri) = document_uri(params) else {
        return;
    };
    match method {
        "textDocument/didOpen" => {
            project.open_documents.insert(uri, params.clone());
        }
        "textDocument/didChange" => {
            let Some(document) = project.open_documents.get_mut(&uri) else {
                return;
            };
            if let Some(version) = params
                .get("textDocument")
                .and_then(|text_document| text_document.get("version"))
                .cloned()
            {
                document["textDocument"]["version"] = version;
            }
            if let Some(text) = params
                .get("contentChanges")
                .and_then(Value::as_array)
                .and_then(|changes| changes.first())
                .and_then(|change| change.get("text"))
                .and_then(Value::as_str)
            {
                document["text"] = Value::String(text.to_owned());
            }
        }
        "textDocument/didClose" => {
            project.open_documents.remove(&uri);
        }
        _ => {}
    }
}

fn replay_documents(
    server: &mut dyn LspServer,
    documents: &BTreeMap<String, Value>,
) -> Result<(), LspError> {
    for params in documents.values() {
        server.notify("textDocument/didOpen", params.clone())?;
    }
    Ok(())
}

/// One server set per project, shared by every editor pane in that project.
pub struct HostLspSupervisor<F: LspServerFactory> {
    factory: F,
    projects: BTreeMap<PathBuf, ProjectServer>,
}

impl<F: LspServerFactory> HostLspSupervisor<F> {
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            projects: BTreeMap::new(),
        }
    }

    pub fn ensure_project(
        &mut self,
        root: impl Into<PathBuf>,
        descriptor: LanguageDescriptor,
    ) -> Result<(), LspError> {
        let root = root.into();
        descriptor.validate()?;
        if let Some(project) = self.projects.get(&root) {
            if project.descriptor.id == descriptor.id
                && project.descriptor.content_hash == descriptor.content_hash
            {
                return Ok(());
            }
            if !project.panes.is_empty() {
                return Err(LspError::Unsupported(format!(
                    "project already has panes attached to `{}`",
                    project.descriptor.id
                )));
            }
        }
        let mut server = self.factory.start(&descriptor, &root)?;
        let (panes, open_documents, restarts) = if let Some(mut old) = self.projects.remove(&root) {
            let panes = old.panes;
            let open_documents = old.open_documents;
            let restarts = old.restarts;
            let _ = old.server.shutdown();
            (panes, open_documents, restarts)
        } else {
            (BTreeSet::new(), BTreeMap::new(), 0)
        };
        if let Err(error) = replay_documents(server.as_mut(), &open_documents) {
            let _ = server.shutdown();
            return Err(error);
        }
        self.projects.insert(
            root.clone(),
            ProjectServer {
                root,
                descriptor,
                server,
                panes,
                open_documents,
                restarts,
            },
        );
        Ok(())
    }

    pub fn attach_pane(
        &mut self,
        root: impl Into<PathBuf>,
        pane_id: impl Into<String>,
        descriptor: LanguageDescriptor,
    ) -> Result<LspPane, LspError> {
        let root = root.into();
        self.ensure_project(root.clone(), descriptor)?;
        let pane_id = pane_id.into();
        if pane_id.trim().is_empty() {
            return Err(LspError::Invalid("pane id is required".into()));
        }
        let project = self
            .projects
            .get_mut(&root)
            .ok_or_else(|| LspError::Invalid("LSP project disappeared during attach".into()))?;
        project.panes.insert(pane_id.clone());
        Ok(LspPane {
            project_root: root,
            pane_id,
        })
    }

    pub fn detach_pane(&mut self, pane: &LspPane) {
        if let Some(project) = self.projects.get_mut(&pane.project_root) {
            project.panes.remove(&pane.pane_id);
        }
    }

    pub fn request(
        &mut self,
        root: impl Into<PathBuf>,
        method: &str,
        params: Value,
    ) -> Result<Value, LspError> {
        let root = root.into();
        let dead = self
            .projects
            .get_mut(&root)
            .ok_or_else(|| LspError::Invalid("project has no LSP server".into()))?
            .server
            .is_alive()
            .is_ok_and(|alive| !alive);
        if dead {
            self.restart(&root)?;
        }
        let supports_method = self
            .projects
            .get(&root)
            .is_some_and(|project| project.server.supports_method(method));
        if !supports_method {
            return Err(LspError::Unsupported(format!(
                "LSP server does not advertise `{method}`"
            )));
        }
        let result = self
            .projects
            .get_mut(&root)
            .ok_or_else(|| LspError::Invalid("project has no LSP server".into()))?
            .server
            .request(method, params.clone());
        match result {
            Ok(value) => Ok(value),
            Err(error) => {
                let alive = self
                    .projects
                    .get_mut(&root)
                    .and_then(|project| project.server.is_alive().ok())
                    .unwrap_or(false);
                if alive && !matches!(error, LspError::ServerExited) {
                    return Err(error);
                }
                self.restart(&root)?;
                self.projects
                    .get_mut(&root)
                    .ok_or_else(|| LspError::Invalid("project has no LSP server".into()))?
                    .server
                    .request(method, params)
            }
        }
    }

    pub fn notify(
        &mut self,
        root: impl Into<PathBuf>,
        method: &str,
        params: Value,
    ) -> Result<(), LspError> {
        let root = root.into();
        let result = {
            let project = self
                .projects
                .get_mut(&root)
                .ok_or_else(|| LspError::Invalid("project has no LSP server".into()))?;
            remember_document(project, method, &params);
            project.server.notify(method, params)
        };
        match result {
            Ok(()) => Ok(()),
            Err(error) => {
                let dead = self
                    .projects
                    .get_mut(&root)
                    .and_then(|project| project.server.is_alive().ok())
                    .is_none_or(|alive| !alive);
                if dead {
                    self.restart(&root)
                } else {
                    Err(error)
                }
            }
        }
    }

    pub fn take_notifications(&mut self, root: &Path) -> Result<Vec<LspNotification>, LspError> {
        self.projects
            .get_mut(root)
            .ok_or_else(|| LspError::Invalid("project has no LSP server".into()))
            .map(|project| project.server.take_notifications())
    }

    pub fn remove_project(&mut self, root: &Path) -> Result<(), LspError> {
        if let Some(mut project) = self.projects.remove(root) {
            project.server.shutdown()?;
        }
        Ok(())
    }

    fn restart(&mut self, root: &Path) -> Result<(), LspError> {
        let (project_root, descriptor, panes, open_documents, restart_count) = {
            let project = self
                .projects
                .get(root)
                .ok_or_else(|| LspError::Invalid("project has no LSP server".into()))?;
            (
                project.root.clone(),
                project.descriptor.clone(),
                project.panes.clone(),
                project.open_documents.clone(),
                project.restarts.saturating_add(1),
            )
        };
        // Start first. If provisioning or initialization fails, the old project remains in the
        // map and its pane ownership is not silently lost.
        let mut server = self.factory.start(&descriptor, &project_root)?;
        if let Err(error) = replay_documents(server.as_mut(), &open_documents) {
            let _ = server.shutdown();
            return Err(error);
        }
        if let Some(mut old) = self.projects.remove(root) {
            let _ = old.server.shutdown();
        }
        self.projects.insert(
            root.to_path_buf(),
            ProjectServer {
                root: project_root,
                descriptor,
                server,
                panes,
                open_documents,
                restarts: restart_count,
            },
        );
        Ok(())
    }

    pub fn project_roots(&self) -> Vec<PathBuf> {
        self.projects.keys().cloned().collect()
    }

    pub fn project_info(&self, root: &Path) -> Option<ProjectServerInfo> {
        self.projects.get(root).map(|project| ProjectServerInfo {
            project_root: project.root.clone(),
            descriptor_id: project.descriptor.id.clone(),
            pane_count: project.panes.len(),
            restart_count: project.restarts,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspDiagnostic {
    pub project_root: PathBuf,
    pub uri: String,
    pub diagnostics: Value,
    pub version: Option<i64>,
}

/// Host-side composition root for one catalog and one supervised server set per project.
pub struct LspHost {
    catalog: LanguageCatalog,
    supervisor: Mutex<HostLspSupervisor<StdioLspFactory>>,
    project_pins: Mutex<BTreeMap<PathBuf, BTreeMap<String, DescriptorPin>>>,
    diagnostics: InProcessBus<LspDiagnostic>,
}

impl LspHost {
    pub fn new(catalog: LanguageCatalog) -> Self {
        Self {
            catalog,
            supervisor: Mutex::new(HostLspSupervisor::new(StdioLspFactory)),
            project_pins: Mutex::new(BTreeMap::new()),
            diagnostics: InProcessBus::new(1_024),
        }
    }

    pub fn catalog(&self) -> &LanguageCatalog {
        &self.catalog
    }

    pub fn descriptor_for_path(&self, path: &Path) -> Result<LanguageDescriptor, LspError> {
        self.catalog.execution_descriptor_for_path(path)
    }

    pub fn enable_project_descriptor(
        &self,
        project_root: impl AsRef<Path>,
        pin: DescriptorPin,
    ) -> Result<LanguageDescriptor, LspError> {
        let project_root = fs::canonicalize(project_root.as_ref())?;
        let descriptor = self.catalog.descriptor_for_pin(&pin)?;
        self.project_pins
            .lock()
            .map_err(|_| LspError::Invalid("LSP project pin lock is poisoned".into()))?
            .entry(project_root)
            .or_default()
            .insert(pin.id.clone(), pin);
        Ok(descriptor)
    }

    pub fn disable_project_descriptor(
        &self,
        project_root: impl AsRef<Path>,
        descriptor_id: &str,
    ) -> Result<(), LspError> {
        let project_root = fs::canonicalize(project_root.as_ref())?;
        let mut pins = self
            .project_pins
            .lock()
            .map_err(|_| LspError::Invalid("LSP project pin lock is poisoned".into()))?;
        if let Some(project) = pins.get_mut(&project_root) {
            project.remove(descriptor_id);
            if project.is_empty() {
                pins.remove(&project_root);
            }
        }
        Ok(())
    }

    pub fn descriptor_for_project_path(
        &self,
        project_root: &Path,
        path: &Path,
    ) -> Result<LanguageDescriptor, LspError> {
        let project_root = fs::canonicalize(project_root)?;
        let extension = path.extension().and_then(|extension| extension.to_str());
        if let Some(extension) = extension {
            let pins = self
                .project_pins
                .lock()
                .map_err(|_| LspError::Invalid("LSP project pin lock is poisoned".into()))?;
            if let Some(project) = pins.get(&project_root) {
                for pin in project.values() {
                    let descriptor = self.catalog.descriptor_for_pin(pin)?;
                    if descriptor
                        .extensions
                        .iter()
                        .any(|candidate| candidate == &format!(".{extension}"))
                    {
                        return Ok(descriptor);
                    }
                }
                return Err(LspError::Unsupported(format!(
                    "project has no pinned LSP descriptor for `.{extension}`"
                )));
            }
        }
        self.descriptor_for_path(path)
    }

    pub fn attach(
        &self,
        project_root: impl Into<PathBuf>,
        pane_id: impl Into<String>,
        file_path: impl AsRef<Path>,
    ) -> Result<LspPane, LspError> {
        let project_root = fs::canonicalize(project_root.into())?;
        let file_path = file_path.as_ref();
        let absolute = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            project_root.join(file_path)
        };
        if !fs::canonicalize(&absolute)?.starts_with(&project_root) {
            return Err(LspError::Invalid(
                "editor file escapes the project root".into(),
            ));
        }
        let descriptor = self.descriptor_for_project_path(&project_root, &absolute)?;
        self.supervisor
            .lock()
            .map_err(|_| LspError::Invalid("LSP supervisor lock is poisoned".into()))?
            .attach_pane(project_root, pane_id, descriptor)
    }

    pub fn detach(
        &self,
        project_root: impl Into<PathBuf>,
        pane_id: impl Into<String>,
    ) -> Result<(), LspError> {
        let project_root = project_root.into();
        let project_root = fs::canonicalize(&project_root).unwrap_or(project_root);
        let pane = LspPane {
            project_root: project_root.clone(),
            pane_id: pane_id.into(),
        };
        let mut supervisor = self
            .supervisor
            .lock()
            .map_err(|_| LspError::Invalid("LSP supervisor lock is poisoned".into()))?;
        supervisor.detach_pane(&pane);
        let empty = supervisor
            .project_info(&project_root)
            .is_some_and(|info| info.pane_count == 0);
        if empty {
            supervisor.remove_project(&project_root)?;
        }
        Ok(())
    }

    pub fn request(
        &self,
        project_root: impl Into<PathBuf>,
        method: &str,
        params: Value,
    ) -> Result<Value, LspError> {
        if !matches!(
            method,
            "initialize"
                | "textDocument/definition"
                | "textDocument/declaration"
                | "textDocument/typeDefinition"
                | "textDocument/implementation"
                | "textDocument/references"
                | "textDocument/hover"
                | "textDocument/completion"
                | "textDocument/signatureHelp"
                | "textDocument/documentSymbol"
                | "textDocument/documentHighlight"
                | "textDocument/diagnostic"
                | "textDocument/rename"
                | "textDocument/prepareRename"
                | "textDocument/formatting"
                | "textDocument/rangeFormatting"
                | "textDocument/foldingRange"
                | "textDocument/codeAction"
                | "textDocument/documentLink"
                | "workspace/symbol"
                | "textDocument/semanticTokens/full"
                | "textDocument/semanticTokens/full/delta"
        ) {
            return Err(LspError::Unsupported(format!(
                "unsupported LSP method `{method}`"
            )));
        }
        let project_root = fs::canonicalize(project_root.into())?;
        validate_lsp_params(&project_root, method, &params)?;
        let mut supervisor = self
            .supervisor
            .lock()
            .map_err(|_| LspError::Invalid("LSP supervisor lock is poisoned".into()))?;
        let result = supervisor.request(project_root.clone(), method, params)?;
        let notifications = supervisor.take_notifications(&project_root)?;
        drop(supervisor);
        self.publish_notifications(&project_root, notifications);
        Ok(result)
    }

    pub fn notify(
        &self,
        project_root: impl Into<PathBuf>,
        method: &str,
        params: Value,
    ) -> Result<(), LspError> {
        if !matches!(
            method,
            "initialized"
                | "textDocument/didOpen"
                | "textDocument/didChange"
                | "textDocument/didClose"
                | "workspace/didChangeConfiguration"
                | "exit"
                | "$/cancelRequest"
        ) {
            return Err(LspError::Unsupported(format!(
                "unsupported LSP notification `{method}`"
            )));
        }
        let project_root = fs::canonicalize(project_root.into())?;
        validate_lsp_params(&project_root, method, &params)?;
        let mut supervisor = self
            .supervisor
            .lock()
            .map_err(|_| LspError::Invalid("LSP supervisor lock is poisoned".into()))?;
        supervisor.notify(project_root.clone(), method, params)?;
        let notifications = supervisor.take_notifications(&project_root)?;
        drop(supervisor);
        self.publish_notifications(&project_root, notifications);
        Ok(())
    }

    pub fn poll_notifications(&self) -> Result<(), LspError> {
        let mut supervisor = self
            .supervisor
            .lock()
            .map_err(|_| LspError::Invalid("LSP supervisor lock is poisoned".into()))?;
        for project_root in supervisor.project_roots() {
            let notifications = supervisor.take_notifications(&project_root)?;
            self.publish_notifications(&project_root, notifications);
        }
        Ok(())
    }

    fn publish_notifications(&self, project_root: &Path, notifications: Vec<LspNotification>) {
        for notification in notifications {
            if notification.method != "textDocument/publishDiagnostics" {
                continue;
            }
            let Some(object) = notification.params.as_object() else {
                continue;
            };
            let Some(uri) = object.get("uri").and_then(Value::as_str) else {
                continue;
            };
            self.diagnostics.publish(LspDiagnostic {
                project_root: project_root.to_path_buf(),
                uri: uri.into(),
                diagnostics: object
                    .get("diagnostics")
                    .cloned()
                    .unwrap_or(Value::Array(vec![])),
                version: object.get("version").and_then(Value::as_i64),
            });
        }
    }

    pub fn subscribe_diagnostics(&self) -> broadcast::Receiver<LspDiagnostic> {
        self.diagnostics.subscribe()
    }

    pub fn project_info(&self, project_root: &Path) -> Result<Option<ProjectServerInfo>, LspError> {
        Ok(self
            .supervisor
            .lock()
            .map_err(|_| LspError::Invalid("LSP supervisor lock is poisoned".into()))?
            .project_info(project_root))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CatalogFile {
    descriptors: Vec<LanguageDescriptor>,
    #[serde(default)]
    executable_hashes: BTreeMap<String, String>,
}

/// A language descriptor is data: the protocol does not branch on its id.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LanguageDescriptor {
    pub id: String,
    pub version: u32,
    pub extensions: Vec<String>,
    pub root_markers: Vec<String>,
    pub command: Vec<String>,
    #[serde(default)]
    pub capabilities: BTreeSet<String>,
    #[serde(default)]
    pub grammar: Option<String>,
    #[serde(default)]
    pub content_hash: String,
}

impl LanguageDescriptor {
    pub fn validate(&self) -> Result<(), LspError> {
        if self.id.trim().is_empty()
            || self.id == "."
            || self.id == ".."
            || self.id.contains('/')
            || self.id.contains('\\')
        {
            return Err(LspError::Invalid("descriptor id is invalid".into()));
        }
        if self.version == 0 {
            return Err(LspError::Invalid(
                "descriptor version must be positive".into(),
            ));
        }
        if self.extensions.is_empty()
            || self.extensions.iter().any(|extension| {
                extension.trim().is_empty()
                    || !extension.starts_with('.')
                    || extension.contains('/')
            })
        {
            return Err(LspError::Invalid(
                "descriptor extensions must be non-empty dot-prefixed values".into(),
            ));
        }
        if self.command.is_empty()
            || self
                .command
                .iter()
                .any(|part| part.trim().is_empty() || part.contains('\0'))
        {
            return Err(LspError::Invalid("descriptor command is invalid".into()));
        }
        Ok(())
    }

    fn canonical_bytes(&self) -> Result<Vec<u8>, LspError> {
        let mut clone = self.clone();
        clone.content_hash.clear();
        Ok(serde_json::to_vec(&clone)?)
    }

    pub fn with_hash(mut self) -> Result<Self, LspError> {
        self.validate()?;
        let digest = Sha256::digest(self.canonical_bytes()?);
        self.content_hash = format!("sha256:{digest:x}");
        Ok(self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorPin {
    pub id: String,
    pub version: u32,
    pub content_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DescriptorSuggestion {
    pub descriptor: DescriptorPin,
    pub matched_root_markers: Vec<String>,
    pub matched_extensions: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionedDescriptor {
    pub descriptor: DescriptorPin,
    pub host_cache: PathBuf,
    pub agent_image_layer: PathBuf,
}

#[derive(Clone, Debug, Default)]
pub struct LanguageCatalog {
    descriptors: BTreeMap<String, LanguageDescriptor>,
    enabled: BTreeSet<String>,
    trusted: BTreeSet<String>,
    binary_hashes: BTreeMap<String, String>,
}

impl LanguageCatalog {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn builtin() -> Result<Self, LspError> {
        let mut catalog = Self::from_toml(include_str!("lsp_catalog.toml"))?;
        for id in catalog.descriptors.keys().cloned().collect::<Vec<_>>() {
            catalog.enabled.insert(id.clone());
            catalog.trusted.insert(id);
        }
        Ok(catalog)
    }

    pub fn from_toml(source: &str) -> Result<Self, LspError> {
        let file: CatalogFile = toml::from_str(source)?;
        let CatalogFile {
            descriptors,
            executable_hashes,
        } = file;
        let mut catalog = Self {
            binary_hashes: executable_hashes,
            ..Self::default()
        };
        for descriptor in descriptors {
            let descriptor = descriptor.with_hash()?;
            if catalog
                .descriptors
                .insert(descriptor.id.clone(), descriptor)
                .is_some()
            {
                return Err(LspError::Invalid("duplicate language descriptor id".into()));
            }
        }
        Ok(catalog)
    }

    pub fn load_user_catalog(root: impl AsRef<Path>) -> Result<Self, LspError> {
        let mut catalog = Self::default();
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            if entry.file_type()?.is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "toml")
            {
                let loaded = Self::from_toml(&fs::read_to_string(entry.path())?)?;
                for (id, descriptor) in loaded.descriptors {
                    if catalog.descriptors.insert(id.clone(), descriptor).is_some() {
                        return Err(LspError::Invalid(
                            "duplicate user language descriptor id".into(),
                        ));
                    }
                    if let Some(hash) = loaded.binary_hashes.get(&id) {
                        catalog.binary_hashes.insert(id, hash.clone());
                    }
                }
            }
        }
        Ok(catalog)
    }

    /// Add an imported catalog without enabling or trusting any of its descriptors.
    pub fn merge_user_catalog(&mut self, user: LanguageCatalog) -> Result<(), LspError> {
        for (id, descriptor) in user.descriptors {
            if self.descriptors.contains_key(&id) {
                return Err(LspError::Invalid(format!(
                    "language descriptor `{id}` conflicts with the active catalog"
                )));
            }
            self.descriptors.insert(id.clone(), descriptor);
            if let Some(hash) = user.binary_hashes.get(&id) {
                self.binary_hashes.insert(id, hash.clone());
            }
        }
        Ok(())
    }

    pub fn insert(&mut self, descriptor: LanguageDescriptor) -> Result<(), LspError> {
        let descriptor = descriptor.with_hash()?;
        if self.descriptors.contains_key(&descriptor.id) {
            return Err(LspError::Invalid(format!(
                "duplicate language descriptor `{}`",
                descriptor.id
            )));
        }
        self.descriptors.insert(descriptor.id.clone(), descriptor);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&LanguageDescriptor> {
        self.descriptors.get(id)
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &LanguageDescriptor> {
        self.descriptors.values()
    }

    pub fn for_path(&self, path: &Path) -> Option<&LanguageDescriptor> {
        let extension = path
            .extension()?
            .to_str()
            .map(|value| format!(".{value}"))?;
        self.descriptors.values().find(|descriptor| {
            self.enabled.contains(&descriptor.id)
                && descriptor
                    .extensions
                    .iter()
                    .any(|value| value == &extension)
        })
    }

    pub fn execution_descriptor_for_path(
        &self,
        path: &Path,
    ) -> Result<LanguageDescriptor, LspError> {
        let descriptor = self.for_path(path).ok_or_else(|| {
            LspError::Unsupported(format!("no descriptor matches `{}`", path.display()))
        })?;
        self.resolve_executable(descriptor)
    }

    fn resolve_executable(
        &self,
        descriptor: &LanguageDescriptor,
    ) -> Result<LanguageDescriptor, LspError> {
        if self.trusted.contains(&descriptor.id) {
            return Ok(descriptor.clone());
        }
        let expected = self.binary_hashes.get(&descriptor.id).ok_or_else(|| {
            LspError::Unsupported(format!(
                "descriptor `{}` has no executable pin",
                descriptor.id
            ))
        })?;
        let executable = descriptor
            .command
            .first()
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .ok_or_else(|| {
                LspError::Unsupported(format!(
                    "descriptor `{}` must use an absolute pinned executable",
                    descriptor.id
                ))
            })?;
        let executable = fs::canonicalize(executable)?;
        if !fs::symlink_metadata(&executable)?.file_type().is_file() {
            return Err(LspError::Unsupported(
                "pinned LSP executable is not a regular file".into(),
            ));
        }
        if sha256_file(&executable)? != *expected {
            return Err(LspError::Unsupported(
                "pinned LSP executable hash changed".into(),
            ));
        }
        let mut resolved = descriptor.clone();
        resolved.command[0] = executable.display().to_string();
        Ok(resolved)
    }

    pub fn pin(&self, id: &str) -> Result<DescriptorPin, LspError> {
        let descriptor = self
            .get(id)
            .ok_or_else(|| LspError::Invalid(format!("unknown language descriptor `{id}`")))?;
        Ok(DescriptorPin {
            id: descriptor.id.clone(),
            version: descriptor.version,
            content_hash: descriptor.content_hash.clone(),
        })
    }

    /// Resolve a project pin without changing global catalog enablement. This is the only path
    /// project activation uses, so editing a catalog cannot silently replace an active server.
    pub fn descriptor_for_pin(&self, pin: &DescriptorPin) -> Result<LanguageDescriptor, LspError> {
        let descriptor = self.get(&pin.id).ok_or_else(|| {
            LspError::Invalid(format!("unknown language descriptor `{}`", pin.id))
        })?;
        if descriptor.version != pin.version || descriptor.content_hash != pin.content_hash {
            return Err(LspError::Invalid(format!(
                "descriptor pin for `{}` no longer matches the catalog",
                pin.id
            )));
        }
        self.resolve_executable(descriptor)
    }

    pub fn enable(&mut self, pin: &DescriptorPin) -> Result<LanguageDescriptor, LspError> {
        let descriptor = self
            .get(&pin.id)
            .ok_or_else(|| LspError::Invalid(format!("unknown language descriptor `{}`", pin.id)))?
            .clone();
        if descriptor.version != pin.version || descriptor.content_hash != pin.content_hash {
            return Err(LspError::Invalid(format!(
                "descriptor pin for `{}` no longer matches the catalog",
                pin.id
            )));
        }
        self.enabled.insert(descriptor.id.clone());
        Ok(descriptor)
    }

    /// Import is schema validation plus an immutable, content-addressed copy. It never runs a
    /// repository-provided installer or reads an executable from the imported bundle.
    pub fn import_bundle(
        &mut self,
        source: impl AsRef<Path>,
        user_catalog: impl AsRef<Path>,
    ) -> Result<Vec<DescriptorPin>, LspError> {
        let source = source.as_ref();
        let text = fs::read_to_string(source)?;
        let file: CatalogFile = toml::from_str(&text)?;
        fs::create_dir_all(user_catalog.as_ref())?;
        let mut pins = Vec::new();
        for descriptor in file.descriptors {
            let descriptor = descriptor.with_hash()?;
            let pin = DescriptorPin {
                id: descriptor.id.clone(),
                version: descriptor.version,
                content_hash: descriptor.content_hash.clone(),
            };
            let destination = user_catalog.as_ref().join(format!(
                "{}-{}.toml",
                descriptor.id,
                &descriptor.content_hash[7..]
            ));
            let executable = descriptor
                .command
                .first()
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .ok_or_else(|| {
                    LspError::Invalid("imported LSP executable must be absolute".into())
                })?;
            let executable = fs::canonicalize(executable)?;
            if !fs::symlink_metadata(&executable)?.file_type().is_file() {
                return Err(LspError::Invalid(
                    "imported LSP executable is not a regular file".into(),
                ));
            }
            let executable_hash = sha256_file(&executable)?;
            let serialized = toml::to_string(&CatalogFile {
                descriptors: vec![descriptor.clone()],
                executable_hashes: BTreeMap::from([(
                    descriptor.id.clone(),
                    executable_hash.clone(),
                )]),
            })?;
            if destination.exists() {
                if !fs::symlink_metadata(&destination)?.file_type().is_file() {
                    return Err(LspError::Invalid(
                        "immutable descriptor copy is not a regular file".into(),
                    ));
                }
                let existing = fs::read_to_string(&destination)?;
                if existing != serialized {
                    return Err(LspError::Invalid(format!(
                        "immutable descriptor copy already differs: {}",
                        destination.display()
                    )));
                }
            } else {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&destination)?;
                file.write_all(serialized.as_bytes())?;
                file.sync_all()?;
            }
            if self.descriptors.contains_key(&descriptor.id) {
                return Err(LspError::Invalid(format!(
                    "descriptor id `{}` is already present; import under a new id",
                    descriptor.id
                )));
            }
            self.binary_hashes
                .insert(descriptor.id.clone(), executable_hash);
            self.descriptors.insert(descriptor.id.clone(), descriptor);
            // Import does not enable a descriptor. Detection can suggest it, but project state
            // must pin and explicitly enable the immutable copy before execution.
            pins.push(pin);
        }
        Ok(pins)
    }
}

pub fn detect_repository(
    root: impl AsRef<Path>,
    catalog: &LanguageCatalog,
) -> Result<Vec<DescriptorSuggestion>, LspError> {
    let root = root.as_ref();
    let mut files = Vec::new();
    collect_files(root, &mut files)?;
    files.sort();
    let mut suggestions = Vec::new();
    for descriptor in catalog.descriptors() {
        let matched_root_markers = descriptor
            .root_markers
            .iter()
            .filter(|marker| {
                files
                    .iter()
                    .any(|path| path.file_name().is_some_and(|name| name == marker.as_str()))
            })
            .cloned()
            .collect::<Vec<_>>();
        let matched_extensions = descriptor
            .extensions
            .iter()
            .filter(|extension| {
                files.iter().any(|path| {
                    path.extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|value| format!(".{value}") == **extension)
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        if !matched_root_markers.is_empty() || !matched_extensions.is_empty() {
            suggestions.push(DescriptorSuggestion {
                descriptor: DescriptorPin {
                    id: descriptor.id.clone(),
                    version: descriptor.version,
                    content_hash: descriptor.content_hash.clone(),
                },
                matched_root_markers,
                matched_extensions,
            });
        }
    }
    suggestions.sort_by(|left, right| left.descriptor.id.cmp(&right.descriptor.id));
    Ok(suggestions)
}

fn copy_pinned_executable(
    descriptor: &LanguageDescriptor,
    destination_root: &Path,
) -> Result<(), LspError> {
    let Some(command) = descriptor.command.first() else {
        return Ok(());
    };
    let source = PathBuf::from(command);
    if !source.is_absolute() {
        return Ok(());
    }
    let source = fs::canonicalize(source)?;
    if !fs::symlink_metadata(&source)?.file_type().is_file() {
        return Err(LspError::Invalid(
            "pinned LSP executable is not a regular file".into(),
        ));
    }
    let file_name = source
        .file_name()
        .ok_or_else(|| LspError::Invalid("pinned LSP executable has no file name".into()))?;
    let directory = destination_root.join("bin");
    fs::create_dir_all(&directory)?;
    let destination = directory.join(file_name);
    let expected = sha256_file(&source)?;
    if destination.exists() {
        if !fs::symlink_metadata(&destination)?.file_type().is_file()
            || sha256_file(&destination)? != expected
        {
            return Err(LspError::Invalid(format!(
                "provisioned LSP executable changed: {}",
                destination.display()
            )));
        }
    } else {
        fs::copy(&source, &destination)?;
        if sha256_file(&destination)? != expected {
            return Err(LspError::Invalid(
                "provisioned LSP executable hash changed while copying".into(),
            ));
        }
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, LspError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), LspError> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if entry.file_name() == ".git" {
                continue;
            }
            collect_files(&path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

pub fn preprovision(
    descriptor: &LanguageDescriptor,
    host_cache_root: impl AsRef<Path>,
    agent_image_root: impl AsRef<Path>,
) -> Result<ProvisionedDescriptor, LspError> {
    let verified = descriptor.clone().with_hash()?;
    if !descriptor.content_hash.is_empty() && descriptor.content_hash != verified.content_hash {
        return Err(LspError::Invalid(
            "descriptor content hash does not match its schema".into(),
        ));
    }
    let descriptor = verified;
    let pin = DescriptorPin {
        id: descriptor.id.clone(),
        version: descriptor.version,
        content_hash: descriptor.content_hash.clone(),
    };
    let host_cache = host_cache_root.as_ref().join(&descriptor.id);
    let agent_image_layer = agent_image_root.as_ref().join(&descriptor.id);
    fs::create_dir_all(&host_cache)?;
    fs::create_dir_all(&agent_image_layer)?;
    for directory in [&host_cache, &agent_image_layer] {
        copy_pinned_executable(&descriptor, directory)?;
    }
    let metadata = serde_json::to_vec_pretty(&descriptor)?;
    for directory in [&host_cache, &agent_image_layer] {
        let path = directory.join("descriptor.json");
        if path.exists() {
            let file_type = fs::symlink_metadata(&path)?.file_type();
            if !file_type.is_file() {
                return Err(LspError::Invalid(format!(
                    "provisioned descriptor is not a regular file: {}",
                    path.display()
                )));
            }
            let existing = fs::read(&path)?;
            if existing != metadata {
                return Err(LspError::Invalid(format!(
                    "provisioned descriptor changed: {}",
                    path.display()
                )));
            }
        } else {
            let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
            file.write_all(&metadata)?;
            file.sync_all()?;
        }
    }
    Ok(ProvisionedDescriptor {
        descriptor: pin,
        host_cache,
        agent_image_layer,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LspVerb {
    Definition,
    References,
    Hover,
    Symbols,
    Diagnostics,
    Rename,
}

impl LspVerb {
    fn descriptor_capability(&self) -> &'static str {
        match self {
            Self::Definition => "definition",
            Self::References => "references",
            Self::Hover => "hover",
            Self::Symbols => "documentSymbol",
            Self::Diagnostics => "diagnostics",
            Self::Rename => "rename",
        }
    }

    pub fn parse(value: &str) -> Result<Self, LspError> {
        match value {
            "lsp.def" => Ok(Self::Definition),
            "lsp.refs" => Ok(Self::References),
            "lsp.hover" => Ok(Self::Hover),
            "lsp.symbols" => Ok(Self::Symbols),
            "lsp.diagnostics" => Ok(Self::Diagnostics),
            "lsp.rename" => Ok(Self::Rename),
            other => Err(LspError::Unsupported(format!("unknown LSP verb `{other}`"))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LspCliRequest {
    pub verb: LspVerb,
    pub path: PathBuf,
    pub line: u32,
    pub character: u32,
    pub new_name: Option<String>,
}

pub fn parse_cli_request(verb: &str, args: &[String]) -> Result<LspCliRequest, LspError> {
    let verb = LspVerb::parse(verb)?;
    let path = args
        .first()
        .filter(|path| !path.trim().is_empty() && !path.starts_with('-'))
        .map(PathBuf::from)
        .ok_or_else(|| LspError::Invalid("LSP request requires a file path".into()))?;
    let (line, character, new_name) = match verb {
        LspVerb::Symbols | LspVerb::Diagnostics => {
            if args.len() != 1 {
                return Err(LspError::Invalid(
                    "symbols and diagnostics accept exactly one file path".into(),
                ));
            }
            (0, 0, None)
        }
        LspVerb::Rename => {
            if args.len() != 4 {
                return Err(LspError::Invalid(
                    "rename requires path, line, character, and new name".into(),
                ));
            }
            (
                parse_position(&args[1])?,
                parse_position(&args[2])?,
                Some(args[3].clone()),
            )
        }
        _ => {
            if args.len() != 3 {
                return Err(LspError::Invalid(
                    "LSP position requests require path, line, and character".into(),
                ));
            }
            (parse_position(&args[1])?, parse_position(&args[2])?, None)
        }
    };
    Ok(LspCliRequest {
        verb,
        path,
        line,
        character,
        new_name,
    })
}

fn parse_position(value: &str) -> Result<u32, LspError> {
    value
        .parse()
        .map_err(|_| LspError::Invalid(format!("invalid LSP position `{value}`")))
}

pub fn query_params(
    request: &LspCliRequest,
    workspace: impl AsRef<Path>,
) -> Result<(&'static str, Value), LspError> {
    let uri = workspace_file_uri(workspace.as_ref(), &request.path)?;
    let position = json!({"line": request.line, "character": request.character});
    let text_document = json!({"uri": uri});
    let (method, params) = match request.verb {
        LspVerb::Definition => (
            "textDocument/definition",
            json!({"textDocument": text_document, "position": position}),
        ),
        LspVerb::References => (
            "textDocument/references",
            json!({"textDocument": text_document, "position": position, "context": {"includeDeclaration": true}}),
        ),
        LspVerb::Hover => (
            "textDocument/hover",
            json!({"textDocument": text_document, "position": position}),
        ),
        LspVerb::Symbols => (
            "textDocument/documentSymbol",
            json!({"textDocument": text_document}),
        ),
        LspVerb::Diagnostics => (
            "textDocument/diagnostic",
            json!({"textDocument": text_document}),
        ),
        LspVerb::Rename => (
            "textDocument/rename",
            json!({"textDocument": text_document, "position": position, "newName": request.new_name.clone().unwrap_or_default()}),
        ),
    };
    Ok((method, params))
}

fn file_uri(path: &Path) -> Result<String, LspError> {
    Url::from_file_path(path)
        .map(|url| url.to_string())
        .map_err(|()| {
            LspError::Invalid(format!(
                "cannot convert path to file URI: {}",
                path.display()
            ))
        })
}

fn workspace_file_uri(workspace: &Path, requested: &Path) -> Result<String, LspError> {
    let workspace = fs::canonicalize(workspace)?;
    let candidate = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        workspace.join(requested)
    };
    let candidate = fs::canonicalize(&candidate)?;
    if !candidate.starts_with(&workspace) {
        return Err(LspError::Invalid("LSP path escapes the workspace".into()));
    }
    file_uri(&candidate)
}

fn validate_uri_in_workspace(workspace: &Path, uri: &str) -> Result<(), LspError> {
    let parsed =
        Url::parse(uri).map_err(|error| LspError::Invalid(format!("invalid LSP URI: {error}")))?;
    if parsed.scheme() != "file" {
        return Err(LspError::Invalid("LSP URI must use the file scheme".into()));
    }
    let path = parsed
        .to_file_path()
        .map_err(|()| LspError::Invalid("LSP URI is not a local file".into()))?;
    let root = fs::canonicalize(workspace)?;
    let path = fs::canonicalize(path)?;
    if !path.starts_with(root) {
        return Err(LspError::Invalid("LSP URI escapes the workspace".into()));
    }
    Ok(())
}

fn validate_lsp_params(workspace: &Path, method: &str, params: &Value) -> Result<(), LspError> {
    if let Some(uri) = params
        .get("textDocument")
        .and_then(Value::as_object)
        .and_then(|document| document.get("uri"))
        .and_then(Value::as_str)
    {
        validate_uri_in_workspace(workspace, uri)?;
    }
    if method == "initialize" {
        if let Some(uri) = params.get("rootUri").and_then(Value::as_str) {
            validate_uri_in_workspace(workspace, uri)?;
        }
        if let Some(folders) = params.get("workspaceFolders").and_then(Value::as_array) {
            for uri in folders
                .iter()
                .filter_map(|folder| folder.get("uri"))
                .filter_map(Value::as_str)
            {
                validate_uri_in_workspace(workspace, uri)?;
            }
        }
    }
    Ok(())
}

pub fn execute_query(
    verb: &str,
    args: &[String],
    workspace: impl AsRef<Path>,
    catalog: &LanguageCatalog,
) -> Result<Value, LspError> {
    let workspace = workspace.as_ref();
    let request = parse_cli_request(verb, args)?;
    let path = if request.path.is_absolute() {
        request.path.clone()
    } else {
        workspace.join(&request.path)
    };
    let descriptor = catalog.execution_descriptor_for_path(&path)?;
    execute_descriptor_query(&descriptor, &request, workspace)
}

/// Execute a descriptor granted by the host inside the caller's current workspace. The
/// descriptor is supplied separately so an agent can obtain a pinned command over the daemon
/// socket without ever asking the host to index or read the agent's clone.
pub fn execute_descriptor_query(
    descriptor: &LanguageDescriptor,
    request: &LspCliRequest,
    workspace: &Path,
) -> Result<Value, LspError> {
    let (method, params) = query_params(request, workspace)?;
    let capability = request.verb.descriptor_capability();
    if !descriptor.capabilities.contains(capability) {
        return Err(LspError::Unsupported(format!(
            "descriptor `{}` does not advertise `{capability}`",
            descriptor.id
        )));
    }
    let mut server = StdioLspServer::start(&descriptor.command, workspace)?;
    if !server.supports_method(method) {
        let _ = server.shutdown();
        return Err(LspError::Unsupported(format!(
            "LSP server does not advertise `{method}`"
        )));
    }
    let result = server.request(method, params);
    let _ = server.shutdown();
    result
}

/// Decode the LSP relative semantic-token stream into absolute token positions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SemanticToken {
    pub line: u32,
    pub start: u32,
    pub length: u32,
    pub token_type: u32,
    pub modifiers: u32,
}

pub fn decode_semantic_tokens(data: &[u32]) -> Result<Vec<SemanticToken>, LspError> {
    if !data.len().is_multiple_of(5) {
        return Err(LspError::Protocol(
            "semantic token data must contain five integers per token".into(),
        ));
    }
    let mut line: u32 = 0;
    let mut start: u32 = 0;
    let mut tokens = Vec::with_capacity(data.len() / 5);
    for chunk in data.chunks_exact(5) {
        line = line
            .checked_add(chunk[0])
            .ok_or_else(|| LspError::Protocol("semantic token line overflow".into()))?;
        start = if chunk[0] == 0 {
            start
                .checked_add(chunk[1])
                .ok_or_else(|| LspError::Protocol("semantic token column overflow".into()))?
        } else {
            chunk[1]
        };
        tokens.push(SemanticToken {
            line,
            start,
            length: chunk[2],
            token_type: chunk[3],
            modifiers: chunk[4],
        });
    }
    Ok(tokens)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticTokenDelta {
    pub start: usize,
    pub delete_count: usize,
    pub data: Vec<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct SemanticTokenCache {
    data: Vec<u32>,
    result_id: Option<String>,
}

impl SemanticTokenCache {
    pub fn replace(
        &mut self,
        result_id: Option<String>,
        data: Vec<u32>,
    ) -> Result<Vec<SemanticToken>, LspError> {
        if !data.len().is_multiple_of(5) {
            return Err(LspError::Protocol(
                "invalid semantic token replacement".into(),
            ));
        }
        self.data = data;
        self.result_id = result_id;
        decode_semantic_tokens(&self.data)
    }

    pub fn apply_delta(
        &mut self,
        result_id: Option<String>,
        edits: &[SemanticTokenDelta],
    ) -> Result<Vec<SemanticToken>, LspError> {
        let mut data = self.data.clone();
        for edit in edits.iter().rev() {
            let end = edit
                .start
                .checked_add(edit.delete_count)
                .ok_or_else(|| LspError::Protocol("semantic token delta overflow".into()))?;
            if end > data.len() || !edit.data.len().is_multiple_of(5) {
                return Err(LspError::Protocol(
                    "semantic token delta is out of bounds".into(),
                ));
            }
            data.splice(edit.start..end, edit.data.iter().copied());
        }
        self.data = data;
        self.result_id = result_id;
        decode_semantic_tokens(&self.data)
    }

    pub fn result_id(&self) -> Option<&str> {
        self.result_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    #[test]
    fn framed_messages_are_json_rpc_content_length_messages() {
        let frame = encode_frame(&json!({"jsonrpc":"2.0","id":1})).expect("encode");
        assert!(std::str::from_utf8(&frame)
            .unwrap()
            .starts_with("Content-Length:"));
        let body = frame
            .split(|byte| *byte == b'\n')
            .skip(2)
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(serde_json::from_slice::<Value>(&body).unwrap()["id"], 1);
    }

    #[test]
    fn semantic_tokens_decode_relative_positions() {
        let tokens = decode_semantic_tokens(&[0, 1, 2, 3, 0, 0, 4, 1, 4, 2]).expect("decode");
        assert_eq!(tokens[0].start, 1);
        assert_eq!((tokens[1].line, tokens[1].start), (0, 5));
    }

    #[test]
    fn semantic_delta_replaces_previous_data() {
        let mut cache = SemanticTokenCache::default();
        cache
            .replace(Some("one".into()), vec![0, 0, 1, 0, 0])
            .expect("replace");
        let tokens = cache.apply_delta(
            Some("two".into()),
            &[SemanticTokenDelta {
                start: 3,
                delete_count: 2,
                data: vec![2, 0],
            }],
        );
        assert!(tokens.is_err());
        assert_eq!(cache.result_id(), Some("one"));
    }

    #[test]
    fn descriptor_hash_is_stable_and_pins_are_exact() {
        let descriptor = LanguageDescriptor {
            id: "example".into(),
            version: 1,
            extensions: vec![".x".into()],
            root_markers: vec!["root.marker".into()],
            command: vec!["server".into()],
            capabilities: BTreeSet::new(),
            grammar: None,
            content_hash: String::new(),
        }
        .with_hash()
        .expect("descriptor");
        let mut catalog = LanguageCatalog::empty();
        catalog.insert(descriptor.clone()).expect("insert");
        let pin = catalog.pin("example").expect("pin");
        assert_eq!(
            catalog.enable(&pin).expect("enable").content_hash,
            descriptor.content_hash
        );
    }

    #[test]
    fn project_activation_uses_the_pinned_descriptor_for_matching_files() {
        let root = std::env::temp_dir().join(format!("locus-lsp-pin-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.rs"), "source").unwrap();
        fs::write(root.join("main.ts"), "source").unwrap();
        let catalog = LanguageCatalog::builtin().unwrap();
        let pin = catalog.pin("rust").unwrap();
        let host = LspHost::new(catalog);
        host.enable_project_descriptor(&root, pin).unwrap();
        assert_eq!(
            host.descriptor_for_project_path(&root, &root.join("main.rs"))
                .unwrap()
                .id,
            "rust"
        );
        assert!(matches!(
            host.descriptor_for_project_path(&root, &root.join("main.ts")),
            Err(LspError::Unsupported(message)) if message.contains(".ts")
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn query_params_use_clone_path_and_the_right_method() {
        let root = std::env::temp_dir().join(format!("locus-lsp-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.x"), "x").unwrap();
        let request = parse_cli_request("lsp.symbols", &["main.x".into()]).unwrap();
        let (method, params) = query_params(&request, &root).unwrap();
        assert_eq!(method, "textDocument/documentSymbol");
        assert!(params["textDocument"]["uri"]
            .as_str()
            .unwrap()
            .starts_with("file:"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn descriptor_queries_execute_against_the_current_clone_root() {
        let root = std::env::temp_dir().join(format!("locus-lsp-trees-{}", uuid::Uuid::new_v4()));
        let first = root.join("first");
        let second = root.join("second");
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        fs::write(first.join("main.rs"), "first tree").unwrap();
        fs::write(second.join("main.rs"), "second tree").unwrap();
        let server = root.join("server.py");
        fs::write(
            &server,
            r#"import json, os, sys

def read_message():
    length = None
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            return None
        if line in (b"\r\n", b"\n"):
            break
        if line.lower().startswith(b"content-length:"):
            length = int(line.split(b":", 1)[1].strip())
    if length is None:
        return None
    return json.loads(sys.stdin.buffer.read(length))

def send(message):
    body = json.dumps(message).encode()
    sys.stdout.buffer.write(f"Content-Length: {len(body)}\r\n\r\n".encode() + body)
    sys.stdout.buffer.flush()

while True:
    message = read_message()
    if message is None:
        break
    method = message.get("method")
    request_id = message.get("id")
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request_id, "result": {"capabilities": {"documentSymbolProvider": True}}})
    elif method == "shutdown":
        send({"jsonrpc": "2.0", "id": request_id, "result": None})
    elif request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "result": [{"name": os.getcwd()}]})
"#,
        )
        .unwrap();
        let descriptor = LanguageDescriptor {
            id: "tree-aware".into(),
            version: 1,
            extensions: vec![".rs".into()],
            root_markers: vec![],
            command: vec!["python3".into(), server.display().to_string()],
            capabilities: BTreeSet::from(["documentSymbol".into()]),
            grammar: None,
            content_hash: String::new(),
        };
        let request = parse_cli_request("lsp.symbols", &["main.rs".into()]).unwrap();
        let first_result = execute_descriptor_query(&descriptor, &request, &first).unwrap();
        let second_result = execute_descriptor_query(&descriptor, &request, &second).unwrap();
        assert_ne!(first_result[0]["name"], second_result[0]["name"]);
        assert!(first_result[0]["name"]
            .as_str()
            .unwrap()
            .ends_with("/first"));
        assert!(second_result[0]["name"]
            .as_str()
            .unwrap()
            .ends_with("/second"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn repository_detection_suggests_without_enabling_descriptors() {
        let root = std::env::temp_dir().join(format!("locus-lsp-detect-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
        fs::write(root.join("main.rs"), "fn main() {}\n").unwrap();
        let catalog = LanguageCatalog::builtin().unwrap();
        let suggestions = detect_repository(&root, &catalog).unwrap();
        let rust = suggestions
            .iter()
            .find(|suggestion| suggestion.descriptor.id == "rust")
            .expect("Rust is suggested from repository markers");
        assert_eq!(rust.matched_root_markers, ["Cargo.toml"]);
        assert_eq!(rust.matched_extensions, [".rs"]);
        assert!(catalog.for_path(&root.join("main.rs")).is_some());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn preprovision_copies_an_absolute_pinned_executable_to_both_layers() {
        let root =
            std::env::temp_dir().join(format!("locus-lsp-provision-{}", uuid::Uuid::new_v4()));
        let executable = std::env::current_exe().unwrap();
        let descriptor = LanguageDescriptor {
            id: "provisioned".into(),
            version: 1,
            extensions: vec![".prov".into()],
            root_markers: vec![],
            command: vec![executable.display().to_string()],
            capabilities: BTreeSet::new(),
            grammar: None,
            content_hash: String::new(),
        };
        let result = preprovision(&descriptor, root.join("host"), root.join("agent")).unwrap();
        let name = executable.file_name().unwrap();
        assert!(result.host_cache.join("bin").join(name).is_file());
        assert!(result.agent_image_layer.join("bin").join(name).is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn imported_descriptors_are_disabled_until_pinned_and_enabled() {
        let root = std::env::temp_dir().join(format!("locus-lsp-import-{}", uuid::Uuid::new_v4()));
        let user_catalog = root.join("catalog");
        fs::create_dir_all(&root).unwrap();
        let executable = std::env::current_exe().unwrap();
        let descriptor = LanguageDescriptor {
            id: "imported".into(),
            version: 1,
            extensions: vec![".imp".into()],
            root_markers: vec![],
            command: vec![executable.display().to_string()],
            capabilities: BTreeSet::new(),
            grammar: None,
            content_hash: String::new(),
        };
        let source = root.join("bundle.toml");
        fs::write(
            &source,
            toml::to_string(&CatalogFile {
                descriptors: vec![descriptor],
                executable_hashes: BTreeMap::new(),
            })
            .unwrap(),
        )
        .unwrap();
        fs::write(root.join("main.imp"), "source").unwrap();
        let mut catalog = LanguageCatalog::empty();
        let pins = catalog.import_bundle(&source, &user_catalog).unwrap();
        assert!(catalog.for_path(&root.join("main.imp")).is_none());
        catalog.enable(&pins[0]).unwrap();
        let resolved = catalog
            .execution_descriptor_for_path(&root.join("main.imp"))
            .unwrap();
        assert_eq!(resolved.command[0], executable.display().to_string());
        let reloaded = LanguageCatalog::load_user_catalog(&user_catalog).unwrap();
        assert!(reloaded.for_path(&root.join("main.imp")).is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ipc_uri_validation_rejects_a_file_outside_the_project() {
        let root = std::env::temp_dir().join(format!("locus-lsp-uri-{}", uuid::Uuid::new_v4()));
        let outside =
            std::env::temp_dir().join(format!("locus-lsp-outside-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::write(&outside, "outside").unwrap();
        let uri = file_uri(&outside).unwrap();
        assert!(validate_lsp_params(
            &root,
            "textDocument/hover",
            &json!({"textDocument": {"uri": uri}}),
        )
        .is_err());
        fs::remove_dir_all(root).unwrap();
        fs::remove_file(outside).unwrap();
    }

    struct FakeServer {
        responses: VecDeque<Result<Value, LspError>>,
        alive: bool,
        notify_fail: bool,
        notifications: Arc<Mutex<Vec<(String, Value)>>>,
    }
    impl LspServer for FakeServer {
        fn request(&mut self, _: &str, _: Value) -> Result<Value, LspError> {
            self.responses
                .pop_front()
                .unwrap_or_else(|| Ok(Value::Null))
        }
        fn notify(&mut self, method: &str, params: Value) -> Result<(), LspError> {
            if self.notify_fail {
                self.notify_fail = false;
                self.alive = false;
                return Err(LspError::ServerExited);
            }
            self.notifications
                .lock()
                .unwrap()
                .push((method.into(), params));
            Ok(())
        }
        fn take_notifications(&mut self) -> Vec<LspNotification> {
            Vec::new()
        }
        fn is_alive(&mut self) -> Result<bool, LspError> {
            Ok(self.alive)
        }
        fn shutdown(&mut self) -> Result<(), LspError> {
            self.alive = false;
            Ok(())
        }
    }
    struct FakeFactory {
        starts: Arc<Mutex<usize>>,
        notify_fail: bool,
        notifications: Arc<Mutex<Vec<(String, Value)>>>,
    }
    impl LspServerFactory for FakeFactory {
        fn start(
            &mut self,
            _: &LanguageDescriptor,
            _: &Path,
        ) -> Result<Box<dyn LspServer>, LspError> {
            let mut starts = self.starts.lock().unwrap();
            *starts += 1;
            let responses = if *starts == 1 {
                VecDeque::from([Err(LspError::ServerExited)])
            } else {
                VecDeque::from([Ok(json!({"ok": true}))])
            };
            Ok(Box::new(FakeServer {
                responses,
                alive: true,
                notify_fail: self.notify_fail && *starts == 1,
                notifications: self.notifications.clone(),
            }))
        }
    }

    fn descriptor() -> LanguageDescriptor {
        LanguageDescriptor {
            id: "example".into(),
            version: 1,
            extensions: vec![".x".into()],
            root_markers: vec![],
            command: vec!["server".into()],
            capabilities: BTreeSet::new(),
            grammar: None,
            content_hash: "hash".into(),
        }
    }

    #[test]
    fn restart_replays_open_documents_and_preserves_panes() {
        let starts = Arc::new(Mutex::new(0));
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let factory = FakeFactory {
            starts,
            notify_fail: false,
            notifications: notifications.clone(),
        };
        let mut supervisor = HostLspSupervisor::new(factory);
        let root = PathBuf::from("/tmp/example");
        supervisor
            .attach_pane(root.clone(), "pane", descriptor())
            .unwrap();
        let uri = "file:///tmp/example/main.x";
        supervisor
            .notify(
                root.clone(),
                "textDocument/didOpen",
                json!({"textDocument": {"uri": uri, "languageId": "example", "version": 1, "text": "x"}}),
            )
            .unwrap();
        supervisor.restart(&root).unwrap();
        assert_eq!(supervisor.project_info(&root).unwrap().pane_count, 1);
        assert_eq!(
            notifications
                .lock()
                .unwrap()
                .iter()
                .filter(|(method, _)| method == "textDocument/didOpen")
                .count(),
            2
        );
    }

    #[test]
    fn notify_restarts_a_dead_server_without_waiting_for_a_request() {
        let starts = Arc::new(Mutex::new(0));
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let factory = FakeFactory {
            starts: starts.clone(),
            notify_fail: true,
            notifications: notifications.clone(),
        };
        let mut supervisor = HostLspSupervisor::new(factory);
        let root = PathBuf::from("/tmp/example");
        supervisor
            .attach_pane(root.clone(), "pane", descriptor())
            .unwrap();
        supervisor
            .notify(
                root.clone(),
                "textDocument/didOpen",
                json!({"textDocument": {"uri": "file:///tmp/example/main.x", "version": 1}, "text": "x"}),
            )
            .unwrap();
        assert_eq!(*starts.lock().unwrap(), 2);
        assert_eq!(supervisor.project_info(&root).unwrap().restart_count, 1);
        assert_eq!(notifications.lock().unwrap().len(), 1);
    }

    #[test]
    fn mixed_descriptors_do_not_rebind_existing_panes_to_the_wrong_server() {
        let factory = FakeFactory {
            starts: Arc::new(Mutex::new(0)),
            notify_fail: false,
            notifications: Arc::new(Mutex::new(Vec::new())),
        };
        let mut supervisor = HostLspSupervisor::new(factory);
        let root = PathBuf::from("/tmp/example");
        supervisor
            .attach_pane(root.clone(), "rust", descriptor())
            .unwrap();
        let mut alternate = descriptor();
        alternate.id = "alternate".into();
        alternate.extensions = vec![".y".into()];
        alternate.content_hash = "alternate-hash".into();
        let error = supervisor
            .attach_pane(root.clone(), "alternate", alternate)
            .unwrap_err();
        assert!(matches!(error, LspError::Unsupported(message) if message.contains("example")));
        assert_eq!(supervisor.project_info(&root).unwrap().pane_count, 1);
    }

    #[test]
    fn supervisor_shares_one_server_and_restarts_after_failure() {
        let starts = Arc::new(Mutex::new(0));
        let factory = FakeFactory {
            starts: starts.clone(),
            notify_fail: false,
            notifications: Arc::new(Mutex::new(Vec::new())),
        };
        let mut supervisor = HostLspSupervisor::new(factory);
        let root = PathBuf::from("/tmp/example");
        let pane_a = supervisor
            .attach_pane(root.clone(), "a", descriptor())
            .unwrap();
        let _pane_b = supervisor
            .attach_pane(root.clone(), "b", descriptor())
            .unwrap();
        assert_eq!(supervisor.project_info(&root).unwrap().pane_count, 2);
        assert_eq!(
            supervisor.request(root.clone(), "x", Value::Null).unwrap()["ok"],
            true
        );
        assert_eq!(*starts.lock().unwrap(), 2);
        supervisor.detach_pane(&pane_a);
        assert_eq!(supervisor.project_info(&root).unwrap().pane_count, 1);
    }
}
