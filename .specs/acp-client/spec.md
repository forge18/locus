# acp-client

**Milestone** M1 · **Depends on** `sandbox`, `telemetry` · **Blocks** `planning-module`

## Purpose

An Agent Client Protocol client, **for the planning/chat module only**. ACP fits a conversation:
`session/new`, `session/prompt`, streamed updates, tool-call permission requests — exactly what spec
work and questions need.

The scope limit is the point of this spec. PLAN.md gives two reasons ACP is not how agent sessions run,
and both are structural rather than preferences.

## Governed by

- PLAN.md §ACP — the planning module only
- PLAN.md §Canonical event vocabulary — the `acp` source row
- PLAN.md ADR-worthy decision: no system-prompt control, and a conversation hides the work

## Contract

Built on the `agent-client-protocol` crate.

**Why ACP is not how agent sessions run, stated so it is not re-litigated:**
1. `session/new` accepts only `cwd` and `mcpServers`, so **a client cannot inject a system prompt or
   any instructions** — prompt assembly belongs entirely to the harness.
2. A conversation abstraction hides the thing terminals exist to show, which is the agent working.

**`mcpServers` is passed empty. Always.** No MCP servers, ever — the agent-facing surface is the
`locus` CLI over a socket.

**The planning agents still run in containers.** ACP is stdio, and stdio attaches to a container
process as readily as a host one — the only difference from a terminal run is that there is no PTY.
Running them on the host would give the researcher your real filesystem and your real credentials while
it indexes repos, which is precisely the exposure the container model exists to remove. **A
conversation is not a reason to leave the sandbox.**

**Normalization is shared.** `session/update` notifications map to the canonical vocabulary through
**one mapping for every ACP harness**, not one per harness:

| ACP | Verb |
| --- | --- |
| `AgentMessageChunk` | `assistant` |
| `AgentThoughtChunk` | `thinking` |
| `ToolCall` / `ToolCallUpdate` | `tool_call` / `tool_result` / `tool_error`, by its `status` |
| `RequestPermission` | `permission_request` |

## Acceptance

1. `session/new`, `session/prompt`, and streamed `session/update` work against a real ACP agent.
2. `mcpServers` is empty on every call — asserted, since it is the kind of thing a future edit adds
   "just for testing".
3. The ACP agent runs **in a container**, not on the host — a test asserts the process's namespace.
4. `session/update` notifications normalize through the shared mapping, and a second ACP harness needs
   no new mapping code.
5. Events from an ACP run are indistinguishable downstream from a hooks run.
6. A tool-call permission request surfaces as `permission_request` and raises the misconfiguration alarm.
7. No code path lets ACP start an ordinary agent session.

## Open

- Whether the planning conversation gets a PTY-less pane type of its own or reuses the Agent Pane with
  the terminal suppressed. The handoff draws the Plan screen as a conversation, which suggests its own.
