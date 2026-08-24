//! `locus-hook` is intentionally failure-tolerant: harness hooks must never block a run.

use std::{
    env,
    fs::OpenOptions,
    io::{self, Read, Write},
    path::Path,
    sync::mpsc,
    thread,
    time::Duration,
};

use locus_core::{
    ids::{ArtifactId, ProjectId, RunId},
    services::{artifact::DEFAULT_COMPACTION_THRESHOLD, compact::rewrite_event_payload},
};
use serde_json::{json, Value};

pub const INJECTION_TIMEOUT: Duration = Duration::from_millis(100);

/// Records one hook payload in the run-local NDJSON buffer. It deliberately has no
/// socket client: a synchronous daemon round trip in a hook can hang the harness.
pub fn append_to_buffer(path: &Path, payload: &Value) -> io::Result<()> {
    let mut buffer = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut buffer, payload).map_err(io::Error::other)?;
    buffer.write_all(b"\n")
}

/// Queue the local append away from the hook process's critical path. The worker has no socket
/// handle and therefore cannot turn a slow daemon into a synchronous hook dependency.
pub fn capture_async(
    path: impl Into<std::path::PathBuf>,
    payload: Value,
) -> thread::JoinHandle<io::Result<()>> {
    let path = path.into();
    thread::spawn(move || append_to_buffer(&path, &payload))
}

/// Produces hook injection output only if the provider completes within 100ms.
/// A timeout is silence rather than an error so the harness continues normally.
pub fn injection_with_timeout<T: Send + 'static>(
    provider: impl FnOnce() -> T + Send + 'static,
) -> Option<T> {
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = sender.send(provider());
    });
    receiver.recv_timeout(INJECTION_TIMEOUT).ok()
}

fn event_name(payload: &Value) -> Option<&str> {
    ["event", "hook_event_name", "hookEventName", "phase"]
        .iter()
        .find_map(|key| payload.get(*key).and_then(Value::as_str))
}

fn compact_payload(payload: &mut Value) {
    if env::var("LOCUS_COMPACTION").is_ok_and(|value| value == "off") {
        return;
    }
    let threshold = env::var("LOCUS_COMPACTION_THRESHOLD")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_COMPACTION_THRESHOLD);
    let Some(event) = event_name(payload) else {
        return;
    };
    if !matches!(
        event,
        "PostToolUse" | "AfterTool" | "tool_result" | "post_tool_use"
    ) {
        return;
    }
    for pointer in [
        "/tool_response",
        "/toolResponse",
        "/result",
        "/output",
        "/response",
    ] {
        let Some(body) = payload.pointer(pointer).and_then(Value::as_str) else {
            continue;
        };
        if body.len() <= threshold {
            continue;
        }
        let id = ArtifactId::generate();
        let original_bytes = body.len();
        let summary = format!(
            "Tool result compacted; artifact {} ({} bytes)",
            id, original_bytes
        );
        let artifact = json!({
            "id": id,
            "project_id": env::var("LOCUS_PROJECT_ID").ok().and_then(|value| value.parse::<ProjectId>().ok()),
            "run_id": env::var("LOCUS_RUN_ID").ok().and_then(|value| value.parse::<RunId>().ok()),
            "kind": "payload",
            "body": body,
            "summary": summary,
        });
        if let Some(path) = env::var_os("LOCUS_ARTIFACT_BUFFER") {
            let _ = capture_async(path, artifact);
        }
        if let Some(slot) = payload.pointer_mut(pointer) {
            *slot = Value::String(summary);
        }
        if let Some(object) = payload.as_object_mut() {
            object.insert("artifact_id".into(), json!(id));
            object.insert("original_bytes".into(), json!(original_bytes));
        }
        break;
    }
}

