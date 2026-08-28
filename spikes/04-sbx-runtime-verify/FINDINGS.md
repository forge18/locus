# Spike 4 — sbx runtime verification · FINDINGS

**Status**: all three checks exercised and passing on one machine (macOS 26.2, Apple M4 Max, sbx
v0.39.0). Unlike spikes 01–03, no QUESTION.md was fixed before running — the questions came from
Locus's integration unknowns for Docker's `sbx` as the agent-container runtime, and the test
scripts were deleted during the spike's own teardown, so this document is transcript-evidenced,
not script-replayable. Every verdict below names the command that produced it.

| | Question | Verdict |
| --- | --- | --- |
| Q1 | Does `sbx exec -i` carry ACP-style framed stdio byte-exact, with correct exit codes and EOF semantics? | **Yes — 7/7** |
| Q2 | Can the VM reach host services through the egress proxy — HTTP, git-over-TCP, raw TCP — under sandbox-scoped policy? | **Yes — 3/3, plus 2 negative controls blocked** |
| Q3 | Is the lifecycle fast enough for disposable-per-run sandboxes? | **Yes — 3.2s cached create, 1.1s resume from stop** |

---

## Q1 — stdio fidelity (ACP transport)

**VERDICT: `sbx exec -i` is a clean byte-exact pipe. The "flags match docker exec" claim holds in
practice, so the ACP driver swap is real.**

Host driver (uv-run Python) spawned `sbx exec -i locus-verify python3 /tmp/sbx-verify/resp.py`;
the responder parsed Content-Length-framed JSON-RPC from stdin and replied with framed responses.

| # | Case | Result |
| --- | --- | --- |
| 1 | Small frame round-trip (`ping` → `pong`, framed) | PASS |
| 2 | 1MB framed payload host → sandbox | PASS, byte-exact |
| 3 | 1MB framed payload sandbox → host | PASS, byte-exact |
| 4 | Hostile bytes inside payloads (CRLF, NUL, high bytes) | PASS, preserved |
| 5 | Clean shutdown, exit 0 | PASS |
| 6 | `sbx exec -i … sh -c 'exit 7'` propagates exit 7 | PASS |
| 7 | `printf '' \| sbx exec -i … cat` — stdin EOF → clean exit | PASS |

One observation to re-check when wiring a real harness: every `sbx exec` emits a
`Sandbox … started successfully` line. The framing checks passed byte-exact, so this does not
contaminate stdout (framed JSON parsed cleanly end to end), but the noise is on the channel
somewhere and should be confirmed stderr-only before an ACP client trusts the stream.

## Q2 — VM→host networking + scoped egress policy

**VERDICT: host.docker.internal works for HTTP and raw TCP through the proxy; git clone over the
git protocol to a host-side bare remote works end to end; default-deny blocks everything not
explicitly allowed. The bare-remote clone model in PLAN.md is proven, not just documented.**

Setup: host listeners on loopback — `git daemon` :9418 serving a bare repo, `uv run python -m
http.server` :8922, raw TCP echo server :9419. Policy: `sbx policy init balanced` once
(machine-level), then `sbx policy allow network --sandbox locus-verify
"localhost:9418,localhost:8922,localhost:9419"` after create.

| # | Case | Result |
| --- | --- | --- |
| 1 | `curl http://host.docker.internal:8922/seed.txt` from VM | PASS, exact host content |
| 2 | `git clone git://host.docker.internal:9418/repo.git` from VM | PASS, content verified |
| 3 | 256KB random binary through the TCP echo, VM → host → VM | PASS, byte-exact |
| 4 | Unallowed port `:9999` | BLOCKED — "no matching allow rule — blocked by default deny" |
| 5 | Unallowed external host (`example.com:80`) | BLOCKED — HTTP 403 |

Four measured gotchas, each one a place locusd must behave deliberately:

1. **Denials surface as HTTP-level responses, and curl exits 0.** A caller detecting egress
   denial from exit codes sees success; it must read the response body or status.
2. **Sandbox-scoped allows require the sandbox to exist.** `sbx policy allow --sandbox` before
   create fails with "sandbox not found" — add rules after `sbx create`.
3. **Policy needs one-time initialization.** No allow is accepted until
   `sbx policy init <allow-all|balanced|deny-all>` has run on the machine.
4. **Scoped rules are lifecycle-bound.** After `sbx rm`, `sbx policy ls` shows only the global
   layer — per-sandbox rules die with the sandbox (verified).

## Q3 — lifecycle

**VERDICT: disposable-per-run VMs are viable. The first create's 37s was a one-time template
pull; everything after that is seconds.**

| Operation | Time |
| --- | --- |
| First `sbx create` (includes template pull) | 37.2s |
| `sbx create`, template cached | 3.2s |
| Resume from stop (exec auto-start) | ~1.1s |
| Steady-state `sbx exec` spawn | ~1.35s |
| `sbx stop` | 5.3s |

`sbx ls --json` returned clean structured state at every step. The 1.35s exec spawn is once per
ACP session — one per run — and the `locus` CLI socket path becomes TCP regardless, so the
spawn cost does not sit on any per-verb path.

## Environment facts

- `shell` template: Ubuntu 26.04, python3 present; **git, curl, node, nc absent**. Installed via
  `sbx exec -u root locus-verify apt-get install git curl` (passwordless root works). Locus
  images must bake git and curl; the stock template does not.
- VM sees 14 vCPU and ~18GB RAM on the test machine.

## NOT EXERCISED

Named, not hidden — these stay unknown after this spike:

- A real harness process. A Python responder stood in for ACP; no agent template (`claude`, …)
  was run inside the sandbox.
- Linux/Windows runtime variance. All evidence is macOS-only.
- Long-run stability, disk growth, snapshot behavior over hours.
- Upstream proxy chaining — Locus's credential broker as sbx's upstream proxy (HTTP/HTTPS only).

## Machine state after the spike

- sbx v0.39.0 installed via Homebrew (`docker/tap`), signed in as forge18. The session crash lost
  the auth secret; re-authenticated afterward.
- Global network policy initialized to `balanced` (193 allow rules, deny-by-default) — it was
  uninitialized before this spike. **Kept** per user decision after the spike; the Locus
  posture (deny-by-default egress, per-sandbox scoping) is the reason.
- Everything else cleaned: both verify sandboxes removed, host listeners killed, `/tmp` scratch
  deleted, sandbox-scoped policy rules removed automatically with their sandbox.

Recorded as `spikes/04-sbx-runtime-verify/` (numbered per the convention of spikes 01–03), without
a QUESTION.md — unlike the earlier spikes, the questions were fixed mid-session rather than before.
