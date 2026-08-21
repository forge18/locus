//! Capture adapters that normalize every harness into the same closed event vocabulary.
//!
//! Adapters deliberately return unsequenced [`CapturedEvent`] values. [`EventCollector`]
//! is the sole place that assigns sequence numbers, so source ordering cannot leak into
//! downstream consumers.

use std::{
    collections::{BTreeSet, HashMap},
    io::Read,
    sync::{Arc, Mutex},
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
#[cfg(test)]
use serde_json::json;
use serde_json::{Map, Value};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::sync::broadcast;

/// The only event verbs persisted by Locus.
///
/// There is intentionally no catch-all variant: adding a thirteenth verb requires
/// changing this type and every exhaustive match over it at compile time.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventVerb {
    SessionStart,
    User,
    Assistant,
    Thinking,
    ToolCall,
    ToolResult,
    ToolError,
    PermissionRequest,
    SubagentStart,
    SubagentStop,
    Aborted,
    SessionEnd,
}

impl EventVerb {
    pub const ALL: [Self; 12] = [
        Self::SessionStart,
        Self::User,
        Self::Assistant,
        Self::Thinking,
        Self::ToolCall,
        Self::ToolResult,
        Self::ToolError,
        Self::PermissionRequest,
        Self::SubagentStart,
        Self::SubagentStop,
        Self::Aborted,
        Self::SessionEnd,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SessionStart => "session_start",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Thinking => "thinking",
            Self::ToolCall => "tool_call",
            Self::ToolResult => "tool_result",
            Self::ToolError => "tool_error",
            Self::PermissionRequest => "permission_request",
            Self::SubagentStart => "subagent_start",
            Self::SubagentStop => "subagent_stop",
            Self::Aborted => "aborted",
            Self::SessionEnd => "session_end",
        }
    }
}

impl std::fmt::Display for EventVerb {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Token counts supplied by a harness. Missing fields remain absent; Locus never derives them.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub input: Option<u64>,
    pub output: Option<u64>,
    pub cache_read: Option<u64>,
    pub cache_write: Option<u64>,
}

impl Usage {
    fn from_value(value: Option<&Value>) -> Option<Self> {
        let usage = value?.as_object()?;
        let count = |keys: &[&str]| {
            keys.iter()
                .find_map(|key| usage.get(*key).and_then(Value::as_u64))
        };
        let usage = Self {
            input: count(&["input", "input_tokens", "prompt_tokens"]),
            output: count(&["output", "output_tokens", "completion_tokens"]),
            cache_read: count(&["cache_read", "cache_read_tokens", "cache_read_input_tokens"]),
            cache_write: count(&[
                "cache_write",
                "cache_write_tokens",
                "cache_creation_input_tokens",
            ]),
        };
        (usage != Self::default()).then_some(usage)
    }
}

/// Source-normalized data before core-assigned ordering.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CapturedEvent {
    pub verb: EventVerb,
    pub ts: String,
    pub text: Option<String>,
    pub tool: Option<String>,
    pub args: Option<Value>,
    pub usage: Option<Usage>,
    pub raw: Value,
}

impl CapturedEvent {
    fn from_raw(verb: EventVerb, raw: Value) -> Self {
        let object = raw.as_object();
        Self {
            verb,
            ts: timestamp_from(object),
            text: object.and_then(text_from),
            tool: object.and_then(tool_from),
            args: object.and_then(args_from),
            usage: object.and_then(|object| Usage::from_value(object.get("usage"))),
            raw,
        }
    }
}

/// The downstream event shape, independent of which capture path produced it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub run_id: String,
    pub seq: u64,
    pub ts: String,
    pub verb: EventVerb,
    pub text: Option<String>,
    pub tool: Option<String>,
    pub args: Option<Value>,
    /// `None` means the harness did not report usage, never zero usage.
    pub usage: Option<Usage>,
    /// Original source record, retained for parser repair and replay.
    pub raw: Value,
}

impl Event {
    fn from_captured(run_id: String, seq: u64, captured: CapturedEvent) -> Self {
        Self {
            run_id,
            seq,
            ts: captured.ts,
            verb: captured.verb,
            text: captured.text,
            tool: captured.tool,
            args: captured.args,
            usage: captured.usage,
            raw: captured.raw,
        }
    }
}

