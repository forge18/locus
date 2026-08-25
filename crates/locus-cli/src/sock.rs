use std::path::Path;

use anyhow::{Context, Result};
use locus_core::runtime::daemon::{
    AgentSocketError, AgentSocketErrorKind, AgentSocketResponse, AgentSocketVerb,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map, Value};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::UnixStream,
};

pub const DEFAULT_SOCKET_PATH: &str = "/run/locus.sock";
const KEY_PACK_ROW_THRESHOLD: usize = 2;
const MAX_SOCKET_FRAME_BYTES: u32 = 1_048_576;

#[derive(Debug, Serialize)]
pub struct SocketRequest<'a> {
    pub nonce: &'a str,
    pub verb: AgentSocketVerb,
    pub args: &'a [String],
}

#[derive(Debug)]
pub struct VerbDispatch {
    pub command: &'static [&'static str],
    pub verb: AgentSocketVerb,
}

pub const VERB_DISPATCHES: &[VerbDispatch] = &[
    VerbDispatch {
        command: &["memory", "note", "add"],
        verb: AgentSocketVerb::MemoryNoteAdd,
    },
    VerbDispatch {
        command: &["memory", "note", "replace"],
        verb: AgentSocketVerb::MemoryNoteReplace,
    },
    VerbDispatch {
        command: &["memory", "note", "remove"],
        verb: AgentSocketVerb::MemoryNoteRemove,
    },
    VerbDispatch {
        command: &["memory", "recall"],
        verb: AgentSocketVerb::MemoryRecall,
    },
    VerbDispatch {
        command: &["memory", "write"],
        verb: AgentSocketVerb::MemoryWrite,
    },
    VerbDispatch {
        command: &["memory", "forget"],
        verb: AgentSocketVerb::MemoryForget,
    },
    VerbDispatch {
        command: &["memory", "adjudicate"],
        verb: AgentSocketVerb::MemoryAdjudicate,
    },
    VerbDispatch {
        command: &["memory", "explain"],
        verb: AgentSocketVerb::MemoryExplain,
    },
    VerbDispatch {
        command: &["mail", "send"],
        verb: AgentSocketVerb::MailSend,
    },
    VerbDispatch {
        command: &["mail", "list"],
        verb: AgentSocketVerb::MailList,
    },
    VerbDispatch {
        command: &["mail", "read"],
        verb: AgentSocketVerb::MailRead,
    },
    VerbDispatch {
        command: &["mail", "reply"],
        verb: AgentSocketVerb::MailReply,
    },
    VerbDispatch {
        command: &["mail", "drain"],
        verb: AgentSocketVerb::MailDrain,
    },
    VerbDispatch {
        command: &["mail", "wait"],
        verb: AgentSocketVerb::MailWait,
    },
    VerbDispatch {
        command: &["task", "list"],
        verb: AgentSocketVerb::TaskList,
    },
    VerbDispatch {
        command: &["task", "show"],
        verb: AgentSocketVerb::TaskShow,
    },
    VerbDispatch {
        command: &["task", "move"],
        verb: AgentSocketVerb::TaskMove,
    },
    VerbDispatch {
        command: &["task", "assign"],
        verb: AgentSocketVerb::TaskAssign,
    },
    VerbDispatch {
        command: &["task", "comment"],
        verb: AgentSocketVerb::TaskComment,
    },
    VerbDispatch {
        command: &["wiki", "search"],
        verb: AgentSocketVerb::WikiSearch,
    },
    VerbDispatch {
        command: &["wiki", "read"],
        verb: AgentSocketVerb::WikiRead,
    },
    VerbDispatch {
        command: &["wiki", "write"],
        verb: AgentSocketVerb::WikiWrite,
    },
    VerbDispatch {
        command: &["wiki", "history"],
        verb: AgentSocketVerb::WikiHistory,
    },
    VerbDispatch {
        command: &["wiki", "ingest"],
        verb: AgentSocketVerb::WikiIngest,
    },
    VerbDispatch {
        command: &["wiki", "query"],
        verb: AgentSocketVerb::WikiQuery,
    },
    VerbDispatch {
        command: &["wiki", "lint"],
        verb: AgentSocketVerb::WikiLint,
    },
    VerbDispatch {
        command: &["lsp", "def"],
        verb: AgentSocketVerb::LspDef,
    },
    VerbDispatch {
        command: &["lsp", "refs"],
        verb: AgentSocketVerb::LspRefs,
    },
    VerbDispatch {
        command: &["lsp", "hover"],
        verb: AgentSocketVerb::LspHover,
    },
    VerbDispatch {
        command: &["lsp", "symbols"],
        verb: AgentSocketVerb::LspSymbols,
    },
    VerbDispatch {
        command: &["lsp", "rename"],
        verb: AgentSocketVerb::LspRename,
    },
    VerbDispatch {
        command: &["debug", "start"],
        verb: AgentSocketVerb::DebugStart,
    },
    VerbDispatch {
        command: &["debug", "break"],
        verb: AgentSocketVerb::DebugBreak,
    },
    VerbDispatch {
        command: &["debug", "step"],
        verb: AgentSocketVerb::DebugStep,
    },
    VerbDispatch {
        command: &["debug", "stack"],
        verb: AgentSocketVerb::DebugStack,
    },
    VerbDispatch {
        command: &["debug", "vars"],
        verb: AgentSocketVerb::DebugVars,
    },
    VerbDispatch {
        command: &["debug", "eval"],
        verb: AgentSocketVerb::DebugEval,
    },
    VerbDispatch {
        command: &["browse", "open"],
        verb: AgentSocketVerb::BrowseOpen,
    },
    VerbDispatch {
        command: &["browse", "click"],
        verb: AgentSocketVerb::BrowseClick,
    },
    VerbDispatch {
        command: &["browse", "fill"],
        verb: AgentSocketVerb::BrowseFill,
    },
    VerbDispatch {
        command: &["browse", "press"],
        verb: AgentSocketVerb::BrowsePress,
    },
    VerbDispatch {
        command: &["browse", "assert"],
        verb: AgentSocketVerb::BrowseAssert,
    },
    VerbDispatch {
        command: &["browse", "screenshot"],
        verb: AgentSocketVerb::BrowseScreenshot,
    },
    VerbDispatch {
        command: &["browse", "record"],
        verb: AgentSocketVerb::BrowseRecord,
    },
    VerbDispatch {
        command: &["browse", "console"],
        verb: AgentSocketVerb::BrowseConsole,
    },
    VerbDispatch {
        command: &["browse", "network"],
        verb: AgentSocketVerb::BrowseNetwork,
    },
    VerbDispatch {
        command: &["agent", "invoke"],
        verb: AgentSocketVerb::AgentInvoke,
    },
    VerbDispatch {
        command: &["svc", "up"],
        verb: AgentSocketVerb::SvcUp,
    },
    VerbDispatch {
        command: &["svc", "down"],
        verb: AgentSocketVerb::SvcDown,
    },
    VerbDispatch {
        command: &["ask"],
        verb: AgentSocketVerb::Ask,
    },
    VerbDispatch {
        command: &["run", "status"],
        verb: AgentSocketVerb::RunStatus,
    },
    VerbDispatch {
        command: &["run", "artifacts"],
        verb: AgentSocketVerb::RunArtifacts,
    },
    VerbDispatch {
        command: &["handoff"],
        verb: AgentSocketVerb::Handoff,
    },
    VerbDispatch {
        command: &["artifact", "put"],
        verb: AgentSocketVerb::ArtifactPut,
    },
    VerbDispatch {
        command: &["artifact", "get"],
        verb: AgentSocketVerb::ArtifactGet,
    },
    VerbDispatch {
        command: &["artifact", "comments"],
        verb: AgentSocketVerb::ArtifactComments,
    },
    VerbDispatch {
        command: &["tools", "list"],
        verb: AgentSocketVerb::ToolsList,
    },
    VerbDispatch {
        command: &["tools", "docs"],
        verb: AgentSocketVerb::ToolsDocs,
    },
    VerbDispatch {
        command: &["lint"],
        verb: AgentSocketVerb::Lint,
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
    resolve_verb(arguments)
        .ok_or_else(|| anyhow::anyhow!("command is not allowlisted: {}", arguments.join(" ")))
}

/// `textDocument/documentSymbol` needs the document it should inspect. Keep the CLI contract
/// explicit while leaving symbol resolution and server capabilities in the daemon/client layer.
pub fn validate_symbols_args(args: &[String]) -> anyhow::Result<()> {
    match args {
        [path] if !path.trim().is_empty() && !path.starts_with('-') => Ok(()),
        [] => anyhow::bail!("locus lsp symbols requires a file path"),
        _ => anyhow::bail!("locus lsp symbols accepts exactly one file path"),
    }
}

#[derive(Debug)]
pub enum DispatchError {
    Daemon {
        kind: AgentSocketErrorKind,
        message: String,
    },
    Transport(anyhow::Error),
}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Daemon { kind, message } => {
                write!(formatter, "daemon request refused ({kind:?}): {message}")
            }
            Self::Transport(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for DispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Daemon { .. } => None,
            Self::Transport(error) => Some(error.as_ref()),
        }
    }
}

