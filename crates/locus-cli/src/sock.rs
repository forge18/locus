use std::path::Path;

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map, Value};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::UnixStream,
};

pub const DEFAULT_SOCKET_PATH: &str = "/run/locus.sock";
const KEY_PACK_ROW_THRESHOLD: usize = 1;

#[derive(Debug, Serialize)]
pub struct SocketRequest<'a> {
    pub verb: &'a str,
    pub args: &'a [String],
}

#[derive(Debug)]
pub struct VerbDispatch {
    pub command: &'static [&'static str],
    pub verb: &'static str,
}

pub const VERB_DISPATCHES: &[VerbDispatch] = &[
    VerbDispatch {
        command: &["memory", "note", "add"],
        verb: "memory.note.add",
    },
    VerbDispatch {
        command: &["memory", "note", "replace"],
        verb: "memory.note.replace",
    },
    VerbDispatch {
        command: &["memory", "note", "remove"],
        verb: "memory.note.remove",
    },
    VerbDispatch {
        command: &["memory", "recall"],
        verb: "memory.recall",
    },
    VerbDispatch {
        command: &["memory", "write"],
        verb: "memory.write",
    },
    VerbDispatch {
        command: &["memory", "forget"],
        verb: "memory.forget",
    },
    VerbDispatch {
        command: &["mail", "send"],
        verb: "mail.send",
    },
    VerbDispatch {
        command: &["mail", "list"],
        verb: "mail.list",
    },
    VerbDispatch {
        command: &["mail", "read"],
        verb: "mail.read",
    },
    VerbDispatch {
        command: &["mail", "reply"],
        verb: "mail.reply",
    },
    VerbDispatch {
        command: &["mail", "drain"],
        verb: "mail.drain",
    },
    VerbDispatch {
        command: &["mail", "wait"],
        verb: "mail.wait",
    },
    VerbDispatch {
        command: &["task", "list"],
        verb: "task.list",
    },
    VerbDispatch {
        command: &["task", "show"],
        verb: "task.show",
    },
    VerbDispatch {
        command: &["task", "move"],
        verb: "task.move",
    },
    VerbDispatch {
        command: &["task", "assign"],
        verb: "task.assign",
    },
    VerbDispatch {
        command: &["task", "comment"],
        verb: "task.comment",
    },
    VerbDispatch {
        command: &["wiki", "search"],
        verb: "wiki.search",
    },
    VerbDispatch {
        command: &["wiki", "read"],
        verb: "wiki.read",
    },
    VerbDispatch {
        command: &["wiki", "write"],
        verb: "wiki.write",
    },
    VerbDispatch {
        command: &["wiki", "history"],
        verb: "wiki.history",
    },
    VerbDispatch {
        command: &["wiki", "ingest"],
        verb: "wiki.ingest",
    },
    VerbDispatch {
        command: &["wiki", "query"],
        verb: "wiki.query",
    },
    VerbDispatch {
        command: &["wiki", "lint"],
        verb: "wiki.lint",
    },
    VerbDispatch {
        command: &["lsp", "def"],
        verb: "lsp.def",
    },
    VerbDispatch {
        command: &["lsp", "refs"],
        verb: "lsp.refs",
    },
    VerbDispatch {
        command: &["lsp", "hover"],
        verb: "lsp.hover",
    },
    VerbDispatch {
        command: &["lsp", "symbols"],
        verb: "lsp.symbols",
    },
    VerbDispatch {
        command: &["lsp", "rename"],
        verb: "lsp.rename",
    },
    VerbDispatch {
        command: &["debug", "start"],
        verb: "debug.start",
    },
    VerbDispatch {
        command: &["debug", "break"],
        verb: "debug.break",
    },
    VerbDispatch {
        command: &["debug", "step"],
        verb: "debug.step",
    },
    VerbDispatch {
        command: &["debug", "stack"],
        verb: "debug.stack",
    },
    VerbDispatch {
        command: &["debug", "vars"],
        verb: "debug.vars",
    },
    VerbDispatch {
        command: &["debug", "eval"],
        verb: "debug.eval",
    },
    VerbDispatch {
        command: &["browse", "open"],
        verb: "browse.open",
    },
    VerbDispatch {
        command: &["browse", "click"],
        verb: "browse.click",
    },
    VerbDispatch {
        command: &["browse", "fill"],
        verb: "browse.fill",
    },
    VerbDispatch {
        command: &["browse", "assert"],
        verb: "browse.assert",
    },
    VerbDispatch {
        command: &["browse", "screenshot"],
        verb: "browse.screenshot",
    },
    VerbDispatch {
        command: &["browse", "record"],
        verb: "browse.record",
    },
    VerbDispatch {
        command: &["browse", "console"],
        verb: "browse.console",
    },
    VerbDispatch {
        command: &["browse", "network"],
        verb: "browse.network",
    },
    VerbDispatch {
        command: &["agent", "invoke"],
        verb: "agent.invoke",
    },
    VerbDispatch {
        command: &["svc", "up"],
        verb: "svc.up",
    },
    VerbDispatch {
        command: &["svc", "down"],
        verb: "svc.down",
    },
    VerbDispatch {
        command: &["ask"],
        verb: "ask",
    },
    VerbDispatch {
        command: &["run", "status"],
        verb: "run.status",
    },
    VerbDispatch {
        command: &["run", "artifacts"],
        verb: "run.artifacts",
    },
    VerbDispatch {
        command: &["handoff"],
        verb: "handoff",
    },
    VerbDispatch {
        command: &["artifact", "put"],
        verb: "artifact.put",
    },
    VerbDispatch {
        command: &["artifact", "get"],
        verb: "artifact.get",
    },
    VerbDispatch {
        command: &["artifact", "comments"],
        verb: "artifact.comments",
    },
    VerbDispatch {
        command: &["tools", "list"],
        verb: "tools.list",
    },
    VerbDispatch {
        command: &["tools", "docs"],
        verb: "tools.docs",
    },
    VerbDispatch {
        command: &["lint"],
        verb: "lint",
    },
];