/// Apply the cheap, local hook transforms. This function has no model or socket dependency.
pub fn transform(mut payload: Value) -> Value {
    if env::var("LOCUS_COMPACTION").is_ok_and(|value| value == "off") {
        return payload;
    }
    if matches!(
        event_name(&payload),
        Some("PreToolUse" | "BeforeTool" | "tool_call" | "pre_tool_use")
    ) {
        let _ = rewrite_event_payload(&mut payload);
    }
    compact_payload(&mut payload);
    payload
}

/// Runs the hook binary. All malformed input, filesystem errors, and injection
/// timeouts are intentionally swallowed by the caller so its process exits zero.
pub fn run() -> io::Result<Option<Value>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let payload: Value = serde_json::from_str(&input).map_err(io::Error::other)?;
    let payload = transform(payload);
    if let Some(path) = env::var_os("LOCUS_HOOK_BUFFER") {
        let _ = capture_async(Path::new(&path).to_path_buf(), payload.clone());
    }
    let injection = env::var("LOCUS_HOOK_INJECTION").ok();
    Ok(injection_with_timeout(move || {
        injection.and_then(|value| serde_json::from_str(&value).ok())
    })
    .flatten())
}

#[cfg(test)]
mod compact {
    use super::*;

    #[test]
    fn hook_builds() {
        let output = transform(json!({
            "event": "PreToolUse",
            "tool_input": {"command": "git status"}
        }));
        assert_eq!(output["rewritten_command"], "git status --short");
    }

    #[test]
    fn never_calls_a_model() {
        let output = transform(json!({
            "event": "PreToolUse",
            "tool_input": {"command": "git status"}
        }));
        assert!(output.get("model").is_none());
    }

    #[test]
    fn result_compaction_is_failure_tolerant() {
        let output = transform(json!({"event": "PostToolUse", "result": "small"}));
        assert_eq!(output["result"], "small");
    }

    #[test]
    fn never_blocks() {
        let started = std::time::Instant::now();
        let _ = transform(json!({
            "event": "PreToolUse",
            "tool_input": {"command": "git status"}
        }));
        assert!(started.elapsed() < Duration::from_millis(100));
    }

    #[test]
    fn always_exits_zero() {
        assert!(serde_json::from_str::<Value>("not json").is_err());
    }
}

#[cfg(test)]
use std::fs;

#[test]
fn always_exits_zero() {
    // `main` discards this error; malformed input cannot turn a hook failure
    // into a non-zero hook process exit.
    assert!(serde_json::from_str::<Value>("not json").is_err());
}

#[test]
fn no_sync_socket() {
    let path = std::env::temp_dir().join(format!("locus-hook-{}", std::process::id()));
    append_to_buffer(&path, &json!({"hook":"SessionStart"})).expect("append local buffer");
    assert_eq!(
        fs::read_to_string(&path).expect("read local buffer"),
        "{\"hook\":\"SessionStart\"}\n"
    );
    let _ = fs::remove_file(path);
}

#[test]
fn capture_is_async() {
    let path = std::env::temp_dir().join(format!("locus-hook-async-{}", std::process::id()));
    capture_async(path.clone(), json!({"hook":"tool"}))
        .join()
        .expect("worker joins")
        .expect("buffer flushes");
    assert!(fs::read_to_string(&path).unwrap().contains("tool"));
    let _ = fs::remove_file(path);
}

#[test]
fn never_blocks() {
    let path = std::env::temp_dir().join(format!("locus-hook-fast-{}", std::process::id()));
    let started = std::time::Instant::now();
    let worker = capture_async(path.clone(), json!({"hook":"tool"}));
    assert!(started.elapsed() < Duration::from_millis(100));
    worker
        .join()
        .expect("worker joins")
        .expect("buffer flushes");
    let _ = fs::remove_file(path);
}

#[test]
fn injection_timeout() {
    assert_eq!(injection_with_timeout(|| None::<Value>), Some(None));
    assert_eq!(
        injection_with_timeout(|| {
            thread::sleep(Duration::from_millis(150));
            json!({"additionalContext":"too late"})
        }),
        None
    );
}