/// A permission prompt is an alarm: a noninteractive run would otherwise hang.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermissionAlarm {
    pub run_id: String,
    pub seq: u64,
}

/// Owns per-run sequence assignment and publishes normalized events to local consumers.
#[derive(Clone)]
pub struct EventCollector {
    next_seq: Arc<Mutex<HashMap<String, u64>>>,
    events: Arc<Mutex<Vec<Event>>>,
    sender: broadcast::Sender<Event>,
    alarms: broadcast::Sender<PermissionAlarm>,
}

impl EventCollector {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        let (alarms, _) = broadcast::channel(capacity);
        Self {
            next_seq: Arc::new(Mutex::new(HashMap::new())),
            events: Arc::new(Mutex::new(Vec::new())),
            sender,
            alarms,
        }
    }

    pub fn capture(&self, run_id: impl Into<String>, captured: CapturedEvent) -> Event {
        let run_id = run_id.into();
        let seq = {
            let mut sequences = self.next_seq.lock().expect("event sequence lock");
            let next = sequences.entry(run_id.clone()).or_insert(0);
            let assigned = *next;
            *next += 1;
            assigned
        };
        let event = Event::from_captured(run_id, seq, captured);
        self.events
            .lock()
            .expect("event store lock")
            .push(event.clone());
        let _ = self.sender.send(event.clone());
        if event.verb == EventVerb::PermissionRequest {
            let _ = self.alarms.send(PermissionAlarm {
                run_id: event.run_id.clone(),
                seq: event.seq,
            });
        }
        event
    }

    pub fn events_for(&self, run_id: &str) -> Vec<Event> {
        self.events
            .lock()
            .expect("event store lock")
            .iter()
            .filter(|event| event.run_id == run_id)
            .cloned()
            .collect()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    pub fn subscribe_alarms(&self) -> broadcast::Receiver<PermissionAlarm> {
        self.alarms.subscribe()
    }
}

/// Normalizes one raw record. Kept separate from collection so a run can be replayed.
pub trait Adapter {
    fn normalize(&self, raw: Value) -> Result<Vec<CapturedEvent>>;
}

/// Replays saved source records through a repaired parser without rerunning an agent.
pub fn replay(
    adapter: &dyn Adapter,
    raw_records: impl IntoIterator<Item = Value>,
) -> Result<Vec<CapturedEvent>> {
    raw_records
        .into_iter()
        .try_fold(Vec::new(), |mut events, raw| {
            events.extend(adapter.normalize(raw)?);
            Ok(events)
        })
}

/// A per-harness hook event-name table. Unmapped hooks are deliberately absent.
#[derive(Clone, Debug, Default)]
pub struct HooksAdapter {
    tables: HashMap<String, HashMap<String, EventVerb>>,
}

impl HooksAdapter {
    pub fn with_table(
        harness: impl Into<String>,
        table: impl IntoIterator<Item = (impl Into<String>, EventVerb)>,
    ) -> Self {
        let mut tables = HashMap::new();
        tables.insert(
            harness.into(),
            table
                .into_iter()
                .map(|(name, verb)| (name.into(), verb))
                .collect(),
        );
        Self { tables }
    }

    pub fn insert_table(
        &mut self,
        harness: impl Into<String>,
        table: impl IntoIterator<Item = (impl Into<String>, EventVerb)>,
    ) {
        self.tables.insert(
            harness.into(),
            table
                .into_iter()
                .map(|(name, verb)| (name.into(), verb))
                .collect(),
        );
    }

    /// Hook capture is merged with transcript capture by callers: hooks carry rich
    /// tool data; transcripts can contribute assistant/thinking/usage records.
    pub fn normalize_hook(&self, harness: &str, raw: Value) -> Result<Vec<CapturedEvent>> {
        let object = raw
            .as_object()
            .context("hook record must be a JSON object")?;
        let name = object
            .get("hook")
            .or_else(|| object.get("hook_event_name"))
            .and_then(Value::as_str)
            .context("hook record is missing hook name")?;
        let payload = object.get("raw").cloned().unwrap_or_else(|| raw.clone());
        if name == "PostToolUse" || name == "AfterTool" || name == "execute.after" {
            let failed = payload
                .get("tool_response")
                .or_else(|| payload.get("toolResponse"))
                .is_some_and(tool_failed);
            let mut event = CapturedEvent::from_raw(
                if failed {
                    EventVerb::ToolError
                } else {
                    EventVerb::ToolResult
                },
                payload,
            );
            event.tool = tool_from(object).or(event.tool);
            return Ok(vec![event]);
        }
        if name == "Notification" {
            let message = payload
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default();
            return Ok((message.contains("permission")
                || message.contains("approve")
                || message.contains("allow"))
            .then(|| CapturedEvent::from_raw(EventVerb::PermissionRequest, payload))
            .into_iter()
            .collect());
        }
        let Some(verb) = self
            .tables
            .get(harness)
            .and_then(|table| table.get(name))
            .copied()
        else {
            return Ok(Vec::new());
        };
        let mut event = CapturedEvent::from_raw(verb, payload);
        if verb == EventVerb::ToolCall {
            event.args = args_from(object).or(event.args);
        }
        Ok(vec![event])
    }
}

