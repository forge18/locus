//! The host credential proxy: only a sentinel and a proxy URL ever enter a container.

use super::*;
use crate::sandbox::egress::{AuditSink, EgressTarget, EgressTier, OutboundAudit};

const CREDENTIAL_SENTINEL: &str = "sk-locus-sentinel";

#[derive(Clone)]
struct CredentialProxyRun {
    nonce: String,
    tier: EgressTier,
}

struct CredentialProxyState {
    secret: String,
    credential_class: &'static str,
    runs: Mutex<HashMap<String, CredentialProxyRun>>,
    audit: Mutex<Vec<OutboundAudit>>,
    audit_sink: Mutex<Option<Arc<dyn AuditSink>>>,
}

struct CredentialProxyListener {
    stop: Arc<AtomicBool>,
    task: JoinHandle<()>,
    address: SocketAddr,
}

impl Drop for CredentialProxyListener {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.task.thread().unpark();
    }
}

/// Host-side credential proxy. Only a sentinel and proxy URL enter the container.
pub struct CredentialProxy {
    state: Arc<CredentialProxyState>,
    upstream: String,
    listener: Mutex<Option<CredentialProxyListener>>,
}

impl CredentialProxy {
    pub fn new(secret: impl Into<String>, credential_class: &'static str) -> Self {
        Self::with_upstream(secret, credential_class, "https://api.anthropic.com")
    }

    /// Testable host-only upstream configuration. The agent never receives this URL or secret.
    pub fn with_upstream(
        secret: impl Into<String>,
        credential_class: &'static str,
        upstream: impl Into<String>,
    ) -> Self {
        Self {
            state: Arc::new(CredentialProxyState {
                secret: secret.into(),
                credential_class,
                runs: Mutex::new(HashMap::new()),
                audit: Mutex::new(Vec::new()),
                audit_sink: Mutex::new(None),
            }),
            upstream: upstream.into().trim_end_matches('/').into(),
            listener: Mutex::new(None),
        }
    }

