//! The ACP capture adapter and the closed event vocabulary every harness normalizes into.
//!
//! ACP is the only harness interface, so there is one adapter with one mapping shared by
//! every harness. `hooks`, `stream-json`, and `session-log` are retired — see PLAN.md §ACP.
//!
//! Adapters deliberately return unsequenced [`CapturedEvent`] values. [`EventCollector`]
//! is the sole place that assigns sequence numbers, so source ordering cannot leak into
//! downstream consumers.

use crate::bus::InProcessBus;
use crate::ids::RunId;
use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use anyhow::{bail, Result};
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub run_id: RunId,
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
    fn from_captured(run_id: RunId, seq: u64, captured: CapturedEvent) -> Self {
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
    pub run_id: RunId,
    pub seq: u64,
}

/// Owns per-run sequence assignment and publishes normalized events to local consumers.
///
/// The journal is a bounded debugging aid; the durable event store remains the source of truth.
const EVENT_JOURNAL_CAPACITY: usize = 1_024;

#[derive(Clone)]
pub struct EventCollector {
    next_seq: Arc<Mutex<HashMap<RunId, u64>>>,
    events: Arc<Mutex<HashMap<RunId, VecDeque<Event>>>>,
    events_out: InProcessBus<Event>,
    alarms: InProcessBus<PermissionAlarm>,
}

impl EventCollector {
    pub fn new(capacity: usize) -> Self {
        Self {
            next_seq: Arc::new(Mutex::new(HashMap::new())),
            events: Arc::new(Mutex::new(HashMap::new())),
            events_out: InProcessBus::new(capacity),
            alarms: InProcessBus::new(capacity),
        }
    }

    pub fn capture(&self, run_id: RunId, captured: CapturedEvent) -> Event {
        let seq = {
            let mut sequences = self.next_seq.lock().expect("event sequence lock");
            let next = sequences.entry(run_id).or_insert(0);
            let assigned = *next;
            *next += 1;
            assigned
        };
        let event = Event::from_captured(run_id, seq, captured);
        let mut journals = self.events.lock().expect("event store lock");
        let journal = journals.entry(run_id).or_default();
        if journal.len() == EVENT_JOURNAL_CAPACITY {
            journal.pop_front();
        }
        journal.push_back(event.clone());
        drop(journals);
        self.events_out.publish(event.clone());
        if event.verb == EventVerb::PermissionRequest {
            self.alarms.publish(PermissionAlarm {
                run_id: event.run_id,
                seq: event.seq,
            });
        }
        event
    }

    pub fn events_for(&self, run_id: RunId) -> Vec<Event> {
        self.events
            .lock()
            .expect("event store lock")
            .get(&run_id)
            .map(|journal| journal.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.events_out.subscribe()
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
            .cloned()
            .unwrap_or_else(|| raw.clone());
        let kind = update
            .get("sessionUpdate")
            .or_else(|| update.get("type"))
            .and_then(Value::as_str);
        let verb = match kind {
            Some("agent_message_chunk") | Some("AgentMessageChunk") => Some(EventVerb::Assistant),
            Some("agent_thought_chunk") | Some("AgentThoughtChunk") => Some(EventVerb::Thinking),
            Some("user_message_chunk") | Some("UserMessageChunk") => Some(EventVerb::User),
            Some("tool_call") | Some("ToolCall") => Some(EventVerb::ToolCall),
            Some("tool_call_update") | Some("ToolCallUpdate") => {
                match update.get("status").and_then(Value::as_str) {
                    Some("completed") => Some(EventVerb::ToolResult),
                    Some("failed") => Some(EventVerb::ToolError),
                    _ => None,
                }
            }
            _ => None,
        };
        Ok(verb
            .map(|verb| {
                let mut event = CapturedEvent::from_raw(verb, raw);
                let update = update.as_object();
                event.text = event.text.or_else(|| {
                    update.and_then(text_from).or_else(|| {
                        update
                            .and_then(|update| update.get("content"))
                            .and_then(Value::as_object)
                            .and_then(text_from)
                    })
                });
                event.tool = event.tool.or_else(|| {
                    update.and_then(tool_from).or_else(|| {
                        update
                            .and_then(|update| update.get("title"))
                            .and_then(Value::as_str)
                            .map(str::to_owned)
                    })
                });
                event.args = event.args.or_else(|| {
                    update
                        .and_then(args_from)
                        .or_else(|| update.and_then(|update| update.get("rawInput")).cloned())
                });
                event
            })
            .into_iter()
            .collect())
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

#[cfg(test)]
mod tests {
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
        // Sequence is per run: two events on one run, then a fresh run restarts at 0.
        let run_a = RunId::generate();
        let run_b = RunId::generate();
        let first = collector.capture(run_a, CapturedEvent::from_raw(EventVerb::User, json!({})));
        let second = collector.capture(
            run_a,
            CapturedEvent::from_raw(EventVerb::Assistant, json!({})),
        );
        let other = collector.capture(run_b, CapturedEvent::from_raw(EventVerb::User, json!({})));
        assert_eq!([first.seq, second.seq, other.seq], [0, 1, 0]);
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
    fn raw_always_present() {
        let raw = json!({"type":"message","opaque":{"source":true}});
        let event = CapturedEvent::from_raw(EventVerb::Assistant, raw.clone());
        assert_eq!(event.raw, raw);
    }

    /// A swappable mapping, so `replay` can be exercised against a parser that changed.
    /// Production has exactly one adapter; this stub exists so that stays true.
    struct TableAdapter(&'static str, EventVerb);

    impl Adapter for TableAdapter {
        fn normalize(&self, record: Value) -> Result<Vec<CapturedEvent>> {
            if record.get("type").and_then(Value::as_str) == Some(self.0) {
                return Ok(vec![CapturedEvent::from_raw(self.1, record)]);
            }
            Ok(Vec::new())
        }
    }

    #[test]
    fn replay_repairs() {
        let raw = json!({"type":"new"});
        let stale = TableAdapter("old", EventVerb::Assistant);
        assert!(replay(&stale, [raw.clone()]).unwrap().is_empty());

        let repaired = TableAdapter("new", EventVerb::Assistant);
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
        // An update kind the vocabulary does not cover produces no event, rather than an
        // empty one. A missing verb is recorded as missing.
        let events = replay(
            &AcpAdapter,
            [
                json!({"method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"spoken"}}}}),
                json!({"method":"session/update","params":{"update":{"sessionUpdate":"unknown_kind"}}}),
            ],
        )
        .unwrap();
        assert_events(
            &events
                .into_iter()
                .enumerate()
                .map(|(seq, captured)| {
                    Event::from_captured(RunId::generate(), seq as u64, captured)
                })
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
        let run = RunId::generate();
        collector.capture(
            run,
            CapturedEvent::from_raw(EventVerb::PermissionRequest, json!({})),
        );
        // The alarm carries the run it came from, not just the fact that one fired.
        assert_eq!(
            alarms.try_recv().unwrap(),
            PermissionAlarm {
                run_id: run,
                seq: 0
            }
        );
    }

    #[test]
    fn event_assertions() {
        let collector = EventCollector::new(4);
        let run = RunId::generate();
        collector.capture(
            run,
            CapturedEvent::from_raw(EventVerb::SessionStart, json!({})),
        );
        collector.capture(
            run,
            CapturedEvent::from_raw(EventVerb::SessionEnd, json!({})),
        );
        assert_events(
            &collector.events_for(run),
            &[EventVerb::SessionStart, EventVerb::SessionEnd],
        )
        .unwrap();
    }
}
