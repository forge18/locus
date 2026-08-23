# F1 reconnaissance — per-project forwarding egress

## Implementation map

| Area | File / symbol | Finding |
| --- | --- | --- |
| Current policy model | `crates/locus-core/src/sandbox/egress.rs:10-45` — `EgressTier`, `EgressTarget`, `EgressTier::allows`, `AuditSink` | Tier-to-logical-target policy and audit contract only; no destination/IP/protocol allowlist or forwarding implementation. |
| Current credential egress | `crates/locus-core/src/sandbox/credential_proxy.rs:40-181` — `CredentialProxy::{container_environment_for_run,configure_run,release_run,listen_configured}` | Host listener has per-run nonce/tier binding and currently binds `0.0.0.0:44000`. |
| Credential authorization / forwarding | `crates/locus-core/src/sandbox/credential_proxy.rs:221-370` — `request_with_state`, `handle_proxy_connection`, `proxy_http_request` | Authenticates sentinel/nonce/tier then forwards only model HTTP requests to a configured upstream, injecting the host secret. |
| Run integration seam | `crates/locus-core/src/runtime/run.rs:131-303` — `SpawnRequest`, `CredentialProxyConfig`, `spawn_at_port` | Configures the credential proxy before container start, injects sentinel/nonce/run ID and proxy URL, and chooses `project_network(project_id)`. |
| Terminal cleanup | `crates/locus-core/src/runtime/run.rs:340-379` — `spawn_persisted`, `cancel_persisted`, `release_terminal_port` | Existing lifecycle seam revokes the credential-proxy run binding on failed spawn/cancel/terminal completion; no egress-proxy/project-network lifecycle exists. |
| Container networking | `crates/locus-core/src/runtime/container.rs:52-70,156-205` — `ContainerLaunch`, `ContainerRuntime::start_container`, `DockerContainerRuntime::start_container` | The sole Docker network control is `HostConfig.network_mode = launch.network`; no network create/remove, endpoint attachment, gateway, DNS, firewall, or proxy routing configuration. |
| Project network naming | `crates/locus-core/src/sandbox/ports.rs:34-40` — `project_network`, `same_project_network` | Names a network `locus-{project_id}` but does not create, inspect, or delete it. |
| Proxy audit persistence | `crates/locus-core/src/store/audits.rs:17-86` — `Store::persist_credential_proxy_audit`, `StoreAuditSink` | Durable audit adapter for the credential proxy; reusable audit seam if forwarding decisions must be recorded. |

## Tests to extend / add

- `crates/locus-core/src/sandbox/egress.rs:1-45`: add `egress::tiers` destination-allowlist unit coverage (specified in `.specs/security/tasks.md:11-15`).
- `crates/locus-core/src/sandbox/credential_proxy.rs:405-567` (`creds` tests): existing sentinel, revocation, denial-audit, and listener-forwarding fixtures can anchor forwarding-auth regressions.
- `crates/locus-core/src/runtime/run.rs:720-845`: spawn fixture asserts the project network, proxy listener, injected environment, and secret absence.
- `crates/locus-core/src/sandbox/ports.rs:60-76`: extend `net::project_isolation` for the required `None`-tier separation assertion.
- `crates/locus-core/tests/run.rs:1-19`: endpoint validation regression for any changed agent-visible endpoint.

## Proposed minimal integration shape

1. Add a sandbox-owned forwarding-egress component beside `credential_proxy.rs`; it owns per-project proxy lifecycle, per-run registration/revocation, tier-to-destination allowlists, and packet/connection forwarding.
2. Introduce a project-network lifecycle adapter in `runtime/container.rs` (or a dedicated Docker-network adapter): create/ensure the named project network and start/attach its forwarder before a run; remove/stop it only when the project has no active runs.
3. At `spawn_at_port` (`runtime/run.rs:245-303`), register `(run_id, nonce, egress_tier)` with the project forwarder and launch the agent on a network that cannot route directly outward. Inject only the forwarder endpoint/config; retain the credential proxy solely for secret injection.
4. At existing error/terminal paths (`runtime/run.rs:340-379`), revoke the forwarding registration alongside `CredentialProxy::release_run`; lifecycle code must also clean orphaned project forwarders/networks.
5. Keep `CredentialProxy` authorization/auditing for model credentials, but make its host listener reachable only through the forwarding path or otherwise authorize/project-scope every caller.

## Hazards / residual risks

- **HIGH — direct egress bypass:** `network_mode` attaches all project runs to an unrestricted Docker network (`runtime/container.rs:186-189`); logical `EgressTier::allows` gates only credential-proxy requests (`sandbox/credential_proxy.rs:221-269`).
- **HIGH — `None` isolation absent:** all runs derive the same `locus-{project_id}` name (`runtime/run.rs:303`, `sandbox/ports.rs:34-35`), so there is no current topology distinction for `None` vs egress-capable runs.
- **HIGH — host gateway exposure:** the credential listener binds all interfaces (`sandbox/credential_proxy.rs:168-176`) and the review records that host-gateway TCP is reachable by every container on macOS (`TODO.md:543-545`); nonce remains load-bearing, but a forwarding design must not treat the project network as the only caller boundary.
- **MEDIUM — lifecycle missing:** the repository has no Docker network create/remove calls; naming a network does not guarantee it exists, is private, or is reclaimed (`sandbox/ports.rs:34-40`, `runtime/container.rs:156-205`).
- **MEDIUM — protocol scope ambiguity:** current proxy parses HTTP and forces `EgressTarget::Model` (`sandbox/credential_proxy.rs:304-370`); “packet-level” F1 needs an explicit transport/DNS/CONNECT/TLS policy rather than reuse of the model HTTP parser.
- **MEDIUM — audit coverage gap:** `AuditSink` records credential-proxy decisions only (`sandbox/egress.rs:38-45`); direct sockets and forwarding denials need defined audit behavior if “all run egress” is audited.

```acceptance-report
{
  "criteriaSatisfied": [{"id":"criterion-1","status":"satisfied","evidence":"Concrete implementation seams and F1 HIGH/MEDIUM hazards are mapped with file:line references."}],
  "changedFiles": ["tmp/security-f1-scout.md"],
  "testsAddedOrUpdated": [],
  "commandsRun": [{"command":"rg -n -i … TODO.md apps packages services","result":"passed","summary":"Located F1 decision and core egress references."},{"command":"rg -n --glob '*.rs' … crates/locus-core/src crates/locus-core/tests","result":"passed","summary":"Located container, credential proxy, egress, project-network, and test seams."}],
  "validationOutput": ["Read-only reconnaissance; no production code modified."],
  "residualRisks": ["F1 is explicitly unimplemented; existing project network topology permits direct egress and same-project sharing."],
  "noStagedFiles": true,
  "diffSummary": "Added required reconnaissance artifact only.",
  "reviewFindings": ["high: crates/locus-core/src/runtime/container.rs:186-189 - unrestricted project network is the only Docker network setting.","high: crates/locus-core/src/sandbox/credential_proxy.rs:221-269 - tier enforcement covers only credential-proxy requests.","high: crates/locus-core/src/sandbox/credential_proxy.rs:168-176 - credential listener binds 0.0.0.0:44000."],
  "manualNotes": "F1 locked design and task verification targets are in .specs/security/spec.md:29-33 and .specs/security/tasks.md:11-15."
}
```