    pub fn container_environment(&self, run_nonce: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("ANTHROPIC_API_KEY".into(), CREDENTIAL_SENTINEL.into()),
            (
                "ANTHROPIC_BASE_URL".into(),
                "http://host.docker.internal:43800".into(),
            ),
            ("LOCUS_RUN_NONCE".into(), run_nonce.into()),
        ])
    }

    pub fn container_environment_for_run(
        &self,
        run_id: &str,
        run_nonce: &str,
    ) -> BTreeMap<String, String> {
        let mut environment = self.container_environment(run_nonce);
        environment.insert("LOCUS_RUN_ID".into(), run_id.into());
        environment
    }

    /// Bind a run's nonce and egress capability before its container starts.
    /// Attach the durable sink before accepting agent requests. The proxy records through
    /// it rather than holding a store, which is what keeps `sandbox` free of persistence.
    pub fn attach_audit_sink(&self, sink: Arc<dyn AuditSink>) {
        *self
            .state
            .audit_sink
            .lock()
            .expect("credential proxy audit sink lock") = Some(sink);
    }

    pub fn configure_run(&self, run_id: &str, nonce: &str, tier: EgressTier) -> Result<()> {
        if run_id.trim().is_empty() || nonce.trim().is_empty() {
            bail!("credential proxy run binding requires a run id and nonce")
        }
        self.state
            .runs
            .lock()
            .expect("credential proxy runs lock")
            .insert(
                run_id.into(),
                CredentialProxyRun {
                    nonce: nonce.into(),
                    tier,
                },
            );
        Ok(())
    }

    /// Start the host listener once. Each inbound request reaches `request` before forwarding.
    pub fn listen(&self, bind: SocketAddr) -> Result<SocketAddr> {
        let mut listener = self
            .listener
            .lock()
            .expect("credential proxy listener lock");
        if let Some(listener) = listener.as_ref() {
            return Ok(listener.address);
        }
        let socket = TcpListener::bind(bind).context("bind credential proxy listener")?;
        socket
            .set_nonblocking(true)
            .context("configure credential proxy listener")?;
        let address = socket
            .local_addr()
            .context("read credential proxy listener address")?;
        let stop = Arc::new(AtomicBool::new(false));
        let state = self.state.clone();
        let upstream = self.upstream.clone();
        let stop_for_task = stop.clone();
        let task = thread::spawn(move || {
            while !stop_for_task.load(Ordering::Acquire) {
                match socket.accept() {
                    Ok((stream, _)) => handle_proxy_connection(stream, &state, &upstream),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::park_timeout(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        *listener = Some(CredentialProxyListener {
            stop,
            task,
            address,
        });
        Ok(address)
    }

    /// Start the configured host gateway used by agent containers.
    pub fn listen_configured(&self) -> Result<SocketAddr> {
        self.listen(
            "0.0.0.0:43800"
                .parse()
                .expect("valid configured proxy address"),
        )
    }

    pub fn listener_address(&self) -> Option<SocketAddr> {
        self.listener
            .lock()
            .expect("credential proxy listener lock")
            .as_ref()
            .map(|listener| listener.address)
    }

    /// Forward one host-side proxy request after exchanging the run sentinel for the host secret.
    /// The forwarding closure is host-only; its credential argument is never returned or audited.
    pub fn request<T>(
        &self,
        run_id: &str,
        supplied_nonce: &str,
        supplied_credential: &str,
        target: EgressTarget,
        forward: impl FnOnce(&str) -> Result<T>,
    ) -> Result<T> {
        request_with_state(
            &self.state,
            run_id,
            supplied_nonce,
            supplied_credential,
            target,
            forward,
        )
    }

    pub fn audit_rows(&self) -> Vec<OutboundAudit> {
        self.state.audit.lock().expect("audit lock").clone()
    }

    pub fn contains_secret(&self, value: &str) -> bool {
        value.contains(&self.state.secret)
    }
}

fn request_with_state<T>(
    state: &CredentialProxyState,
    run_id: &str,
    supplied_nonce: &str,
    supplied_credential: &str,
    target: EgressTarget,
    forward: impl FnOnce(&str) -> Result<T>,
) -> Result<T> {
    let binding = state
        .runs
        .lock()
        .expect("credential proxy runs lock")
        .get(run_id)
        .cloned();
    let tier = binding
        .as_ref()
        .map_or(EgressTier::None, |binding| binding.tier);
    let allowed = binding.is_some_and(|binding| {
        supplied_nonce == binding.nonce
            && supplied_credential == CREDENTIAL_SENTINEL
            && binding.tier.allows(target)
    });
    let audit = OutboundAudit {
        run_id: run_id.into(),
        target,
        tier,
        allowed,
        credential_class: state.credential_class,
    };
    state.audit.lock().expect("audit lock").push(audit.clone());
    let sink = state
        .audit_sink
        .lock()
        .expect("credential proxy audit sink lock")
        .clone();
    if let Some(sink) = sink {
        // Durable before the call is forwarded: an outbound request that is not recorded
        // is not made.
        sink.record(&audit)?;
    }
    if !allowed {
        bail!("credential proxy refused outbound request")
    }
    forward(&state.secret)
}

fn handle_proxy_connection(mut stream: TcpStream, state: &CredentialProxyState, upstream: &str) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let mut request = Vec::new();
    let mut chunk = [0_u8; 4096];
    while request.len() < 1024 * 1024 {
        let Ok(read) = stream.read(&mut chunk) else {
            return;
        };
        if read == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..read]);
        if let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") {
            let header_end = header_end + 4;
            let head = String::from_utf8_lossy(&request[..header_end]);
            let content_length = head
                .lines()
                .find_map(|line| {
                    line.split_once(':')
                        .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                        .and_then(|(_, value)| value.trim().parse::<usize>().ok())
                })
                .unwrap_or(0);
            if request.len() >= header_end + content_length {
                break;
            }
        }
    }
    let response = proxy_http_request(&request, state, upstream);
    let _ = stream.write_all(&response);
}

fn proxy_http_request(request: &[u8], state: &CredentialProxyState, upstream: &str) -> Vec<u8> {
    let request = String::from_utf8_lossy(request);
    let (head, body) = request.split_once("\r\n\r\n").unwrap_or((&request, ""));
    let mut lines = head.lines();
    let Some(request_line) = lines.next() else {
        return http_response(400, b"bad request");
    };
    let mut parts = request_line.split_whitespace();
    let (Some(method), Some(path), Some(_)) = (parts.next(), parts.next(), parts.next()) else {
        return http_response(400, b"bad request");
    };
    if !path.starts_with('/') {
        return http_response(400, b"bad request");
    }
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim()))
        .collect::<HashMap<_, _>>();
    let run_id = headers.get("x-locus-run-id").copied().unwrap_or("");
    let nonce = headers.get("x-locus-run-nonce").copied().unwrap_or("");
    let sentinel = headers
        .get("authorization")
        .and_then(|value| value.strip_prefix("Bearer "))
        .or_else(|| headers.get("x-api-key").copied())
        .unwrap_or("");
    match request_with_state(
        state,
        run_id,
        nonce,
        sentinel,
        EgressTarget::Model,
        |secret| {
            let client = reqwest::blocking::Client::new();
            let mut outbound = client
                .request(
                    method.parse().context("parse proxied method")?,
                    format!("{upstream}{path}"),
                )
                .header("x-api-key", secret);
            for name in ["content-type", "anthropic-version"] {
                if let Some(value) = headers.get(name) {
                    outbound = outbound.header(name, *value);
                }
            }
            let response = outbound
                .body(body.as_bytes().to_vec())
                .send()
                .context("send host credential proxy request")?;
            Ok((
                response.status().as_u16(),
                response
                    .bytes()
                    .context("read host proxy response")?
                    .to_vec(),
            ))
        },
    ) {
        Ok((status, body)) => http_response(status, &body),
        Err(_) => http_response(401, b"credential proxy refused request"),
    }
}