impl From<anyhow::Error> for DispatchError {
    fn from(error: anyhow::Error) -> Self {
        Self::Transport(error)
    }
}

pub async fn dispatch(
    socket_path: impl AsRef<Path>,
    nonce: &str,
    dispatch: &VerbDispatch,
    args: &[String],
) -> std::result::Result<Value, DispatchError> {
    if nonce.trim().is_empty() {
        return Err(DispatchError::Transport(anyhow::anyhow!(
            "LOCUS_RUN_NONCE is required for daemon requests"
        )));
    }
    let response: AgentSocketResponse = SocketClient::round_trip(
        socket_path,
        &SocketRequest {
            nonce,
            verb: dispatch.verb,
            args,
        },
    )
    .await?;
    response.result.ok_or_else(|| match response.error {
        Some(error) => socket_error(error),
        None => {
            DispatchError::Transport(anyhow::anyhow!("daemon returned neither result nor error"))
        }
    })
}

fn socket_error(error: AgentSocketError) -> DispatchError {
    DispatchError::Daemon {
        kind: error.kind,
        message: error.message,
    }
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
    if length > MAX_SOCKET_FRAME_BYTES {
        anyhow::bail!("socket frame exceeds {MAX_SOCKET_FRAME_BYTES} bytes");
    }
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
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(test)]
use tokio::net::UnixListener;