pub fn without_json_flag(arguments: &[String]) -> Vec<String> {
    arguments
        .iter()
        .filter(|argument| argument.as_str() != "--json")
        .cloned()
        .collect()
}

pub fn compact_json(response: &Value) -> serde_json::Result<String> {
    serde_json::to_string(response)
}

pub(crate) fn key_pack(value: Value) -> Value {
    match value {
        Value::Array(rows) => pack_uniform_table(&rows)
            .unwrap_or_else(|| Value::Array(rows.into_iter().map(key_pack).collect())),
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| (key, key_pack(value)))
                .collect(),
        ),
        value => value,
    }
}

fn pack_uniform_table(rows: &[Value]) -> Option<Value> {
    if rows.len() < KEY_PACK_ROW_THRESHOLD {
        return None;
    }

    let keys: Vec<_> = rows.first()?.as_object()?.keys().cloned().collect();
    if keys.is_empty()
        || !rows.iter().all(|row| {
            row.as_object().is_some_and(|object| {
                object.len() == keys.len() && keys.iter().all(|key| object.contains_key(key))
            })
        })
    {
        return None;
    }

    let rows = rows
        .iter()
        .map(|row| Value::Array(keys.iter().map(|key| key_pack(row[key].clone())).collect()))
        .collect();
    let mut packed = Map::new();
    packed.insert(
        "keys".to_owned(),
        Value::Array(keys.into_iter().map(Value::String).collect()),
    );
    packed.insert("rows".to_owned(), Value::Array(rows));
    Some(Value::Object(packed))
}

pub fn resolve_verb(arguments: &[String]) -> Option<(&'static VerbDispatch, &[String])> {
    VERB_DISPATCHES
        .iter()
        .find(|dispatch| {
            arguments
                .iter()
                .map(String::as_str)
                .zip(dispatch.command)
                .all(|(argument, command)| argument == *command)
                && arguments.len() >= dispatch.command.len()
        })
        .map(|dispatch| (dispatch, &arguments[dispatch.command.len()..]))
}

/// Refuse commands outside the CLI's declared agent-facing verb table before opening a socket.
pub fn allowed_verb(arguments: &[String]) -> Result<(&'static VerbDispatch, &[String])> {
    resolve_verb(arguments).ok_or_else(|| {
        anyhow::anyhow!(
            "command is not allowlisted: {}",
            arguments.join(" ")
        )
    })
}

