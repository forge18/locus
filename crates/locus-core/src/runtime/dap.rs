//! A bounded, harness-neutral Debug Adapter Protocol client and run-scoped debug sessions.
//!
//! The CLI is intentionally stateless: this module owns the adapter/session state keyed by
//! `RunId`.  The adapter process belongs to the run's container and is represented here by a
//! small lifecycle boundary so tests do not need a real debugger or Docker.

use crate::{
    ids::RunId,
    services::{
        mail::{idle_guardrail_applies, WaitingState},
        tools::{ProjectToolScope, RoleToolScope},
    },
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Read, Write},
    sync::{Arc, RwLock},
};
use thiserror::Error;

pub const MAX_DAP_FRAME_BYTES: usize = 1_048_576;
const MAX_DAP_HEADER_BYTES: usize = 16 * 1024;

#[derive(Debug, Error)]
pub enum DapError {
    #[error("DAP transport failed: {0}")]
    Io(#[from] io::Error),
    #[error("DAP message is malformed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("DAP frame has no valid Content-Length")]
    MissingContentLength,
    #[error("DAP frame exceeds {MAX_DAP_FRAME_BYTES} bytes")]
    FrameTooLarge,
    #[error("DAP response did not match request {0}")]
    MismatchedResponse(u64),
    #[error("unexpected DAP message type")]
    UnexpectedMessage,
    #[error("debug session for run {0} was not found")]
    SessionNotFound(RunId),
    #[error("debug adapter `{0}` is not available in the tool allowlist")]
    AdapterUnavailable(String),
    #[error("debug session is already running for {0}")]
    SessionAlreadyRunning(RunId),
    #[error("debug command requires a run-owned session")]
    InvalidRun,
    #[error("debug location must be FILE:LINE with a positive line")]
    InvalidLocation,
    #[error("debug expression is empty")]
    EmptyExpression,
    #[error("debug adapter request failed: {0}")]
    AdapterRequestFailed(String),
    #[error("debug adapter runtime is not available: {0}")]
    AdapterRuntimeUnavailable(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DapRequest {
    pub seq: u64,
    #[serde(rename = "type")]
    pub message_type: String,
    pub command: String,
    #[serde(default)]
    pub arguments: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DapResponse {
    pub seq: u64,
    #[serde(rename = "type")]
    pub message_type: String,
    pub request_seq: u64,
    pub success: bool,
    pub command: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub body: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DapEvent {
    pub seq: u64,
    #[serde(rename = "type")]
    pub message_type: String,
    pub event: String,
    #[serde(default)]
    pub body: Option<Value>,
}

/// A synchronous DAP client. Events received while waiting for a response are retained.
pub struct DapClient<T> {
    transport: T,
    next_seq: u64,
    max_frame_bytes: usize,
    events: Vec<DapEvent>,
}

impl<T: Read + Write> DapClient<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            next_seq: 1,
            max_frame_bytes: MAX_DAP_FRAME_BYTES,
            events: Vec::new(),
        }
    }

    pub fn with_frame_limit(mut self, max_frame_bytes: usize) -> Result<Self, DapError> {
        if max_frame_bytes == 0 || max_frame_bytes > MAX_DAP_FRAME_BYTES {
            return Err(DapError::FrameTooLarge);
        }
        self.max_frame_bytes = max_frame_bytes;
        Ok(self)
    }

    pub fn events(&self) -> &[DapEvent] {
        &self.events
    }

    pub fn take_events(&mut self) -> Vec<DapEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn initialize(&mut self) -> Result<DapResponse, DapError> {
        self.request(
            "initialize",
            json!({
                "clientID": "locus",
                "adapterID": "locus",
                "linesStartAt1": true,
                "columnsStartAt1": true,
                "supportsRunInTerminalRequest": false
            }),
        )
    }

    pub fn launch(&mut self, arguments: Value) -> Result<DapResponse, DapError> {
        self.request("launch", arguments)
    }

    fn send_request(&mut self, command: &str, arguments: Value) -> Result<u64, DapError> {
        let request_seq = self.next_seq;
        self.next_seq += 1;
        let request = DapRequest {
            seq: request_seq,
            message_type: "request".into(),
            command: command.into(),
            arguments: Some(arguments),
        };
        write_frame(
            &mut self.transport,
            &serde_json::to_vec(&request)?,
            self.max_frame_bytes,
        )?;
        Ok(request_seq)
    }

    pub fn request(&mut self, command: &str, arguments: Value) -> Result<DapResponse, DapError> {
        let request_seq = self.send_request(command, arguments)?;
        loop {
            let value = read_frame(&mut self.transport, self.max_frame_bytes)?;
            match value.get("type").and_then(Value::as_str) {
                Some("event") => self.events.push(serde_json::from_value(value)?),
                Some("response") => {
                    let response: DapResponse = serde_json::from_value(value)?;
                    if response.request_seq != request_seq {
                        return Err(DapError::MismatchedResponse(request_seq));
                    }
                    return Ok(response);
                }
                _ => return Err(DapError::UnexpectedMessage),
            }
        }
    }
}

/// The process boundary owned by a run-scoped debug session.
///
/// A real implementation is backed by [`DapClientProcess`]. The recording implementation is
/// used for Docker-free core tests and keeps lifecycle behavior observable without starting a
/// host process.
pub trait DebugAdapterProcess: Send + Sync {
    fn initialize_and_launch(&mut self, launch_arguments: Value) -> Result<(), DapError> {
        let initialize = self.request(
            "initialize",
            json!({
                "clientID": "locus",
                "adapterID": "locus",
                "linesStartAt1": true,
                "columnsStartAt1": true,
                "supportsRunInTerminalRequest": false
            }),
        )?;
        ensure_success(&initialize)?;
        let launch = self.request("launch", launch_arguments)?;
        ensure_success(&launch)
    }

    fn request(&mut self, command: &str, arguments: Value) -> Result<DapResponse, DapError>;
    fn terminate(&mut self);
    fn drain_events(&mut self) -> Vec<DapEvent> {
        Vec::new()
    }
    fn is_alive(&self) -> bool;
}

pub struct DapClientProcess<T> {
    client: DapClient<T>,
    alive: bool,
}

impl<T: Read + Write> DapClientProcess<T> {
    pub fn new(transport: T) -> Self {
        Self {
            client: DapClient::new(transport),
            alive: true,
        }
    }

    pub fn initialize_and_launch(&mut self, launch_arguments: Value) -> Result<(), DapError> {
        let initialize = self.client.initialize()?;
        ensure_success(&initialize)?;
        let launch = self.client.launch(launch_arguments)?;
        ensure_success(&launch)?;
        Ok(())
    }

    pub fn events(&self) -> &[DapEvent] {
        self.client.events()
    }
}

impl<T: Read + Write + Send + Sync> DebugAdapterProcess for DapClientProcess<T> {
    fn initialize_and_launch(&mut self, launch_arguments: Value) -> Result<(), DapError> {
        DapClientProcess::initialize_and_launch(self, launch_arguments)
    }

    fn request(&mut self, command: &str, arguments: Value) -> Result<DapResponse, DapError> {
        if !self.alive {
            return Err(DapError::AdapterRequestFailed("adapter is stopped".into()));
        }
        self.client.request(command, arguments)
    }

    fn terminate(&mut self) {
        if self.alive {
            // Do not wait for a response while tearing down a run. An adapter that is already
            // paused or exiting must not make run cleanup block; dropping the transport closes
            // its container exec stdin immediately afterward.
            let _ = self
                .client
                .send_request("disconnect", json!({"terminateDebuggee": true}));
        }
        self.alive = false;
    }

    fn drain_events(&mut self) -> Vec<DapEvent> {
        self.client.take_events()
    }

    fn is_alive(&self) -> bool {
        self.alive
    }
}

#[derive(Default)]
struct RecordingAdapterProcess {
    alive: bool,
}

impl RecordingAdapterProcess {
    fn running() -> Self {
        Self { alive: true }
    }
}

impl DebugAdapterProcess for RecordingAdapterProcess {
    fn request(&mut self, command: &str, _arguments: Value) -> Result<DapResponse, DapError> {
        if !self.alive {
            return Err(DapError::AdapterRequestFailed("adapter is stopped".into()));
        }
        Ok(DapResponse {
            seq: 0,
            message_type: "response".into(),
            request_seq: 0,
            success: true,
            command: command.into(),
            message: None,
            body: Some(match command {
                "stackTrace" => json!({"stackFrames": []}),
                "scopes" => json!({"scopes": []}),
                "evaluate" => json!({"result": "<adapter-evaluation>"}),
                _ => json!({}),
            }),
        })
    }

    fn terminate(&mut self) {
        self.alive = false;
    }

    fn is_alive(&self) -> bool {
        self.alive
    }
}

fn ensure_success(response: &DapResponse) -> Result<(), DapError> {
    if response.success {
        Ok(())
    } else {
        Err(DapError::AdapterRequestFailed(
            response
                .message
                .clone()
                .unwrap_or_else(|| format!("{} failed", response.command)),
        ))
    }
}

fn write_frame<T: Write>(
    writer: &mut T,
    payload: &[u8],
    max_frame_bytes: usize,
) -> Result<(), DapError> {
    if payload.len() > max_frame_bytes || payload.len() > MAX_DAP_FRAME_BYTES {
        return Err(DapError::FrameTooLarge);
    }
    write!(writer, "Content-Length: {}\r\n\r\n", payload.len())?;
    writer.write_all(payload)?;
    writer.flush()?;
    Ok(())
}

fn read_frame<T: Read>(reader: &mut T, max_frame_bytes: usize) -> Result<Value, DapError> {
    let mut header = Vec::new();
    let mut byte = [0_u8; 1];
    while !header.ends_with(b"\r\n\r\n") {
        if header.len() >= MAX_DAP_HEADER_BYTES {
            return Err(DapError::MissingContentLength);
        }
        reader.read_exact(&mut byte)?;
        header.push(byte[0]);
    }
    let header = std::str::from_utf8(&header).map_err(|_| DapError::MissingContentLength)?;
    let length = header
        .lines()
        .find_map(|line| {
            line.strip_prefix("Content-Length:")?
                .trim()
                .parse::<usize>()
                .ok()
        })
        .ok_or(DapError::MissingContentLength)?;
    if length > max_frame_bytes {
        return Err(DapError::FrameTooLarge);
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    Ok(serde_json::from_slice(&body)?)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugStatus {
    Running,
    Paused,
    Stopped,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DebugBreakpoint {
    pub file: String,
    pub line: u32,
    pub condition: Option<String>,
    pub log_message: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DebugSessionSnapshot {
    pub run_id: RunId,
    /// The marketplace/plugin id, never a language-specific branch in the core.
    pub adapter: String,
    pub run_command: String,
    pub status: DebugStatus,
    pub adapter_in_container: bool,
    pub breakpoints: Vec<DebugBreakpoint>,
    pub events: Vec<DebugEvent>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DebugEvent {
    pub kind: String,
    pub payload: Value,
}

struct DebugSession {
    snapshot: DebugSessionSnapshot,
    breakpoint_map: BTreeMap<(String, u32), DebugBreakpoint>,
    adapter_allowlist: BTreeSet<String>,
    adapter_process: Box<dyn DebugAdapterProcess>,
}

/// Core-held debug state. Cloning this value shares the registry; it does not copy sessions.
#[derive(Clone, Default)]
pub struct DebugSessionRegistry {
    sessions: Arc<RwLock<BTreeMap<RunId, DebugSession>>>,
}

impl DebugSessionRegistry {
    /// Start a Docker-free recording session for unit tests. Production callers must use
    /// [`Self::start_with_process`] with a process returned by
    /// [`crate::runtime::container::ContainerRuntime`].
    pub fn start(
        &self,
        run_id: RunId,
        adapter: impl Into<String>,
        run_command: impl Into<String>,
        allowlisted_adapters: impl IntoIterator<Item = String>,
    ) -> Result<DebugSessionSnapshot, DapError> {
        self.start_with_process(
            run_id,
            adapter,
            run_command,
            allowlisted_adapters,
            Box::new(RecordingAdapterProcess::running()),
        )
    }

    /// Start a session with the adapter process created by the container runtime.
    pub fn start_with_process(
        &self,
        run_id: RunId,
        adapter: impl Into<String>,
        run_command: impl Into<String>,
        allowlisted_adapters: impl IntoIterator<Item = String>,
        mut adapter_process: Box<dyn DebugAdapterProcess>,
    ) -> Result<DebugSessionSnapshot, DapError> {
        let adapter = adapter.into();
        let run_command = run_command.into();
        let allowlisted_adapters: BTreeSet<_> = allowlisted_adapters.into_iter().collect();
        if run_command.trim().is_empty() {
            return Err(DapError::InvalidRun);
        }
        if !allowlisted_adapters.contains(&adapter) {
            return Err(DapError::AdapterUnavailable(adapter));
        }
        adapter_process.initialize_and_launch(json!({"command": run_command}))?;
        if !adapter_process.is_alive() {
            return Err(DapError::AdapterRuntimeUnavailable(
                "adapter exited during startup".into(),
            ));
        }
        let mut sessions = self.sessions.write().expect("debug registry lock");
        if sessions.contains_key(&run_id) {
            return Err(DapError::SessionAlreadyRunning(run_id));
        }
        let snapshot = DebugSessionSnapshot {
            run_id,
            adapter,
            run_command,
            status: DebugStatus::Running,
            adapter_in_container: true,
            breakpoints: Vec::new(),
            events: vec![DebugEvent {
                kind: "session_started".into(),
                payload: json!({"run_id": run_id}),
            }],
        };
        sessions.insert(
            run_id,
            DebugSession {
                snapshot: snapshot.clone(),
                breakpoint_map: BTreeMap::new(),
                adapter_allowlist: allowlisted_adapters,
                adapter_process,
            },
        );
        Ok(snapshot)
    }

    pub fn snapshot(&self, run_id: RunId) -> Result<DebugSessionSnapshot, DapError> {
        let mut sessions = self.sessions.write().expect("debug registry lock");
        let session = sessions
            .get_mut(&run_id)
            .ok_or(DapError::SessionNotFound(run_id))?;
        absorb_adapter_events(session);
        Ok(session.snapshot.clone())
    }

    /// Replace the Docker-free recording process with a live DAP transport owned by this run.
    /// The caller is responsible for creating the transport to an adapter inside the run's
    /// container; the registry owns it until `stop` or `end_run`.
    pub fn attach_transport<T: Read + Write + Send + Sync + 'static>(
        &self,
        run_id: RunId,
        transport: T,
        launch_arguments: Value,
    ) -> Result<DebugSessionSnapshot, DapError> {
        let mut process = DapClientProcess::new(transport);
        process.initialize_and_launch(launch_arguments)?;
        let mut sessions = self.sessions.write().expect("debug registry lock");
        let session = sessions
            .get_mut(&run_id)
            .ok_or(DapError::SessionNotFound(run_id))?;
        session.adapter_process = Box::new(process);
        session.snapshot.adapter_in_container = true;
        absorb_adapter_events(session);
        Ok(session.snapshot.clone())
    }

    pub fn set_breakpoint(
        &self,
        run_id: RunId,
        location: &str,
        condition: Option<String>,
        log_message: Option<String>,
    ) -> Result<DebugSessionSnapshot, DapError> {
        let (file, line) = parse_location(location)?;
        if condition
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(DapError::EmptyExpression);
        }
        let mut sessions = self.sessions.write().expect("debug registry lock");
        let session = sessions
            .get_mut(&run_id)
            .ok_or(DapError::SessionNotFound(run_id))?;
        let breakpoint = DebugBreakpoint {
            file: file.to_owned(),
            line,
            condition,
            log_message,
        };
        let mut breakpoints = session
            .breakpoint_map
            .values()
            .filter(|existing| existing.file == breakpoint.file && existing.line != breakpoint.line)
            .cloned()
            .collect::<Vec<_>>();
        breakpoints.push(breakpoint.clone());
        let response = session.adapter_process.request(
            "setBreakpoints",
            json!({
                "source": {"path": breakpoint.file},
                "breakpoints": breakpoints.iter().map(|value| json!({
                    "line": value.line,
                    "condition": value.condition,
                    "logMessage": value.log_message
                })).collect::<Vec<_>>()
            }),
        )?;
        ensure_success(&response)?;
        if response
            .body
            .as_ref()
            .and_then(|body| body.get("breakpoints"))
            .and_then(Value::as_array)
            .is_some_and(|breakpoints| {
                breakpoints
                    .iter()
                    .any(|breakpoint| breakpoint.get("verified") == Some(&Value::Bool(false)))
            })
        {
            return Err(DapError::AdapterRequestFailed(
                "adapter did not verify the breakpoint".into(),
            ));
        }
        absorb_adapter_events(session);
        session
            .breakpoint_map
            .insert((file.into(), line), breakpoint);
        session.snapshot.breakpoints = session.breakpoint_map.values().cloned().collect();
        session.snapshot.events.push(DebugEvent {
            kind: "breakpoint_set".into(),
            payload: json!({"file": file, "line": line}),
        });
        Ok(session.snapshot.clone())
    }

    pub fn hit_breakpoint(
        &self,
        run_id: RunId,
        file: &str,
        line: u32,
    ) -> Result<DebugSessionSnapshot, DapError> {
        let mut sessions = self.sessions.write().expect("debug registry lock");
        let session = sessions
            .get_mut(&run_id)
            .ok_or(DapError::SessionNotFound(run_id))?;
        let Some(breakpoint) = session.breakpoint_map.get(&(file.to_owned(), line)) else {
            return Ok(session.snapshot.clone());
        };
        if let Some(message) = &breakpoint.log_message {
            session.snapshot.events.push(DebugEvent {
                kind: "logpoint".into(),
                payload: json!({"file": file, "line": line, "message": message}),
            });
        } else {
            session.snapshot.status = DebugStatus::Paused;
            session.snapshot.events.push(DebugEvent {
                kind: "breakpoint".into(),
                payload: json!({"file": file, "line": line}),
            });
        }
        Ok(session.snapshot.clone())
    }

    pub fn command(
        &self,
        run_id: RunId,
        command: &str,
        arguments: Value,
    ) -> Result<Value, DapError> {
        let mut sessions = self.sessions.write().expect("debug registry lock");
        let session = sessions
            .get_mut(&run_id)
            .ok_or(DapError::SessionNotFound(run_id))?;
        if !session
            .adapter_allowlist
            .contains(&session.snapshot.adapter)
        {
            return Err(DapError::AdapterUnavailable(
                session.snapshot.adapter.clone(),
            ));
        }
        let (dap_command, dap_arguments) = match command {
            "run" | "continue" => ("continue", arguments.clone()),
            "next" => ("next", arguments.clone()),
            "step" => ("stepIn", arguments.clone()),
            "finish" => ("stepOut", arguments.clone()),
            "stack" => ("stackTrace", arguments.clone()),
            "vars" => ("scopes", arguments.clone()),
            "eval" => {
                let expression = arguments
                    .get("expression")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if expression.trim().is_empty() {
                    return Err(DapError::EmptyExpression);
                }
                ("evaluate", arguments.clone())
            }
            _ => return Err(DapError::InvalidRun),
        };
        let response = session
            .adapter_process
            .request(dap_command, dap_arguments)?;
        ensure_success(&response)?;
        let paused_from_event = absorb_adapter_events(session);
        if matches!(command, "run" | "continue" | "next" | "step" | "finish") && !paused_from_event
        {
            session.snapshot.status = DebugStatus::Running;
            session.snapshot.events.push(DebugEvent {
                kind: command.into(),
                payload: arguments.clone(),
            });
        }
        let body = response.body.unwrap_or_else(|| json!({}));
        Ok(json!({
            "status": status_name(session.snapshot.status),
            "command": command,
            "body": body
        }))
    }

    pub fn waiting_state(&self, run_id: RunId) -> Result<Option<WaitingState>, DapError> {
        let snapshot = self.snapshot(run_id)?;
        Ok((snapshot.status == DebugStatus::Paused).then(|| {
            WaitingState::from_debug_breakpoint(run_id, json!({"adapter": snapshot.adapter}))
        }))
    }

    pub fn idle_guardrail_applies(&self, run_id: RunId) -> Result<bool, DapError> {
        Ok(idle_guardrail_applies(self.waiting_state(run_id)?.as_ref()))
    }

    pub fn stop(&self, run_id: RunId) -> Result<DebugSessionSnapshot, DapError> {
        let mut sessions = self.sessions.write().expect("debug registry lock");
        let mut session = sessions
            .remove(&run_id)
            .ok_or(DapError::SessionNotFound(run_id))?;
        session.adapter_process.terminate();
        session.snapshot.status = DebugStatus::Stopped;
        session.snapshot.adapter_in_container = false;
        session.snapshot.events.push(DebugEvent {
            kind: "session_stopped".into(),
            payload: json!({"run_id": run_id}),
        });
        Ok(session.snapshot)
    }

    pub fn end_run(&self, run_id: RunId) -> bool {
        let mut sessions = self.sessions.write().expect("debug registry lock");
        let Some(mut session) = sessions.remove(&run_id) else {
            return false;
        };
        session.adapter_process.terminate();
        true
    }

    pub fn adapters_are_tools(
        project: &ProjectToolScope,
        role: &RoleToolScope,
        adapter: &str,
    ) -> bool {
        project.permits(adapter) && role.permits(adapter)
    }
}

fn absorb_adapter_events(session: &mut DebugSession) -> bool {
    let mut paused = false;
    for event in session.adapter_process.drain_events() {
        match event.event.as_str() {
            "stopped" => {
                session.snapshot.status = DebugStatus::Paused;
                paused = true;
            }
            "continued" => session.snapshot.status = DebugStatus::Running,
            "terminated" | "exited" => {
                session.snapshot.status = DebugStatus::Stopped;
                session.snapshot.adapter_in_container = false;
            }
            _ => {}
        }
        session.snapshot.events.push(DebugEvent {
            kind: event.event,
            payload: event.body.unwrap_or_else(|| json!({})),
        });
    }
    paused
}

fn status_name(status: DebugStatus) -> &'static str {
    match status {
        DebugStatus::Running => "running",
        DebugStatus::Paused => "paused",
        DebugStatus::Stopped => "stopped",
    }
}

fn parse_location(location: &str) -> Result<(&str, u32), DapError> {
    let (file, line) = location.rsplit_once(':').ok_or(DapError::InvalidLocation)?;
    let line = line.parse::<u32>().map_err(|_| DapError::InvalidLocation)?;
    if file.trim().is_empty() || line == 0 {
        return Err(DapError::InvalidLocation);
    }
    Ok((file, line))
}

pub const DEBUG_DOCS: &str =
    "Prefer --log logpoints before breakpoints: logpoints continue; breakpoints stop the program.";

#[cfg(test)]
mod dap {
    use super::*;
    use std::io::Cursor;

    struct MemoryTransport {
        incoming: Cursor<Vec<u8>>,
        outgoing: Vec<u8>,
    }

    impl Read for MemoryTransport {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            self.incoming.read(buffer)
        }
    }

    impl Write for MemoryTransport {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.outgoing.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn response(seq: u64, command: &str) -> Vec<u8> {
        let body = serde_json::to_vec(&json!({
            "seq": seq + 1,
            "type": "response",
            "request_seq": seq,
            "success": true,
            "command": command
        }))
        .unwrap();
        let mut bytes = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
        bytes.extend(body);
        bytes
    }

    #[test]
    fn client() {
        let transport = MemoryTransport {
            incoming: Cursor::new(response(1, "initialize")),
            outgoing: Vec::new(),
        };
        let mut client = DapClient::new(transport);
        assert!(client.initialize().unwrap().success);
    }

    #[test]
    fn session_in_core() {
        let registry = DebugSessionRegistry::default();
        let run = RunId::generate();
        registry
            .start(run, "debugpy", "python app.py", ["debugpy".into()])
            .unwrap();
        assert_eq!(registry.snapshot(run).unwrap().run_id, run);
    }

    #[test]
    fn adapter_in_container() {
        let registry = DebugSessionRegistry::default();
        let snapshot = registry
            .start(
                RunId::generate(),
                "codelldb",
                "cargo run",
                ["codelldb".into()],
            )
            .unwrap();
        assert!(snapshot.adapter_in_container);
    }

    #[test]
    fn same_run_command() {
        let registry = DebugSessionRegistry::default();
        let run = RunId::generate();
        registry
            .start(run, "debugpy", "python -m app", ["debugpy".into()])
            .unwrap();
        assert_eq!(registry.snapshot(run).unwrap().run_command, "python -m app");
    }

    #[test]
    fn breakpoint_persists() {
        let registry = DebugSessionRegistry::default();
        let run = RunId::generate();
        registry
            .start(run, "debugpy", "python app.py", ["debugpy".into()])
            .unwrap();
        registry
            .set_breakpoint(run, "app.py:12", Some("x > 0".into()), None)
            .unwrap();
        assert_eq!(registry.snapshot(run).unwrap().breakpoints[0].line, 12);
    }

    #[test]
    fn pause_suppresses_idle() {
        let registry = DebugSessionRegistry::default();
        let run = RunId::generate();
        registry
            .start(run, "debugpy", "python app.py", ["debugpy".into()])
            .unwrap();
        registry
            .set_breakpoint(run, "app.py:12", None, None)
            .unwrap();
        registry.hit_breakpoint(run, "app.py", 12).unwrap();
        assert!(!registry.idle_guardrail_applies(run).unwrap());
    }

    #[test]
    fn long_pause_no_trip() {
        let registry = DebugSessionRegistry::default();
        let run = RunId::generate();
        registry
            .start(run, "debugpy", "python app.py", ["debugpy".into()])
            .unwrap();
        registry
            .set_breakpoint(run, "app.py:12", None, None)
            .unwrap();
        registry.hit_breakpoint(run, "app.py", 12).unwrap();
        assert!(registry.waiting_state(run).unwrap().is_some());
        assert!(!registry.idle_guardrail_applies(run).unwrap());
    }

    #[test]
    fn adapters_are_tools() {
        let project = ProjectToolScope::default();
        let role = RoleToolScope::default();
        assert!(DebugSessionRegistry::adapters_are_tools(
            &project, &role, "debugpy"
        ));
        let project = ProjectToolScope::new(["debugpy"]);
        assert!(!DebugSessionRegistry::adapters_are_tools(
            &project, &role, "debugpy"
        ));
    }

    #[test]
    fn adapter_dies_with_run() {
        let registry = DebugSessionRegistry::default();
        let run = RunId::generate();
        registry
            .start(run, "debugpy", "python app.py", ["debugpy".into()])
            .unwrap();
        assert!(registry.end_run(run));
        assert!(matches!(
            registry.snapshot(run),
            Err(DapError::SessionNotFound(_))
        ));
    }

    #[test]
    fn docs_prefer_logpoints() {
        assert!(DEBUG_DOCS.contains("logpoints"));
        assert!(DEBUG_DOCS.contains("continue"));
    }
}
