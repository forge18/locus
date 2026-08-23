//! Vendored, dependency-free HTTP/HTTPS forwarding proxy.
//!
//! Policy files are written by the Locus supervisor into `/locus/policies/<run-id>` and are
//! re-read for every request. Each file is four lines: nonce, tier, model hosts, package hosts.
//! This binary intentionally has no fallback/direct mode: an unrecognized request is denied.

use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    thread,
    time::Duration,
};

const POLICY_DIR: &str = "/locus/policies";
const MAX_HEADER_BYTES: usize = 32 * 1024;

struct Policy {
    nonce: String,
    tier: String,
    models: BTreeSet<String>,
    packages: BTreeSet<String>,
}

fn main() -> io::Result<()> {
    let bind = std::env::var("LOCUS_PROXY_BIND").unwrap_or_else(|_| "0.0.0.0:3128".into());
    let listener = TcpListener::bind(bind)?;
    for connection in listener.incoming() {
        match connection {
            Ok(client) => {
                thread::spawn(|| {
                    if let Err(error) = handle(client) {
                        eprintln!("locus egress proxy refused malformed request: {error}");
                    }
                });
            }
            Err(error) => eprintln!("locus egress proxy accept error: {error}"),
        }
    }
    Ok(())
}

fn handle(mut client: TcpStream) -> io::Result<()> {
    client.set_read_timeout(Some(Duration::from_secs(10)))?;
    let request = read_header(&mut client)?;
    let header_end = find_header_end(&request).ok_or_else(|| invalid("missing request header"))?;
    let head =
        std::str::from_utf8(&request[..header_end]).map_err(|_| invalid("non-utf8 header"))?;
    let (method, target) = request_line(head)?;
    let headers = headers(head);
    let (run_id, nonce) = proxy_identity(headers.get("proxy-authorization").copied())?;
    let Some(policy) = load_policy(&run_id) else {
        return deny(&mut client, &run_id, "missing policy");
    };
    if nonce != policy.nonce {
        return deny(&mut client, &run_id, "bad capability");
    }
    let host = destination_host(method, target, headers.get("host").copied())?;
    if !permitted(&policy, &host) {
        return deny(&mut client, &run_id, "destination denied");
    }
    let method = method.to_owned();
    let target = target.to_owned();
    eprintln!("locus egress proxy allow run={run_id} host={host}");
    if method == "CONNECT" {
        connect_tunnel(client, &host)
    } else {
        forward_http(client, request, header_end, &method, &target, &host)
    }
}

fn read_header(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 4096];
    while request.len() < MAX_HEADER_BYTES {
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..count]);
        if find_header_end(&request).is_some() {
            return Ok(request);
        }
    }
    Err(invalid("request header exceeds limit"))
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|end| end + 4)
}

fn request_line(head: &str) -> io::Result<(&str, &str)> {
    let line = head
        .lines()
        .next()
        .ok_or_else(|| invalid("missing request line"))?;
    let mut fields = line.split_whitespace();
    let method = fields.next().ok_or_else(|| invalid("missing method"))?;
    let target = fields
        .next()
        .ok_or_else(|| invalid("missing request target"))?;
    if fields.next().is_none() {
        return Err(invalid("missing HTTP version"));
    }
    Ok((method, target))
}

fn headers(head: &str) -> std::collections::BTreeMap<String, &str> {
    head.lines()
        .skip(1)
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim()))
        .collect()
}

fn proxy_identity(value: Option<&str>) -> io::Result<(String, String)> {
    let encoded = value
        .and_then(|value| value.strip_prefix("Basic "))
        .ok_or_else(|| invalid("missing proxy authentication"))?;
    let decoded =
        String::from_utf8(base64_decode(encoded)?).map_err(|_| invalid("invalid identity"))?;
    let (run_id, nonce) = decoded
        .split_once(':')
        .ok_or_else(|| invalid("invalid identity"))?;
    let run_id = percent_decode(run_id)?;
    let nonce = percent_decode(nonce)?;
    if run_id.is_empty() || nonce.is_empty() || !valid_file_name(&run_id) {
        return Err(invalid("invalid identity"));
    }
    Ok((run_id, nonce))
}

fn load_policy(run_id: &str) -> Option<Policy> {
    let directory = std::env::var("LOCUS_POLICY_DIR").unwrap_or_else(|_| POLICY_DIR.into());
    let contents = fs::read_to_string(format!("{directory}/{run_id}")).ok()?;
    let mut lines = contents.lines();
    let nonce = lines.next()?.to_owned();
    let tier = lines.next()?.to_owned();
    let models = host_set(lines.next().unwrap_or_default());
    let packages = host_set(lines.next().unwrap_or_default());
    Some(Policy {
        nonce,
        tier,
        models,
        packages,
    })
}

fn host_set(value: &str) -> BTreeSet<String> {
    value
        .split(',')
        .filter(|host| !host.is_empty())
        .map(normalize_host)
        .collect()
}

