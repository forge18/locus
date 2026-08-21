use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::UnixStream,
};

pub const DEFAULT_SOCKET_PATH: &str = "/run/locus.sock";

#[derive(Clone, Debug)]
pub struct SocketClient {
    path: PathBuf,
}

impl SocketClient {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub async fn round_trip<Request, Response>(&self, request: &Request) -> Result<Response>
    where
        Request: Serialize,
        Response: DeserializeOwned,
    {
        let mut stream = UnixStream::connect(&self.path)
            .await
            .with_context(|| format!("connect to daemon socket `{}`", self.path.display()))?;
        write_frame(&mut stream, request).await?;
        read_frame(&mut stream).await
    }
}

impl Default for SocketClient {
    fn default() -> Self {
        Self::new(DEFAULT_SOCKET_PATH)
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
use serde_json::{json, Value};
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

    let response: Value = SocketClient::new(&path)
        .round_trip(&json!({"verb":"run.status"}))
        .await
        .expect("round trip succeeds");

    assert_eq!(response, json!({"status":"running"}));
    server.await.expect("server task completes");
    std::fs::remove_file(path).expect("remove test socket");
}