pub async fn dispatch(
    socket_path: impl AsRef<Path>,
    dispatch: &VerbDispatch,
    args: &[String],
) -> Result<Value> {
    SocketClient::round_trip(
        socket_path,
        &SocketRequest {
            verb: dispatch.verb,
            args,
        },
    )
    .await
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SocketClient;

impl SocketClient {
    pub async fn round_trip<Request, Response>(
        socket_path: impl AsRef<Path>,
        request: &Request,
    ) -> Result<Response>
    where
        Request: Serialize,
        Response: DeserializeOwned,
    {
        let socket_path = socket_path.as_ref();
        let mut stream = UnixStream::connect(socket_path)
            .await
            .with_context(|| format!("connect to daemon socket `{}`", socket_path.display()))?;
        write_frame(&mut stream, request).await?;
        read_frame(&mut stream).await
    }
}

async fn write_frame<Writer, Message>(writer: &mut Writer, message: &Message) -> Result<()>
where
    Writer: AsyncWrite + Unpin,
    Message: Serialize,
{
    let payload = serde_json::to_vec(message).context("serialize socket request")?;
    let length = u32::try_from(payload.len()).context("socket frame exceeds u32 length")?;
    writer
        .write_u32(length)
        .await
        .context("write socket frame length")?;
    writer
        .write_all(&payload)
        .await
        .context("write socket frame body")?;
    writer.flush().await.context("flush socket frame")
}

async fn read_frame<Reader, Message>(reader: &mut Reader) -> Result<Message>
where
    Reader: AsyncRead + Unpin,
    Message: DeserializeOwned,
{
    let length = reader
        .read_u32()
        .await
        .context("read socket frame length")?;
    let mut payload = vec![0; length as usize];
    reader
        .read_exact(&mut payload)
        .await
        .context("read socket frame body")?;
    serde_json::from_slice(&payload).context("deserialize socket response")
}

#[cfg(test)]
use serde_json::json;
#[cfg(test)]
use tokio::net::UnixListener;

#[tokio::test]
async fn roundtrip() {
    let path = std::env::temp_dir().join(format!(
        "locus-sock-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    let listener = UnixListener::bind(&path).expect("bind test socket");
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept client");
        let request: Value = read_frame(&mut stream).await.expect("read request");
        assert_eq!(request, json!({"verb":"run.status"}));
        write_frame(&mut stream, &json!({"status":"running"}))
            .await
            .expect("write response");
    });

    let response: Value = SocketClient::round_trip(&path, &json!({"verb":"run.status"}))
        .await
        .expect("round trip succeeds");

    assert_eq!(response, json!({"status":"running"}));
    server.await.expect("server task completes");
    std::fs::remove_file(path).expect("remove test socket");
}

#[test]
fn stateless() {
    assert_eq!(std::mem::size_of::<SocketClient>(), 0);
}

#[tokio::test]
async fn all_verbs_are_round_trips() {
    let path = std::env::temp_dir().join(format!(
        "locus-sock-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    let listener = UnixListener::bind(&path).expect("bind test socket");
    let server = tokio::spawn(async move {
        for _ in VERB_DISPATCHES {
            let (mut stream, _) = listener.accept().await.expect("accept client");
            let request: Value = read_frame(&mut stream).await.expect("read request");
            assert_eq!(request["args"], json!(["argument"]));
            write_frame(&mut stream, &json!({"verb": request["verb"]}))
                .await
                .expect("write response");
        }
    });

    for verb in VERB_DISPATCHES {
        let response = dispatch(&path, verb, &["argument".to_owned()])
            .await
            .expect("verb round trip succeeds");
        assert_eq!(response, json!({"verb": verb.verb}));
    }

    server.await.expect("server task completes");
    std::fs::remove_file(path).expect("remove test socket");
}

#[cfg(test)]
mod json {
    use super::{compact_json, key_pack, resolve_verb, without_json_flag, VERB_DISPATCHES};
    use serde_json::json;

    #[test]
    fn flag_everywhere() {
        for dispatch in VERB_DISPATCHES {
            let mut command: Vec<_> = dispatch.command.iter().map(ToString::to_string).collect();
            command.push("--json".to_owned());

            let command = without_json_flag(&command);
            let (resolved, args) = resolve_verb(&command).expect("JSON command resolves");

            assert_eq!(resolved.verb, dispatch.verb);
            assert!(args.is_empty());
        }
    }

    #[test]
    fn never_pretty() {
        for dispatch in VERB_DISPATCHES {
            let response = json!({"verb": dispatch.verb, "rows": [{"id": 1, "state": "ready"}]});
            assert_eq!(
                compact_json(&response).expect("response serializes"),
                format!(
                    "{{\"verb\":\"{}\",\"rows\":[{{\"id\":1,\"state\":\"ready\"}}]}}",
                    dispatch.verb
                )
            );
        }
    }

    #[test]
    fn key_packed() {
        assert_eq!(
            key_pack(json!({
                "tasks": [
                    {"id": "a", "state": "ready"},
                    {"id": "b", "state": "done"}
                ]
            })),
            json!({
                "tasks": {
                    "keys": ["id", "state"],
                    "rows": [["a", "ready"], ["b", "done"]]
                }
            })
        );
    }

    #[test]
    fn threshold() {
        assert_eq!(
            key_pack(json!({"tasks": [{"id": "a", "state": "ready"}]})),
            json!({"tasks": {"keys": ["id", "state"], "rows": [["a", "ready"]]}})
        );
        assert_eq!(key_pack(json!({"tasks": []})), json!({"tasks": []}));
    }

    #[test]
    fn packing_saving() {
        let tasks: Vec<_> = (0..20)
            .map(|index| {
                json!({
                    "task_id": format!("task-{index:02}"), "workflow_id": "release",
                    "assigned_agent": "reviewer", "verification_command": "cargo test",
                    "status": "ready", "updated_at": "2026-08-21"
                })
            })
            .collect();
        let response = json!({"tasks": tasks});
        let minified = compact_json(&response).expect("minified response serializes");
        let packed = compact_json(&key_pack(response)).expect("packed response serializes");
        let saving = (minified.len() - packed.len()) * 100 / minified.len();
        assert!(
            (50..=60).contains(&saving),
            "expected packed table to save 50-60%, saved {saving}%"
        );
    }
}

#[cfg(test)]
mod run {
    use super::{resolve_verb, VERB_DISPATCHES};

    #[test]
    fn status() {
        let command = ["run".into(), "status".into()];
        let (dispatch, args) = resolve_verb(&command).expect("run status is dispatched");
        assert_eq!(dispatch.verb, "run.status");
        assert!(args.is_empty());
        assert!(VERB_DISPATCHES.iter().any(|entry| entry.verb == dispatch.verb));
    }

    #[test]
    fn artifacts() {
        let command = ["run".into(), "artifacts".into()];
        let (dispatch, args) = resolve_verb(&command).expect("run artifacts is dispatched");
        assert_eq!(dispatch.verb, "run.artifacts");
        assert!(args.is_empty());
        assert!(VERB_DISPATCHES.iter().any(|entry| entry.verb == dispatch.verb));
    }
}

#[cfg(test)]
mod svc {
    use super::resolve_verb;

    #[test]
    fn up_down() {
        for (command, verb) in [
            (["svc".into(), "up".into()], "svc.up"),
            (["svc".into(), "down".into()], "svc.down"),
        ] {
            let (dispatch, args) = resolve_verb(&command).expect("service command dispatches");
            assert_eq!(dispatch.verb, verb);
            assert!(args.is_empty());
        }
    }
}

#[test]
fn allowlist_message() {
    let error = allowed_verb(&["svc".into(), "restart".into()])
        .expect_err("unlisted command is refused before the socket");
    assert_eq!(error.to_string(), "command is not allowlisted: svc restart");
}

#[tokio::test]
async fn no_local_logic() {
    let path = std::env::temp_dir().join(format!(
        "locus-sock-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    let listener = UnixListener::bind(&path).expect("bind test socket");
    let answer = json!({
        "state": "waiting",
        "future_core_field": {"rows": [["one", 1], ["two", 2]]}
    });
    let server_answer = answer.clone();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept client");
        let request: Value = read_frame(&mut stream).await.expect("read request");
        assert_eq!(request, json!({"verb": "ask", "args": ["question"]}));
        write_frame(&mut stream, &server_answer)
            .await
            .expect("write response");
    });

    let ask = VERB_DISPATCHES
        .iter()
        .find(|dispatch| dispatch.verb == "ask")
        .expect("ask dispatch is registered");
    let response = dispatch(&path, ask, &["question".to_owned()])
        .await
        .expect("ask round trip succeeds");

    assert_eq!(response, answer);
    server.await.expect("server task completes");
    std::fs::remove_file(path).expect("remove test socket");
}
