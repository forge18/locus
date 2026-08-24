# TODO

**Remaining:** 32 features · 621 tasks.

This is the unfinished-work index. [`PLAN.md`](PLAN.md) is the architecture authority. Each feature's
`spec.md` is its contract; its `tasks.md` is the runnable decomposition and verification source.
Completed work is summarized as context, not repeated as a task list.

## How to use this file

Start with the execution order below. The feature specs contain the task details and runnable `verify:`
commands; this file keeps the dependency map, the decisions that still need answers, and the risks that
would otherwise be forgotten.

## Where to start

M1.5 is the current focus. Build session controls first, then dispatch permissions, then the Agent Pane.
The research feed is a cross-milestone dependency: it needs `memory` and `planning-module` before the
M1.5 surface can close. Rows 3–9 can proceed independently while that path is moving.

## Progress

| Milestone | Scope | Features | Tasks | State |
| --- | --- | ---: | ---: | --- |
| M0 | Spikes | 3 | 39 | closed |
| M0.5 | Desktop UI on fixtures | 12 | 268 | closed |
| M0.6 | Desktop reconciliation | 6 | 195 | closed |
| M0.7 | Current desktop mockup reconciliation | 11 | 338 | closed |
| M1 | Core runtime | 14 | 270 | closed |
| M1.5 | ACP agent panel and controls | 4 | 44 | active |
| M2 | Workspace | 3 | 57 | queued |
| M3 | Coordination, memory, and mail | 6 | 133 | queued |
| M3.5 | Agent capabilities: debug and browser | 3 | 56 | queued |
| M4 | Workflow canvas | 3 | 69 | queued |
| M5 | Project management | 5 | 127 | queued |
| M6 | Automation and discoverability | 3 | 46 | queued |
| M7 | Forge providers | 4 | 78 | queued |
| M8 | Marketplace installer | 1 | 11 | queued |
| **Total** | | **78** | **1,731** | **456 remaining** |

## Settled foundations

These are decisions, not unfinished tasks. They stay here because they explain the shape of the active
work without reopening completed milestones.

- The three M0 spikes answered their primary questions: CodeMirror 6, the host credential proxy, and
  `solid-flow`. Their evidence is in the [editor](spikes/02-editor-embed/FINDINGS.md),
  [sandbox](spikes/01-sandboxed-harness/FINDINGS.md), and [canvas](spikes/03-workflow-canvas/FINDINGS.md)
  findings.
- M0.7's current desktop mockup reconciliation and M1's core runtime are complete. Their feature rows
  are intentionally absent; the specs remain the historical contract.
- ACP is the only agent-session transport. Active work assumes one ACP conversation per container and
  does not add a PTY-based agent surface. See [acp-client](.specs/acp-client/spec.md).
- Security hardening is recorded in [.specs/security](.specs/security/spec.md): per-project forwarding
  egress, channel-owned context, boundary redaction, and socket-boundary ownership checks.

## Execution order

Start with items 1–2. Items 3–9 have only completed prerequisites and can run in parallel. The remaining
items follow their dependency edges. Items 31–32 are intentionally last because they depend on later
planning and memory features.

- [x] **1. M1.5 — [agent-session-controls](.specs/agent-session-controls/spec.md)** · [tasks](.specs/agent-session-controls/tasks.md) · 11 tasks
  ACP plan, elicitation, steering, commands, checkpoints, and replay.
  *Depends on:* completed M1 runtime

- [x] **2. M1.5 — [agent-dispatch-permissions](.specs/agent-dispatch-permissions/spec.md)** · [tasks](.specs/agent-dispatch-permissions/tasks.md) · 3 tasks
  Immutable per-job bypass/gated permission posture.
  *Depends on:* `agent-session-controls`

- [x] **3. M2 — [editor](.specs/editor/spec.md)** · [tasks](.specs/editor/tasks.md) · 22 tasks
  CodeMirror workspace editor and real agent diffs.
  *Depends on:* completed desktop/editor spike
  *Platform tasks 18–20 were not runnable here; `check-webview-matrix.sh` records all three targets as untested, never passing.*