#[cfg(test)]
fn test_socket_path() -> std::path::PathBuf {
    static NEXT_TEST_SOCKET: AtomicUsize = AtomicUsize::new(0);

    std::env::temp_dir().join(format!(
        "locus-sock-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos(),
        NEXT_TEST_SOCKET.fetch_add(1, Ordering::Relaxed),
    ))
}

#[tokio::test]
async fn roundtrip() {
    let path = test_socket_path();
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

#[tokio::test]
async fn preserves_daemon_error_kind_for_callers() {
    let path = test_socket_path();
    let listener = UnixListener::bind(&path).expect("bind test socket");
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept client");
        let _: Value = read_frame(&mut stream).await.expect("read request");
        write_frame(
            &mut stream,
            &AgentSocketResponse {
                result: None,
                error: Some(AgentSocketError {
                    kind: AgentSocketErrorKind::Unavailable,
                    message: "run status is not wired".into(),
                }),
            },
        )
        .await
        .expect("write response");
    });
    let verb_dispatch = VerbDispatch {
        command: &["run", "status"],
        verb: AgentSocketVerb::RunStatus,
    };

    let error = dispatch(&path, "nonce", &verb_dispatch, &[])
        .await
        .expect_err("daemon refusal is returned");
    assert!(matches!(
        error,
        DispatchError::Daemon {
            kind: AgentSocketErrorKind::Unavailable,
            ..
        }
    ));
    server.await.expect("server task completes");
    std::fs::remove_file(path).expect("remove test socket");
}