impl Adapter for HooksAdapter {
    fn normalize(&self, raw: Value) -> Result<Vec<CapturedEvent>> {
        let harness = raw
            .get("harness")
            .and_then(Value::as_str)
            .context("hook record is missing harness")?
            .to_owned();
        self.normalize_hook(&harness, raw)
    }
}

/// ACP has one protocol mapping shared by every ACP harness.
#[derive(Clone, Copy, Debug, Default)]
pub struct AcpAdapter;

impl Adapter for AcpAdapter {
    fn normalize(&self, raw: Value) -> Result<Vec<CapturedEvent>> {
        if raw.get("method").and_then(Value::as_str) == Some("session/request_permission") {
            return Ok(vec![CapturedEvent::from_raw(
                EventVerb::PermissionRequest,
                raw,
            )]);
        }
        let update = raw
            .pointer("/params/update")
            .or_else(|| raw.get("update"))
            .unwrap_or(&raw);
        let kind = update
            .get("sessionUpdate")
            .or_else(|| update.get("type"))
            .and_then(Value::as_str);
        let verb = match kind {
            Some("agent_message_chunk") | Some("AgentMessageChunk") => Some(EventVerb::Assistant),
            Some("agent_thought_chunk") | Some("AgentThoughtChunk") => Some(EventVerb::Thinking),
            Some("user_message_chunk") | Some("UserMessageChunk") => Some(EventVerb::User),
            Some("tool_call") | Some("ToolCall") => Some(EventVerb::ToolCall),
            Some("tool_call_update") | Some("ToolCallUpdate") => Some(
                if update.get("status").and_then(Value::as_str) == Some("failed") {
                    EventVerb::ToolError
                } else {
                    EventVerb::ToolResult
                },
            ),
            _ => None,
        };
        Ok(verb
            .map(|verb| CapturedEvent::from_raw(verb, raw))
            .into_iter()
            .collect())
    }
}

/// TOML-provided stream-json declaration: record type key and its value-to-verb table.
#[derive(Clone, Debug)]
pub struct StreamJsonAdapter {
    pub type_key: String,
    pub verbs: HashMap<String, EventVerb>,
}

impl StreamJsonAdapter {
    pub fn new(
        type_key: impl Into<String>,
        verbs: impl IntoIterator<Item = (impl Into<String>, EventVerb)>,
    ) -> Self {
        Self {
            type_key: type_key.into(),
            verbs: verbs
                .into_iter()
                .map(|(key, verb)| (key.into(), verb))
                .collect(),
        }
    }
}

impl Adapter for StreamJsonAdapter {
    fn normalize(&self, raw: Value) -> Result<Vec<CapturedEvent>> {
        let kind = raw
            .get(&self.type_key)
            .and_then(Value::as_str)
            .context("stream-json record is missing configured type key")?;
        Ok(self
            .verbs
            .get(kind)
            .copied()
            .map(|verb| CapturedEvent::from_raw(verb, raw))
            .into_iter()
            .collect())
    }
}

/// Session-log parser selected by harness. Parsers work in file position order.
#[derive(Clone, Debug, Default)]
pub struct SessionLogAdapter {
    parsers: HashMap<String, HashMap<String, EventVerb>>,
}

impl SessionLogAdapter {
    pub fn with_parser(
        harness: impl Into<String>,
        parser: impl IntoIterator<Item = (impl Into<String>, EventVerb)>,
    ) -> Self {
        let mut adapter = Self::default();
        adapter.parsers.insert(
            harness.into(),
            parser
                .into_iter()
                .map(|(kind, verb)| (kind.into(), verb))
                .collect(),
        );
        adapter
    }