- [x] **4. M3 — [mail](.specs/mail/spec.md)** · [tasks](.specs/mail/tasks.md) · 20 tasks
  Agent-to-agent mail and human escalation.
  *Depends on:* completed store
  *Live socket routing remains on the explicit `UnroutedVerbs` seam; core mail, inbox, wait, projection, and CLI allowlisting are implemented. Live socket delivery tasks were skipped.*

- [x] **5. M3 — [memory](.specs/memory/spec.md)** · [tasks](.specs/memory/tasks.md) · 40 tasks
  Scoped facts, provenance, embeddings, promotion, and decay.
  *Depends on:* completed store, event store, telemetry, and materializers

- [x] **6. M3 — [repo-manager](.specs/repo-manager/spec.md)** · [tasks](.specs/repo-manager/tasks.md) · 17 tasks
  Bare remote, per-run clones, and merge-back.
  *Depends on:* completed sandbox and store

- [x] **7. M3 — [tool-compaction](.specs/tool-compaction/spec.md)** · [tasks](.specs/tool-compaction/tasks.md) · 16 tasks
  Compact tool output before it enters context.
  *Depends on:* completed materializers, artifacts, and telemetry

- [x] **8. M4 — [marketplace-index](.specs/marketplace-index/spec.md)** · [tasks](.specs/marketplace-index/tasks.md) · 15 tasks
  Resolve marketplace manifests and tool pins.
  *Depends on:* completed agent definitions

- [x] **9. M3.5 — [locus-browse](.specs/locus-browse/spec.md)** · [tasks](.specs/locus-browse/tasks.md) · 21 tasks
  Browser recordings as reviewable artifacts.
  *Depends on:* completed sandbox and artifacts
  *Task 16 is skipped: recording duration cap remains an open product decision; see `.specs/locus-browse/task-16-note.md`.*

- [ ] **10. M2 — [lsp](.specs/lsp/spec.md)** · [tasks](.specs/lsp/tasks.md) · 24 tasks
  Shared LSP navigation, diagnostics, and semantic tokens.
  *Depends on:* `editor`

- [ ] **11. M2 — [project-search](.specs/project-search/spec.md)** · [tasks](.specs/project-search/tasks.md) · 11 tasks
  Search symbols and files across project repos.
  *Depends on:* `editor`

- [ ] **12. M3 — [guardrails](.specs/guardrails/spec.md)** · [tasks](.specs/guardrails/tasks.md) · 23 tasks
  Safe idle detection, retry, reassignment, and gates.
  *Depends on:* `mail`

- [ ] **13. M4 — [workflow-engine](.specs/workflow-engine/spec.md)** · [tasks](.specs/workflow-engine/tasks.md) · 30 tasks
  Execute governed workflow graphs.
  *Depends on:* `guardrails`

- [ ] **14. M3.5 — [locus-debug](.specs/locus-debug/spec.md)** · [tasks](.specs/locus-debug/tasks.md) · 19 tasks
  Rust DAP client for agents.
  *Depends on:* `guardrails` and `marketplace-index`

- [ ] **15. M3.5 — [media-artifacts](.specs/media-artifacts/spec.md)** · [tasks](.specs/media-artifacts/tasks.md) · 16 tasks
  OCR and keyframe derivation for model review.
  *Depends on:* `locus-browse`

- [ ] **16. M3 — [handoffs](.specs/handoffs/spec.md)** · [tasks](.specs/handoffs/tasks.md) · 17 tasks
  Transfer ownership with a bounded handoff payload.
  *Depends on:* `mail` and `guardrails`

- [ ] **17. M4 — [workflow-canvas](.specs/workflow-canvas/spec.md)** · [tasks](.specs/workflow-canvas/tasks.md) · 24 tasks
  Author and validate workflow graphs.
  *Depends on:* `workflow-engine`

- [ ] **18. M5 — [board](.specs/board/spec.md)** · [tasks](.specs/board/tasks.md) · 23 tasks
  Fixed columns and evidence-gated completion.
  *Depends on:* `workflow-engine`

- [ ] **19. M6 — [schedules](.specs/schedules/spec.md)** · [tasks](.specs/schedules/tasks.md) · 12 tasks
  Turn cron schedules into recorded workflows.
  *Depends on:* `workflow-engine`

- [ ] **20. M6 — [dashboard-metrics](.specs/dashboard-metrics/spec.md)** · [tasks](.specs/dashboard-metrics/tasks.md) · 20 tasks
  Query telemetry, spend, cache, and run metrics.
  *Depends on:* `workflow-engine`