#[tokio::test]
async fn rejects_an_oversized_response_frame_before_allocating() {
    let (mut writer, mut reader) = tokio::io::duplex(8);
    assert!(writer.write_u32(1_048_577).await.is_ok());
    drop(writer);

    let error = match read_frame::<_, Value>(&mut reader).await {
        Ok(_) => panic!("oversized frame is rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("exceeds"));
}

#[test]
fn stateless() {
    assert_eq!(std::mem::size_of::<SocketClient>(), 0);
}

#[tokio::test]
async fn all_verbs_are_round_trips() {
    let path = test_socket_path();
    let listener = UnixListener::bind(&path).expect("bind test socket");
    let server = tokio::spawn(async move {
        for _ in VERB_DISPATCHES {
            let (mut stream, _) = listener.accept().await.expect("accept client");
            let request: Value = read_frame(&mut stream).await.expect("read request");
            assert_eq!(request["args"], json!(["argument"]));
            write_frame(
                &mut stream,
                &AgentSocketResponse {
                    result: Some(json!({"verb": request["verb"]})),
                    error: None,
                },
            )
            .await
            .expect("write response");
        }
    });

    for verb in VERB_DISPATCHES {
        let response = dispatch(&path, "nonce", verb, &["argument".to_owned()])
            .await
            .expect("verb round trip succeeds");
        assert_eq!(response, json!({"verb": verb.verb}));
    }

    server.await.expect("server task completes");
    std::fs::remove_file(path).expect("remove test socket");
}

#[cfg(test)]
mod lsp {
    use super::{allowed_verb, validate_symbols_args, AgentSocketVerb};

    #[test]
    fn symbols_requires_one_file_path() {
        let command = ["lsp".into(), "symbols".into(), "src/lib.rs".into()];
        let (dispatch, args) = allowed_verb(&command).expect("symbols is allowlisted");
        assert_eq!(dispatch.verb, AgentSocketVerb::LspSymbols);
        validate_symbols_args(args).expect("file path is accepted");
    }

    #[test]
    fn symbols_rejects_missing_or_ambiguous_path() {
        for args in [
            Vec::<String>::new(),
            vec!["src/lib.rs".into(), "extra.rs".into()],
            vec!["--all".into()],
        ] {
            assert!(validate_symbols_args(&args).is_err(), "accepted {args:?}");
        }
    }
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
            json!({"tasks": [{"id": "a", "state": "ready"}]})
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
mod mail {
    use super::*;

    fn command(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).into()).collect()
    }

    #[test]
    fn send() {
        let input = command(&["mail", "send", "reviewer", "body"]);
        let (dispatch, args) = allowed_verb(&input).expect("mail send is allowlisted");
        assert_eq!(dispatch.verb, AgentSocketVerb::MailSend);
        assert_eq!(args, ["reviewer", "body"]);
    }

    #[test]
    fn list_read() {
        for (parts, expected) in [
            (&["mail", "list"][..], AgentSocketVerb::MailList),
            (&["mail", "read", "thread-1"][..], AgentSocketVerb::MailRead),
        ] {
            let (dispatch, _) = allowed_verb(&command(parts)).expect("mail read/list allowlisted");
            assert_eq!(dispatch.verb, expected);
        }
    }

    #[test]
    fn reply_threads() {
        let input = command(&["mail", "reply", "thread-1", "answer"]);
        let (dispatch, args) = allowed_verb(&input).expect("mail reply is allowlisted");
        assert_eq!(dispatch.verb, AgentSocketVerb::MailReply);
        assert_eq!(args, ["thread-1", "answer"]);
    }

    #[test]
    fn drain() {
        let input = command(&["mail", "drain", "thread-1"]);
        let (dispatch, args) = allowed_verb(&input).expect("mail drain is allowlisted");
        assert_eq!(dispatch.verb, AgentSocketVerb::MailDrain);
        assert_eq!(args, ["thread-1"]);
    }

    #[test]
    fn wait_times_out() {
        let input = command(&["mail", "wait"]);
        let (dispatch, args) = allowed_verb(&input).expect("mail wait is allowlisted");
        assert_eq!(dispatch.verb, AgentSocketVerb::MailWait);
        assert!(args.is_empty());
    }
}

#[cfg(test)]
mod ask {
    use super::*;

    #[test]
    fn blocks_and_reaches_inbox() {
        let input: Vec<String> = ["ask", "Which deployment window?"]
            .into_iter()
            .map(String::from)
            .collect();
        let (dispatch, args) = allowed_verb(&input).expect("ask is allowlisted");
        assert_eq!(dispatch.verb, AgentSocketVerb::Ask);
        assert_eq!(args, ["Which deployment window?"]);
    }
}

#[cfg(test)]
mod run {
    use super::{resolve_verb, AgentSocketVerb, VERB_DISPATCHES};

    #[test]
    fn status() {
        let command = ["run".into(), "status".into()];
        let (dispatch, args) = resolve_verb(&command).expect("run status is dispatched");
        assert_eq!(dispatch.verb, AgentSocketVerb::RunStatus);
        assert!(args.is_empty());
        assert!(VERB_DISPATCHES
            .iter()
            .any(|entry| entry.verb == dispatch.verb));
    }

    #[test]
    fn artifacts() {
        let command = ["run".into(), "artifacts".into()];
        let (dispatch, args) = resolve_verb(&command).expect("run artifacts is dispatched");
        assert_eq!(dispatch.verb, AgentSocketVerb::RunArtifacts);
        assert!(args.is_empty());
        assert!(VERB_DISPATCHES
            .iter()
            .any(|entry| entry.verb == dispatch.verb));
    }
}

#[cfg(test)]
mod memory {
    use super::{resolve_verb, AgentSocketVerb};

    #[test]
    fn adjudicate() {
        let command = ["memory".into(), "adjudicate".into(), "fact-1".into()];
        let (dispatch, args) = resolve_verb(&command).expect("memory adjudicate dispatches");
        assert_eq!(dispatch.verb, AgentSocketVerb::MemoryAdjudicate);
        assert_eq!(args, ["fact-1"]);
    }

    #[test]
    fn explain() {
        let command = ["memory".into(), "explain".into(), "fact-1".into()];
        let (dispatch, args) = resolve_verb(&command).expect("memory explain dispatches");
        assert_eq!(dispatch.verb, AgentSocketVerb::MemoryExplain);
        assert_eq!(args, ["fact-1"]);
    }

    #[test]
    fn note_verbs() {
        for command in [
            ["memory", "note", "add"],
            ["memory", "note", "replace"],
            ["memory", "note", "remove"],
        ] {
            let args: Vec<String> = command.iter().map(ToString::to_string).collect();
            assert!(
                resolve_verb(&args).is_some(),
                "note verb is allowlisted: {command:?}"
            );
        }
    }

    #[test]
    fn store_verbs() {
        for command in [
            ["memory", "recall"],
            ["memory", "write"],
            ["memory", "forget"],
        ] {
            let args: Vec<String> = command.iter().map(ToString::to_string).collect();
            assert!(
                resolve_verb(&args).is_some(),
                "store verb is allowlisted: {command:?}"
            );
        }
    }
}

#[cfg(test)]
mod svc {
    use super::{resolve_verb, AgentSocketVerb};

    #[test]
    fn up_down() {
        for (command, verb) in [
            (["svc".into(), "up".into()], AgentSocketVerb::SvcUp),
            (["svc".into(), "down".into()], AgentSocketVerb::SvcDown),
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
        assert_eq!(
            request,
            json!({"nonce": "nonce", "verb": "ask", "args": ["question"]})
        );
        write_frame(
            &mut stream,
            &AgentSocketResponse {
                result: Some(server_answer),
                error: None,
            },
        )
        .await
        .expect("write response");
    });

    let ask = VERB_DISPATCHES
        .iter()
        .find(|dispatch| dispatch.verb == AgentSocketVerb::Ask)
        .expect("ask dispatch is registered");
    let response = dispatch(&path, "nonce", ask, &["question".to_owned()])
        .await
        .expect("ask round trip succeeds");

    assert_eq!(response, answer);
    server.await.expect("server task completes");
    std::fs::remove_file(path).expect("remove test socket");
}