    pub fn normalize_record(&self, harness: &str, raw: Value) -> Result<Vec<CapturedEvent>> {
        let kind = raw
            .get("kind")
            .or_else(|| raw.get("event"))
            .or_else(|| raw.get("type"))
            .and_then(Value::as_str)
            .context("session-log record is missing kind")?;
        Ok(self
            .parsers
            .get(harness)
            .and_then(|parser| parser.get(kind))
            .copied()
            .map(|verb| CapturedEvent::from_raw(verb, raw))
            .into_iter()
            .collect())
    }
}

impl Adapter for SessionLogAdapter {
    fn normalize(&self, raw: Value) -> Result<Vec<CapturedEvent>> {
        let harness = raw
            .get("harness")
            .and_then(Value::as_str)
            .context("session-log record is missing harness")?
            .to_owned();
        self.normalize_record(&harness, raw)
    }
}

/// Tracks a session log while a run is live, then re-reads it once on exit.
pub struct SessionLogTail {
    offset: usize,
}

impl SessionLogTail {
    pub fn new() -> Self {
        Self { offset: 0 }
    }

    pub fn read_new(&mut self, contents: &str) -> Vec<Value> {
        let lines: Vec<_> = contents.lines().collect();
        let records = lines[self.offset.min(lines.len())..]
            .iter()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        self.offset = lines.len();
        records
    }

    pub fn reread_at_exit(&mut self, contents: &str) -> Vec<Value> {
        self.read_new(contents)
    }
}

impl Default for SessionLogTail {
    fn default() -> Self {
        Self::new()
    }
}

/// Copies declared structured stdout byte-for-byte to terminal and normalizer sinks.
pub fn tee_stdout(
    mut input: impl Read,
    mut terminal: impl std::io::Write,
    mut normalizer: impl std::io::Write,
) -> std::io::Result<u64> {
    let mut total = 0;
    let mut buffer = [0; 8192];
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            return Ok(total);
        }
        terminal.write_all(&buffer[..read])?;
        normalizer.write_all(&buffer[..read])?;
        total += read as u64;
    }
}

/// Expectations derive from each harness declaration rather than a weakest-source baseline.
pub fn expected_verbs(declared: Option<&[String]>) -> BTreeSet<EventVerb> {
    declared
        .unwrap_or_default()
        .iter()
        .filter_map(|verb| {
            EventVerb::ALL
                .into_iter()
                .find(|candidate| candidate.as_str() == verb)
        })
        .collect()
}

/// Event-based test helper: assert the events a run produced without test-only instrumentation.
pub fn assert_events(events: &[Event], expected: &[EventVerb]) -> Result<()> {
    let actual: BTreeSet<_> = events.iter().map(|event| event.verb).collect();
    let expected: BTreeSet<_> = expected.iter().copied().collect();
    if actual == expected {
        Ok(())
    } else {
        bail!("event assertion failed: expected {expected:?}, got {actual:?}")
    }
}

fn timestamp_from(object: Option<&Map<String, Value>>) -> String {
    object
        .and_then(|object| object.get("ts").or_else(|| object.get("timestamp")))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(now_timestamp)
}

fn now_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("UTC timestamps always format as RFC3339")
}

