# acp-client

**Milestone** M1 · **Depends on** `sandbox`, `telemetry` · **Blocks** `planning-module`, `pane-manager`, `run-supervisor`

## Purpose

An Agent Client Protocol client — **the only agent interface**. ACP is the one transport over which an
agent speaks to Locus. Every supported harness fronts it, every agent session answers through it, and
the PTY is retired from the agent surface entirely.

This supersedes PLAN.md's former planning-only ACP framing. The revision is
deliberate and architecture-wide; the terminal/PTY docs that asserted the old model have been
reconciled to ACP-only, and this spec is the keystone that broke the old plan.

**ACP fits the job it has taken over.** What it carried as a chat — `session/new`, `session/prompt`,
streamed updates, tool-call permission requests — is the vocabulary of any agent conversation, and it is
now the vocabulary every agent pane speaks.

## Governed by

- PLAN.md §ACP — superseded in part: ACP is no longer planning-module only. PLAN.md §Harness I/O,
  §One clarification, and the container row were reconciled to terminal-free ACP in the sweep.
- PLAN.md §Canonical event vocabulary — the `acp` source row
- PLAN.md ADR-worthy decision: no system-prompt control. **Kept.** `session/new` accepts only `cwd`
  and `mcpServers`, so prompt assembly belongs to the harness. ACP being the only interface does not
  change that — an ACP session is not the vehicle for base-context, skills, or rules.

## Contract

Built on the `agent-client-protocol` crate.

**Why ACP is the only interface, and what that retires:**

- The PTY is gone from the agent surface. No xterm.js, no `Channel<&[u8]>` behind an agent pane, no
  terminal keyboard handling on the agent path. An agent renders as events, not as raw terminal bytes.
- ACP telemetry is the single source. The `hooks` / `stream-json` / `session-log` capture paths no
  longer feed the agent surface. `[telemetry].source` collapses to `acp` for every supported harness,
  with a Locus-side mapping authored where a harness has no native ACP mode.
- **`mcpServers` is passed empty. Always.** No MCP servers, ever.

**`mcpServers` is passed empty. Always.** No MCP servers, ever — the agent-facing surface is the
`locus` CLI over a socket.

**Every agent runs in a container over ACP.** ACP is stdio, and stdio attaches to a container process
as readily as a host one. Running agents on the host would give them your real filesystem and your real
credentials, which is precisely the exposure the container model exists to remove. **ACP is not a
reason to leave the sandbox.**

**Normalization is shared.** `session/update` notifications map to the canonical vocabulary through
**one mapping for every ACP harness**, not one per harness:

| ACP | Verb |
| --- | --- |
| `AgentMessageChunk` | `assistant` |
| `AgentThoughtChunk` | `thinking` |
| `ToolCall` / `ToolCallUpdate` | `tool_call` / `tool_result` / `tool_error`, by its `status` |
| `RequestPermission` | `permission_request`, interpreted from the run's posture |

## Acceptance

 1. `session/new`, `session/prompt`, and streamed `session/update` work against a real ACP agent.
 2. `mcpServers` is empty on every call — asserted, since it is the kind of thing a future edit adds
    "just for testing".
 3. The ACP agent runs **in a container**, not on the host — a test asserts the process's namespace.
 4. `session/update` notifications normalize through the shared mapping, and a second ACP harness needs
    no new mapping code.
 5. **An ACP run is the run.** ACP events are the only conversation Locus has with an agent; there is no
    terminal stream to cross-check against.
 6. A tool-call permission request surfaces as `permission_request`: it raises the misconfiguration
    alarm for a bypass run, and becomes a resolvable human-action request for a gated run.
 7. **No PTY is attached to any agent run.** A test asserts the agent process has no terminal
    attached beyond what stdio requires — sealed at the container.
 8. A harness with no native ACP mode still fronts an ACP surface through a Locus-side mapping, and that
    mapping is registered per harness, not assumed zero.

## Open

- Whether the planning conversation is a distinct pane or the same event-rendered Agent Pane. With the
  PTY gone, both are events; the distinction is layout, not transport, and is now the pane-manager's
  call rather than ACP's.