fn destination_host(method: &str, target: &str, host_header: Option<&str>) -> io::Result<String> {
    let authority = if method == "CONNECT" {
        target
    } else if let Some(rest) = target
        .strip_prefix("http://")
        .or_else(|| target.strip_prefix("https://"))
    {
        rest.split('/').next().unwrap_or_default()
    } else {
        host_header.ok_or_else(|| invalid("missing absolute destination"))?
    };
    let authority = authority.rsplit('@').next().unwrap_or_default();
    let host = authority
        .strip_prefix('[')
        .and_then(|value| value.split_once(']').map(|(host, _)| host))
        .unwrap_or_else(|| authority.split(':').next().unwrap_or_default());
    let host = normalize_host(host);
    if host.is_empty() {
        return Err(invalid("missing destination host"));
    }
    Ok(host)
}

fn permitted(policy: &Policy, host: &str) -> bool {
    match policy.tier.as_str() {
        "open" => !host.is_empty(),
        "model" => policy.models.contains(host),
        "packages" => policy.models.contains(host) || policy.packages.contains(host),
        _ => false,
    }
}

fn connect_tunnel(mut client: TcpStream, host: &str) -> io::Result<()> {
    let mut upstream = TcpStream::connect(format!("{host}:443"))?;
    client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")?;
    relay(&mut client, &mut upstream)
}

fn forward_http(
    mut client: TcpStream,
    request: Vec<u8>,
    header_end: usize,
    method: &str,
    target: &str,
    host: &str,
) -> io::Result<()> {
    let port = if target.starts_with("https://") {
        443
    } else {
        80
    };
    let mut upstream = TcpStream::connect(format!("{host}:{port}"))?;
    let head =
        std::str::from_utf8(&request[..header_end]).map_err(|_| invalid("non-utf8 header"))?;
    let path = target
        .strip_prefix("http://")
        .or_else(|| target.strip_prefix("https://"))
        .and_then(|rest| rest.find('/').map(|offset| &rest[offset..]))
        .unwrap_or("/");
    let mut rewritten = format!("{method} {path} HTTP/1.1\r\n");
    for line in head.lines().skip(1) {
        if !line
            .to_ascii_lowercase()
            .starts_with("proxy-authorization:")
        {
            rewritten.push_str(line);
            rewritten.push_str("\r\n");
        }
    }
    rewritten.push_str("\r\n");
    upstream.write_all(rewritten.as_bytes())?;
    upstream.write_all(&request[header_end..])?;
    relay(&mut client, &mut upstream)
}

fn relay(client: &mut TcpStream, upstream: &mut TcpStream) -> io::Result<()> {
    let mut upstream_write = upstream.try_clone()?;
    let mut client_read = client.try_clone()?;
    let outbound = thread::spawn(move || {
        let result = io::copy(&mut client_read, &mut upstream_write);
        let _ = upstream_write.shutdown(Shutdown::Write);
        result
    });
    let inbound = io::copy(upstream, client);
    let _ = client.shutdown(Shutdown::Write);
    let _ = outbound.join();
    inbound.map(|_| ())
}

fn deny(client: &mut TcpStream, run_id: &str, reason: &str) -> io::Result<()> {
    eprintln!("locus egress proxy deny run={run_id} reason={reason}");
    client.write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
}

fn normalize_host(host: &str) -> String {
    host.trim().trim_end_matches('.').to_ascii_lowercase()
}

fn valid_file_name(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && !value.contains('/')
        && !value.contains('\\')
}

fn percent_decode(value: &str) -> io::Result<String> {
    let mut output = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let high = hex(bytes[index + 1]).ok_or_else(|| invalid("invalid percent encoding"))?;
            let low = hex(bytes[index + 2]).ok_or_else(|| invalid("invalid percent encoding"))?;
            output.push(high * 16 + low);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output).map_err(|_| invalid("invalid percent encoding"))
}

fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn base64_decode(value: &str) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut bits = 0_u32;
    let mut count = 0_u8;
    for byte in value.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        if byte == b'=' {
            break;
        }
        let Some(value) = (match byte {
            b'A'..=b'Z' => Some(byte - b'A'),
            b'a'..=b'z' => Some(byte - b'a' + 26),
            b'0'..=b'9' => Some(byte - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }) else {
            return Err(invalid("invalid base64"));
        };
        bits = (bits << 6) | u32::from(value);
        count += 1;
        if count == 4 {
            output.extend_from_slice(&bits.to_be_bytes()[1..]);
            bits = 0;
            count = 0;
        }
    }
    match count {
        0 => Ok(output),
        2 => {
            output.push((bits >> 4) as u8);
            Ok(output)
        }
        3 => {
            output.push((bits >> 10) as u8);
            output.push((bits >> 2) as u8);
            Ok(output)
        }
        _ => Err(invalid("invalid base64")),
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