fn text_from(object: &Map<String, Value>) -> Option<String> {
    object
        .get("text")
        .or_else(|| object.get("message"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn tool_from(object: &Map<String, Value>) -> Option<String> {
    object
        .get("tool")
        .or_else(|| object.get("tool_name"))
        .or_else(|| object.get("toolName"))
        .or_else(|| object.get("name"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn args_from(object: &Map<String, Value>) -> Option<Value> {
    object
        .get("args")
        .or_else(|| object.get("tool_input"))
        .or_else(|| object.get("toolInput"))
        .cloned()
}

fn tool_failed(value: &Value) -> bool {
    let object = value.as_object();
    object.and_then(|object| object.get("is_error").or_else(|| object.get("isError")))
        == Some(&Value::Bool(true))
        || object
            .and_then(|object| object.get("error"))
            .and_then(Value::as_str)
            .is_some_and(|error| !error.is_empty())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn assigned_timestamps_are_rfc3339() {
        assert!(OffsetDateTime::parse(&now_timestamp(), &Rfc3339).is_ok());
    }

    #[test]
    fn vocabulary_is_closed() {
        assert_eq!(EventVerb::ALL.len(), 12);
        assert_eq!(
            EventVerb::ALL
                .iter()
                .map(|verb| verb.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            12
        );
    }

    #[test]
    fn no_extension_at_runtime() {
        // This exhaustive match stops compiling if EventVerb gains a variant.
        let exhaustive = |verb: EventVerb| match verb {
            EventVerb::SessionStart
            | EventVerb::User
            | EventVerb::Assistant
            | EventVerb::Thinking
            | EventVerb::ToolCall
            | EventVerb::ToolResult
            | EventVerb::ToolError
            | EventVerb::PermissionRequest
            | EventVerb::SubagentStart
            | EventVerb::SubagentStop
            | EventVerb::Aborted
            | EventVerb::SessionEnd => verb,
        };
        assert_eq!(exhaustive(EventVerb::SessionEnd), EventVerb::SessionEnd);
    }

    #[test]
    fn seq_is_total() {
        let collector = EventCollector::new(4);
        let first = collector.capture("run-a", CapturedEvent::from_raw(EventVerb::User, json!({})));
        let second = collector.capture(
            "run-a",
            CapturedEvent::from_raw(EventVerb::Assistant, json!({})),
        );
        let other = collector.capture("run-b", CapturedEvent::from_raw(EventVerb::User, json!({})));
        assert_eq!([first.seq, second.seq, other.seq], [0, 1, 0]);
    }

    #[test]
    fn hooks_adapter() {
        let adapter = HooksAdapter::with_table(
            "claude",
            [
                ("SessionStart", EventVerb::SessionStart),
                ("PreToolUse", EventVerb::ToolCall),
            ],
        );
        assert_eq!(
            adapter
                .normalize(
                    json!({"harness":"claude","hook":"SessionStart","raw":{"timestamp":"t"}})
                )
                .unwrap()[0]
                .verb,
            EventVerb::SessionStart
        );
        let event = adapter.normalize(json!({"harness":"claude","hook":"PreToolUse","raw":{"tool_name":"bash","tool_input":{"command":"true"}}})).unwrap().pop().unwrap();
        assert_eq!(
            (event.verb, event.tool.as_deref(), event.args),
            (
                EventVerb::ToolCall,
                Some("bash"),
                Some(json!({"command":"true"}))
            )
        );
        assert!(adapter
            .normalize(json!({"harness":"claude","hook":"Stop","raw":{}}))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn acp_adapter() {
        let adapter = AcpAdapter;
        assert_eq!(
            adapter
                .normalize(json!({"params":{"update":{"sessionUpdate":"AgentMessageChunk"}}}))
                .unwrap()[0]
                .verb,
            EventVerb::Assistant
        );
        assert_eq!(adapter.normalize(json!({"params":{"update":{"sessionUpdate":"ToolCallUpdate","status":"failed"}}})).unwrap()[0].verb, EventVerb::ToolError);
        assert_eq!(
            adapter
                .normalize(json!({"method":"session/request_permission"}))
                .unwrap()[0]
                .verb,
            EventVerb::PermissionRequest
        );
    }

    #[test]
    fn stream_json_adapter() {
        let adapter = StreamJsonAdapter::new("record_type", [("reply", EventVerb::Assistant)]);
        assert_eq!(
            adapter
                .normalize(json!({"record_type":"reply","usage":{"input_tokens":3}}))
                .unwrap()[0]
                .usage
                .as_ref()
                .unwrap()
                .input,
            Some(3)
        );
        assert!(adapter
            .normalize(json!({"record_type":"other"}))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn tee_is_lossless() {
        let bytes = b"{\"type\":\"assistant\"}\n";
        let mut terminal = Vec::new();
        let mut normalizer = Vec::new();
        assert_eq!(
            tee_stdout(Cursor::new(bytes), &mut terminal, &mut normalizer).unwrap(),
            bytes.len() as u64
        );
        assert_eq!(terminal, bytes);
        assert_eq!(normalizer, bytes);
    }

    #[test]
    fn session_log_adapter() {
        let adapter = SessionLogAdapter::with_parser(
            "aider",
            [
                ("message", EventVerb::Assistant),
                ("end", EventVerb::SessionEnd),
            ],
        );
        assert_eq!(
            adapter
                .normalize(json!({"harness":"aider","kind":"message","usage":{"output":2}}))
                .unwrap()[0]
                .verb,
            EventVerb::Assistant
        );
    }

    #[test]
    fn session_log_reread() {
        let mut tail = SessionLogTail::new();
        assert_eq!(tail.read_new("{\"kind\":\"user\"}\n").len(), 1);
        assert_eq!(
            tail.reread_at_exit("{\"kind\":\"user\"}\n{\"kind\":\"end\"}\n")
                .len(),
            1
        );
    }

    #[test]
    fn raw_always_present() {
        let raw = json!({"type":"message","opaque":{"source":true}});
        let event = CapturedEvent::from_raw(EventVerb::Assistant, raw.clone());
        assert_eq!(event.raw, raw);
    }

    #[test]
    fn replay_repairs() {
        let first = StreamJsonAdapter::new("type", [("old", EventVerb::Assistant)]);
        let raw = json!({"type":"new"});
        assert!(replay(&first, [raw.clone()]).unwrap().is_empty());
        let repaired = StreamJsonAdapter::new("type", [("new", EventVerb::Assistant)]);
        assert_eq!(
            replay(&repaired, [raw]).unwrap()[0].verb,
            EventVerb::Assistant
        );
    }

    #[test]
    fn usage_unknown_not_zero() {
        assert_eq!(
            CapturedEvent::from_raw(EventVerb::Assistant, json!({})).usage,
            None
        );
        assert_eq!(
            CapturedEvent::from_raw(EventVerb::Assistant, json!({"usage":{"input":0}}))
                .usage
                .unwrap()
                .input,
            Some(0)
        );
    }

    #[test]
    fn never_counts_tokens() {
        let event =
            CapturedEvent::from_raw(EventVerb::Assistant, json!({"text":"four words here"}));
        assert_eq!(event.usage, None);
    }

    #[test]
    fn missing_verb_stays_missing() {
        let adapter = StreamJsonAdapter::new("type", [("assistant", EventVerb::Assistant)]);
        let events = replay(
            &adapter,
            [json!({"type":"assistant"}), json!({"type":"unknown"})],
        )
        .unwrap();
        assert_events(
            &events
                .into_iter()
                .enumerate()
                .map(|(seq, captured)| Event::from_captured("run".into(), seq as u64, captured))
                .collect::<Vec<_>>(),
            &[EventVerb::Assistant],
        )
        .unwrap();
    }

    #[test]
    fn expectations_per_harness() {
        let declared = vec!["assistant".into(), "tool_call".into()];
        assert_eq!(
            expected_verbs(Some(&declared)),
            BTreeSet::from([EventVerb::Assistant, EventVerb::ToolCall])
        );
        assert!(expected_verbs(None).is_empty());
    }

    #[test]
    fn permission_request_alarms() {
        let collector = EventCollector::new(4);
        let mut alarms = collector.subscribe_alarms();
        collector.capture(
            "run",
            CapturedEvent::from_raw(EventVerb::PermissionRequest, json!({})),
        );
        assert_eq!(
            alarms.try_recv().unwrap(),
            PermissionAlarm {
                run_id: "run".into(),
                seq: 0
            }
        );
    }

    #[test]
    fn sources_indistinguishable() {
        let hooks = HooksAdapter::with_table("claude", [("UserPromptSubmit", EventVerb::User)]);
        let stream = StreamJsonAdapter::new("type", [("input", EventVerb::User)]);
        let hook_event = hooks
            .normalize(json!({"harness":"claude","hook":"UserPromptSubmit","raw":{"text":"hello"}}))
            .unwrap()
            .pop()
            .unwrap();
        let stream_event = stream
            .normalize(json!({"type":"input","text":"hello"}))
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(hook_event.verb, stream_event.verb);
        assert_eq!(hook_event.text, stream_event.text);
    }

    #[test]
    fn event_assertions() {
        let collector = EventCollector::new(4);
        collector.capture(
            "run",
            CapturedEvent::from_raw(EventVerb::SessionStart, json!({})),
        );
        collector.capture(
            "run",
            CapturedEvent::from_raw(EventVerb::SessionEnd, json!({})),
        );
        assert_events(
            &collector.events_for("run"),
            &[EventVerb::SessionStart, EventVerb::SessionEnd],
        )
        .unwrap();
    }
}
