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

use serde_json::Value;

pub const INJECTION_TIMEOUT: Duration = Duration::from_millis(100);

/// Records one hook payload in the run-local NDJSON buffer. It deliberately has no
/// socket client: a synchronous daemon round trip in a hook can hang the harness.
pub fn append_to_buffer(path: &Path, payload: &Value) -> io::Result<()> {
    let mut buffer = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut buffer, payload).map_err(io::Error::other)?;
    buffer.write_all(b"\n")
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

/// Runs the hook binary. All malformed input, filesystem errors, and injection
/// timeouts are intentionally swallowed by the caller so its process exits zero.
pub fn run() -> io::Result<Option<Value>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let payload: Value = serde_json::from_str(&input).map_err(io::Error::other)?;
    if let Some(path) = env::var_os("LOCUS_HOOK_BUFFER") {
        append_to_buffer(Path::new(&path), &payload)?;
    }
    let injection = env::var("LOCUS_HOOK_INJECTION").ok();
    Ok(injection_with_timeout(move || {
        injection.and_then(|value| serde_json::from_str(&value).ok())
    })
    .flatten())
}

#[cfg(test)]
mod tests {
    use std::{fs, thread, time::Duration};

    use serde_json::json;

    use super::*;

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
}