- [ ] **21. M5 — [wiki](.specs/wiki/spec.md)** · [tasks](.specs/wiki/tasks.md) · 26 tasks
  Ingest typed knowledge and generate a durable wiki.
  *Depends on:* `workflow-canvas`

- [ ] **22. M5 — [task-orchestration](.specs/task-orchestration/spec.md)** · [tasks](.specs/task-orchestration/tasks.md) · 18 tasks
  Make Automate task-centric.
  *Depends on:* `board` and `workflow-engine`

- [ ] **23. M5 — [planning-module](.specs/planning-module/spec.md)** · [tasks](.specs/planning-module/tasks.md) · 40 tasks
  Guided planning with research, audit, and approval.
  *Depends on:* `board` and `wiki`

- [ ] **24. M5 — [calibration-loop](.specs/calibration-loop/spec.md)** · [tasks](.specs/calibration-loop/tasks.md) · 20 tasks
  Learn from failures through specialization.
  *Depends on:* `workflow-engine`, `planning-module`, and `memory`

- [ ] **25. M6 — [command-palette](.specs/command-palette/spec.md)** · [tasks](.specs/command-palette/tasks.md) · 14 tasks
  Search wiki, code, tasks, and runs globally.
  *Depends on:* `project-search`, `wiki`, and `board`

- [ ] **26. M7 — [forge-providers](.specs/forge-providers/spec.md)** · [tasks](.specs/forge-providers/tasks.md) · 30 tasks
  Provider-neutral PR, CI, review, and issue integration.
  *Depends on:* `repo-manager` and `board`

- [ ] **27. M8 — [marketplace-installer](.specs/marketplace-installer/spec.md)** · [tasks](.specs/marketplace-installer/tasks.md) · 11 tasks
  Install pinned tools into agent images.
  *Depends on:* `marketplace-index`

- [ ] **28. M7 — [agent-prs](.specs/agent-prs/spec.md)** · [tasks](.specs/agent-prs/tasks.md) · 14 tasks
  Agent-authored provider change requests.
  *Depends on:* `forge-providers` and `locus-browse`

- [ ] **29. M7 — [ci-babysitter](.specs/ci-babysitter/spec.md)** · [tasks](.specs/ci-babysitter/tasks.md) · 14 tasks
  Bounded CI repair and escalation loop.
  *Depends on:* `forge-providers`, `agent-prs`, and `guardrails`

- [ ] **30. M7 — [external-work-items](.specs/external-work-items/spec.md)** · [tasks](.specs/external-work-items/tasks.md) · 20 tasks
  Import tracker work into the local task workflow.
  *Depends on:* `task-orchestration`, `forge-providers`, and `board`

- [ ] **31. M1.5 — [agent-session-research](.specs/agent-session-research/spec.md)** · [tasks](.specs/agent-session-research/tasks.md) · 4 tasks
  Session research feed and reviewed promotion.
  *Depends on:* `artifacts`, `memory`, and `planning-module`

- [ ] **32. M1.5 — [agent-interface](.specs/agent-interface/spec.md)** · [tasks](.specs/agent-interface/tasks.md) · 26 tasks
  One ACP Agent Pane for stream, gates, plan, and research.
  *Depends on:* `agent-session-controls`, `agent-dispatch-permissions`, and `agent-session-research`

## Open decisions

Only unresolved decisions remain here; the owning spec is the source of truth. Closed decisions from
completed milestones are omitted unless they still affect an active feature.

