//! Deterministic tool-boundary compaction.
//!
//! This module contains the policy shared by the in-container hook and the core.  It never
//! invokes a model and does not perform I/O while transforming a tool call or result.

use crate::{
    ids::{ProjectId, RunId},
    services::artifact::{ArtifactKind, ArtifactRow, ArtifactStore},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The one threshold used by tool compaction and payload artifacts.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompactionSettings {
    pub threshold: usize,
}

impl Default for CompactionSettings {
    fn default() -> Self {
        Self {
            threshold: crate::services::artifact::DEFAULT_COMPACTION_THRESHOLD,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandRewrite {
    pub original: String,
    pub rewritten: String,
}

impl CommandRewrite {
    pub fn changed(&self) -> bool {
        self.original != self.rewritten
    }
}

/// Rewrite only complete, known command prefixes.  Unknown commands and already compact
/// commands pass through byte-for-byte; this is intentional because changing shell semantics
/// costs more than the bytes saved.
pub fn rewrite_command(command: &str) -> String {
    const RULES: &[(&str, &str)] = &[
        ("git status", "git status --short"),
        ("git diff", "git diff --stat"),
        ("git log", "git log --oneline --decorate -n 20"),
        ("find .", "rg --files ."),
        ("tree", "rg --files"),
        ("ls", "ls -1"),
    ];
    for (prefix, compact) in RULES {
        if command == *prefix {
            return (*compact).into();
        }
        if let Some(rest) = command.strip_prefix(prefix) {
            if rest.starts_with(char::is_whitespace)
                && !command.contains("--short")
                && !(prefix == &"git diff" && command.contains("--stat"))
                && !(prefix == &"git log" && command.contains("--oneline"))
            {
                return format!("{compact}{rest}");
            }
        }
    }
    command.into()
}

pub fn rewrite(command: impl Into<String>) -> CommandRewrite {
    let original = command.into();
    let rewritten = rewrite_command(&original);
    CommandRewrite {
        original,
        rewritten,
    }
}

/// A normalized tool call carries the rewrite in its event payload, making the optimization
/// observable without adding a new event verb.
pub fn rewrite_event_payload(payload: &mut Value) -> Option<CommandRewrite> {
    let command_path = [
        "/command",
        "/tool_input/command",
        "/toolInput/command",
        "/input/command",
    ];
    for pointer in command_path {
        let Some(command) = payload.pointer(pointer).and_then(Value::as_str) else {
            continue;
        };
        let result = rewrite(command);
        if !result.changed() {
            return None;
        }
        let key = pointer.rsplit('/').next().expect("command pointer");
        if let Some(parent) = payload.pointer_mut(&pointer[..pointer.rfind('/').unwrap_or(0)]) {
            if let Some(object) = parent.as_object_mut() {
                object.insert(key.into(), Value::String(result.rewritten.clone()));
            }
        }
        if let Some(object) = payload.as_object_mut() {
            object.insert(
                "original_command".into(),
                Value::String(result.original.clone()),
            );
            object.insert(
                "rewritten_command".into(),
                Value::String(result.rewritten.clone()),
            );
        }
        return Some(result);
    }
    None
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompactedResult {
    pub body: String,
    pub original_bytes: usize,
    pub compacted_bytes: usize,
    pub saving_ratio: f64,
    pub artifact_id: Option<crate::ids::ArtifactId>,
}

impl CompactedResult {
    pub fn saving_ratio(&self) -> f64 {
        self.saving_ratio
    }
}

/// Compact a result and retain an over-threshold body as a payload artifact.
pub fn compact_result(
    store: &mut ArtifactStore,
    project_id: ProjectId,
    run_id: RunId,
    body: String,
    settings: CompactionSettings,
) -> CompactedResult {
    compact_result_with_enabled(store, project_id, run_id, body, settings, true)
}

/// The disabled path intentionally returns the exact original body and creates no artifact.
/// Callers can compare the resulting event stream with the enabled path; only cost metadata
/// differs.
pub fn compact_result_with_enabled(
    store: &mut ArtifactStore,
    project_id: ProjectId,
    run_id: RunId,
    body: String,
    settings: CompactionSettings,
    enabled: bool,
) -> CompactedResult {
    let original_bytes = body.len();
    if !enabled {
        return CompactedResult {
            compacted_bytes: original_bytes,
            body,
            original_bytes,
            saving_ratio: 0.0,
            artifact_id: None,
        };
    }
    if original_bytes <= settings.threshold {
        return CompactedResult {
            compacted_bytes: original_bytes,
            body,
            original_bytes,
            saving_ratio: 0.0,
            artifact_id: None,
        };
    }

    let mut row = ArtifactRow::text(project_id, run_id, ArtifactKind::Payload, body);
    let artifact_id = row.id;
    row.summary = Some(format!(
        "Tool result compacted; artifact {} ({} bytes)",
        artifact_id, original_bytes
    ));
    let summary = row.summary.clone().expect("summary is set");
    let compacted_bytes = summary.len();
    store.put(row);
    CompactedResult {
        body: summary,
        original_bytes,
        compacted_bytes,
        saving_ratio: 1.0 - compacted_bytes as f64 / original_bytes as f64,
        artifact_id: Some(artifact_id),
    }
}

/// Compact one normalized event while preserving its verb and all event metadata.
pub fn compact_event(
    store: &mut ArtifactStore,
    project_id: ProjectId,
    event: &mut crate::services::telemetry::CapturedEvent,
    run_id: RunId,
    settings: CompactionSettings,
) -> Option<CompactedResult> {
    if event.verb != crate::services::telemetry::EventVerb::ToolResult {
        return None;
    }
    let text = event.text.clone()?;
    let result = compact_result(store, project_id, run_id, text, settings);
    event.text = Some(result.body.clone());
    if let Some(raw) = event.raw.as_object_mut() {
        raw.insert("original_bytes".into(), json!(result.original_bytes));
        raw.insert("compacted_bytes".into(), json!(result.compacted_bytes));
        raw.insert("saving_ratio".into(), json!(result.saving_ratio));
        if let Some(id) = result.artifact_id {
            raw.insert("artifact_id".into(), json!(id));
        }
    }
    Some(result)
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ToolResultSample {
    pub project_id: ProjectId,
    pub agent: String,
    pub harness: String,
    pub tool: String,
    pub payload_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolOffender {
    pub project_id: ProjectId,
    pub agent: String,
    pub harness: String,
    pub tool: String,
    pub payload_bytes: usize,
    pub calls: usize,
}

/// The in-memory equivalent of the durable `tool_result GROUP BY` projection.  BTreeMap and
/// explicit tie-breaks keep dashboard output stable when payload totals are equal.
pub fn offender_ranking(rows: impl IntoIterator<Item = ToolResultSample>) -> Vec<ToolOffender> {
    let mut grouped = BTreeMap::<(ProjectId, String, String, String), ToolOffender>::new();
    for row in rows {
        let key = (
            row.project_id,
            row.agent.clone(),
            row.harness.clone(),
            row.tool.clone(),
        );
        let entry = grouped.entry(key).or_insert_with(|| ToolOffender {
            project_id: row.project_id,
            agent: row.agent,
            harness: row.harness,
            tool: row.tool,
            payload_bytes: 0,
            calls: 0,
        });
        entry.payload_bytes += row.payload_bytes;
        entry.calls += 1;
    }
    let mut result = grouped.into_values().collect::<Vec<_>>();
    result.sort_by(|left, right| {
        right
            .payload_bytes
            .cmp(&left.payload_bytes)
            .then_with(|| left.project_id.cmp(&right.project_id))
            .then_with(|| left.agent.cmp(&right.agent))
            .then_with(|| left.harness.cmp(&right.harness))
            .then_with(|| left.tool.cmp(&right.tool))
    });
    result
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod compact {
    use super::*;
    use crate::ids::{ProjectId, RunId};

    fn ids() -> (ProjectId, RunId) {
        (ProjectId::generate(), RunId::generate())
    }

    #[test]
    fn rewrites() {
        assert_eq!(rewrite_command("git status"), "git status --short");
        assert_eq!(rewrite_command("git status --short"), "git status --short");
        assert_eq!(rewrite_command("unknown --flag"), "unknown --flag");
    }

    #[test]
    fn rewrite_observable() {
        let mut payload = json!({"tool_input":{"command":"git status"}});
        let result = rewrite_event_payload(&mut payload).expect("rewrite");
        assert_eq!(result.rewritten, "git status --short");
        assert_eq!(payload["rewritten_command"], "git status --short");
    }

    #[test]
    fn compacts_result() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        let result = compact_result(
            &mut store,
            project,
            run,
            "x".repeat(10_000),
            CompactionSettings { threshold: 2 },
        );
        assert!(result.saving_ratio() > 0.5);
        assert_eq!(store.review_inbox().len(), 0);
    }

    #[test]
    fn overflow_to_artifact() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        let result = compact_result(
            &mut store,
            project,
            run,
            "body".into(),
            CompactionSettings { threshold: 2 },
        );
        let id = result.artifact_id.expect("payload id");
        assert_eq!(store.get(id).expect("payload").kind, ArtifactKind::Payload);
    }

    #[test]
    fn summary_with_handle() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        let result = compact_result(
            &mut store,
            project,
            run,
            "body".into(),
            CompactionSettings { threshold: 2 },
        );
        assert!(result.body.contains("artifact"));
        assert!(result
            .body
            .contains(&result.artifact_id.unwrap().to_string()));
    }

    #[test]
    fn single_threshold_setting() {
        assert_eq!(
            CompactionSettings::default().threshold,
            crate::services::artifact::DEFAULT_COMPACTION_THRESHOLD
        );
    }

    #[test]
    fn behavior_unchanged_when_below_threshold() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        let result = compact_result(
            &mut store,
            project,
            run,
            "same".into(),
            CompactionSettings { threshold: 99 },
        );
        assert_eq!(result.body, "same");
        assert_eq!(result.original_bytes, result.compacted_bytes);
        assert!(result.artifact_id.is_none());
    }

    #[test]
    fn ranking_dimensions() {
        let project = ProjectId::generate();
        let rows = super::offender_ranking([
            ToolResultSample {
                project_id: project,
                agent: "a".into(),
                harness: "h".into(),
                tool: "cat".into(),
                payload_bytes: 4,
            },
            ToolResultSample {
                project_id: project,
                agent: "a".into(),
                harness: "h".into(),
                tool: "cat".into(),
                payload_bytes: 6,
            },
        ]);
        assert_eq!(rows[0].payload_bytes, 10);
        assert_eq!(rows[0].calls, 2);
    }

    #[test]
    fn offender_ranking() {
        let project = ProjectId::generate();
        let rows = super::offender_ranking([ToolResultSample {
            project_id: project,
            agent: "agent".into(),
            harness: "harness".into(),
            tool: "rg".into(),
            payload_bytes: 8,
        }]);
        assert_eq!(rows[0].tool, "rg");
    }

    #[test]
    fn degrades_not_fails() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        let result = compact_result_with_enabled(
            &mut store,
            project,
            run,
            "original".into(),
            CompactionSettings { threshold: 1 },
            false,
        );
        assert_eq!(result.body, "original");
        assert!(result.artifact_id.is_none());
    }

    #[test]
    fn behavior_unchanged() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        let result = compact_result_with_enabled(
            &mut store,
            project,
            run,
            "original".into(),
            CompactionSettings { threshold: 1 },
            false,
        );
        assert_eq!(result.body, "original");
        assert_eq!(result.original_bytes, result.compacted_bytes);
    }
}
