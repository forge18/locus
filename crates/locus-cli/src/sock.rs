use std::path::Path;

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::UnixStream,
};

pub const DEFAULT_SOCKET_PATH: &str = "/run/locus.sock";

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