| Feature | Decision |
| --- | --- |
| [agent-definitions](.specs/agent-definitions/spec.md) | Resolve `harness: any` at save time or at run start. |
| [agent-interface](.specs/agent-interface/spec.md) | Panel density variants, permission label, always-available research CLIs, checkpoint retention, workflow provenance, and agent identity treatment. |
| [editor](.specs/editor/spec.md) | Which languages receive Lezer grammars at M2; the internal catalog must make that explicit. |
| [lsp](.specs/lsp/spec.md) | Implement semantic-token full, delta, and CodeMirror decorations; `@codemirror/lsp-client` provides none. |
| [project-search](.specs/project-search/spec.md) | Trigger `codanna` indexing on demand, on a schedule, or on git change. |
| [board](.specs/board/spec.md) | Choose the column-two label: **Building** or **In Progress**. |
| [guardrails](.specs/guardrails/spec.md) | Decide whether the idle window scales by `task_class`. |
| [handoffs](.specs/handoffs/spec.md) | Decide whether handoffs may cross projects; reassignment otherwise uses a new handoff session. |
| [mail](.specs/mail/spec.md) | Decide whether agents can mail the human directly or must use `locus ask`. |
| [memory](.specs/memory/spec.md) | Set the initial `importance` value before a memory is recalled. |
| [repo-manager](.specs/repo-manager/spec.md) | Define when a linked repo syncs: on demand, on a timer, or on git change. |
| [tool-compaction](.specs/tool-compaction/spec.md) | Choose one compaction threshold shared with `artifacts`. |
| [locus-browse](.specs/locus-browse/spec.md) | Set the recording duration cap alongside the 30-day media retention policy. |
| [locus-debug](.specs/locus-debug/spec.md) | Set the supported language-adapter boundary; the tail is not covered yet. |
| [media-artifacts](.specs/media-artifacts/spec.md) | Set the OCR-confidence threshold for falling back to the source image. |
| [marketplace-index](.specs/marketplace-index/spec.md) | Choose a vetted catalog or an open index ranked by usage; the installer shares this decision. |
| [workflow-canvas](.specs/workflow-canvas/spec.md) | Confirm that workflows cannot re-plan during execution; dynamic changes require a new authored workflow. |
| [workflow-engine](.specs/workflow-engine/spec.md) | Set the arbiter's bounded model budget. |
| [planning-module](.specs/planning-module/spec.md) | Decide how to turn “I don't know what I want yet” into a goal. |
| [wiki](.specs/wiki/spec.md) | Decide whether regenerating `overview` on every ingest remains affordable at scale. |
| [command-palette](.specs/command-palette/spec.md) | Define ranking across wiki pages, tasks, symbols, and runs. |
| [dashboard-metrics](.specs/dashboard-metrics/spec.md) | Define the cache-rate alert threshold and what counts as a long session. |
| [schedules](.specs/schedules/spec.md) | Define timezone and DST behavior for cron expressions. |
| [agent-prs](.specs/agent-prs/spec.md) | Define what “large” means for slicing a change. |
| [ci-babysitter](.specs/ci-babysitter/spec.md) | Choose an ordinary workflow or supervisor behavior. |

## Carry-forward from M0

The spikes answered their primary questions, but these checks were not exercised. They remain here
because they affect trust in later features; the detailed evidence is in the linked findings.

- [ ] **Telemetry:** observe real token `usage` from one live run. If a harness reports nothing, spend
  must remain `unknown`, never zero. See [Spike 1 findings](spikes/01-sandboxed-harness/FINDINGS.md).
- [ ] **Editor:** exercise real MergeView chunk revert, the three target webviews, and Cmd/IME behavior.
  Reviewing an agent's diff is the primary editor job. See [Spike 2 findings](spikes/02-editor-embed/FINDINGS.md).
- [ ] **Harness registry:** run `dsh` end to end; its model-selection mechanism is still unverified.
  The [registry spec](.specs/harness-registry/spec.md) records the boundary.

## Constraints discovered by the spikes

These are not generic backlog items; they are implementation constraints that must stay visible until
the owning feature absorbs them.

- [ ] **Workflow canvas:** use `@dschz/solid-flow`, not the unrelated `solid-flow` npm package. WebKit
  needs a `requestIdleCallback` polyfill, and `ViewportPortal` is broken in `@dschz/solid-flow@0.1.4`.
  See [Spike 3 findings](spikes/03-workflow-canvas/FINDINGS.md).
- [ ] **Editor/LSP:** semantic tokens are not supplied by `@codemirror/lsp-client`; Locus must implement
  full, delta, and CodeMirror decoration support. See [editor](.specs/editor/spec.md) and [lsp](.specs/lsp/spec.md).
- [ ] **Sandbox:** macOS/colima cannot bind-mount a Unix socket. The relay path is weaker than a mount,
  so the per-run nonce is a required authenticator, not optional defense in depth. See [Spike 1 findings](spikes/01-sandboxed-harness/FINDINGS.md).