fn http_response(status: u16, body: &[u8]) -> Vec<u8> {
    let reason = if (200..300).contains(&status) {
        "OK"
    } else {
        "Unauthorized"
    };
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes()
    .into_iter()
    .chain(body.iter().copied())
    .collect()
}

pub fn no_long_lived_secret(
    secret: &str,
    environment: &BTreeMap<String, String>,
    files: &[String],
) -> bool {
    !environment
        .values()
        .chain(files.iter())
        .any(|value| value.contains(secret))
}

#[cfg(test)]
mod creds {
    use super::*;

    #[test]
    fn injects() {
        let proxy = CredentialProxy::new("real-secret", "api_key");
        let environment = proxy.container_environment("nonce");
        assert_eq!(environment["ANTHROPIC_API_KEY"], "sk-locus-sentinel");
        assert!(!proxy.contains_secret(&environment["ANTHROPIC_API_KEY"]));
    }

    #[test]
    fn no_long_lived_secret() {
        let proxy = CredentialProxy::new("real-secret", "oauth");
        assert!(super::no_long_lived_secret(
            "real-secret",
            &proxy.container_environment("nonce"),
            &["config".into()]
        ));
    }

    #[test]
    fn request_exchanges_only_the_run_sentinel_for_the_host_secret() {
        let proxy = CredentialProxy::new("real-secret", "api_key");
        proxy
            .configure_run("run", "nonce", EgressTier::Model)
            .unwrap();
        let forwarded = proxy
            .request(
                "run",
                "nonce",
                "sk-locus-sentinel",
                EgressTarget::Model,
                |secret| {
                    assert_eq!(secret, "real-secret");
                    Ok("host response")
                },
            )
            .unwrap();
        assert_eq!(forwarded, "host response");
        let audit = proxy.audit_rows();
        assert_eq!(audit.len(), 1);
        assert!(audit[0].allowed);
        assert!(!format!("{:?}", audit).contains("real-secret"));
    }

    #[test]
    fn request_denials_are_audited_per_request() {
        let proxy = CredentialProxy::new("real-secret", "api_key");
        proxy
            .configure_run("run", "nonce", EgressTier::Model)
            .unwrap();
        assert!(proxy
            .request(
                "run",
                "wrong",
                "sk-locus-sentinel",
                EgressTarget::Model,
                |_| Ok(())
            )
            .is_err());
        assert!(proxy
            .request(
                "run",
                "nonce",
                "sk-locus-sentinel",
                EgressTarget::Package,
                |_| Ok(())
            )
            .is_err());
        assert!(proxy
            .request("run", "nonce", "wrong", EgressTarget::Model, |_| Ok(()))
            .is_err());
        let audit = proxy.audit_rows();
        assert_eq!(audit.len(), 3);
        assert!(audit.iter().all(|row| !row.allowed));
    }

    #[test]
    fn listener_authenticates_and_injects_the_secret_only_upstream() {
        let upstream = TcpListener::bind("127.0.0.1:0").unwrap();
        let upstream_address = upstream.local_addr().unwrap();
        let (received_tx, received_rx) = std::sync::mpsc::channel();
        thread::spawn(move || {
            let (mut stream, _) = upstream.accept().unwrap();
            let mut bytes = [0_u8; 4096];
            let read = stream.read(&mut bytes).unwrap();
            received_tx
                .send(String::from_utf8_lossy(&bytes[..read]).into_owned())
                .unwrap();
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
                .unwrap();
        });

        let proxy = CredentialProxy::with_upstream(
            "real-secret",
            "api_key",
            format!("http://{upstream_address}"),
        );
        proxy
            .configure_run("run", "nonce", EgressTier::Model)
            .unwrap();
        let address = proxy.listen("127.0.0.1:0".parse().unwrap()).unwrap();
        let mut agent = TcpStream::connect(address).unwrap();
        agent.write_all(b"GET /v1/messages HTTP/1.1\r\nHost: proxy\r\nX-Api-Key: sk-locus-sentinel\r\nX-Locus-Run-Id: run\r\nX-Locus-Run-Nonce: nonce\r\nConnection: close\r\n\r\n").unwrap();
        let mut response = String::new();
        agent.read_to_string(&mut response).unwrap();

        assert!(response.starts_with("HTTP/1.1 200"));
        assert!(received_rx
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .contains("x-api-key: real-secret"));
        assert_eq!(proxy.audit_rows().len(), 1);
        assert!(proxy.audit_rows()[0].allowed);
        assert!(!response.contains("real-secret"));
    }
}
