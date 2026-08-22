# agent-interface — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Project the session, run, task, workflow, and permission posture into one Agent Pane view model | `run-supervisor`, `v2-project-operations` | `cargo test -p locus-core agent_interface::view_model` |
| 2 | Mount the panel as a flex column: stream is the only flexible region; header, blockers, plan dock, and composer remain visible | 1 | `pnpm -C apps/desktop test -- agent-panel/layout` |
| 3 | Render the 44px header with project handle, optional task/workflow chips, editable session name, cost toggle, and research control | 2 | `pnpm -C apps/desktop test -- agent-panel/header` |
| 4 | Derive the live pill from run progress plus pending gate/elicitation; clicking it finds the active blocker | 1 | `pnpm -C apps/desktop test -- agent-panel/live-status` |
| 5 | Render collapsible user prompts with copy metadata and resubmission | 2 | `pnpm -C apps/desktop test -- agent-panel/user-card` |
| 6 | Render agent turns with shared identity metadata and markdown, code, tables, images, and citations | 2 | `pnpm -C apps/desktop test -- agent-panel/agent-turn` |
| 7 | Render `agent_thought` as an independent Summary / Full / Hidden block, defaulting to Summary | 6 | `pnpm -C apps/desktop test -- agent-panel/thinking` |
| 8 | Render live, collapsible tool cards with running, complete, failed, and queued states | 6 | `pnpm -C apps/desktop test -- agent-panel/tool-card` |
| 9 | Render non-wrapping, horizontally scrollable diffs with line gutters and added/removed/context rows | 8 | `pnpm -C apps/desktop test -- agent-panel/diff` |
| 10 | Pin a citation from a turn into the session research feed | 6 | `pnpm -C apps/desktop test -- agent-panel/citation-to-research` |
| 11 | Open a rendered file path in an editor pane beside the Agent Pane | `pane-manager` | `pnpm -C apps/desktop test -- agent-panel/file-link` |
| 12 | Dock a pending blocker above the plan dock and composer; cap its height and scrim only the stream | 2 | `pnpm -C apps/desktop test -- agent-panel/docked-blocker` |
| 13 | Render a gated edit approval with inline diff, approve, decline, and approve-remaining-turn actions | 9,12 | `pnpm -C apps/desktop test -- agent-panel/permission-gate` |
| 14 | Render a restricted-schema elicitation form with validation, defaults, accept, decline, and cancel | `acp-client`,12 | `pnpm -C apps/desktop test -- agent-panel/elicitation` |
| 15 | Minimize a blocker to a one-line pill and restore it without losing its upstream request state | 12 | `pnpm -C apps/desktop test -- agent-panel/blocker-minimize` |
| 16 | Stack one gate and one elicitation safely; collapse the plan while either blocker is expanded | 13,14 | `pnpm -C apps/desktop test -- agent-panel/blocker-stack` |
| 17 | Render the current ACP plan in a collapsed or expanded dock, with step state and outcome metadata | `acp-client`,2 | `pnpm -C apps/desktop test -- agent-panel/plan-dock` |
| 18 | Send a prompt, queue it at a turn boundary while running, and replace Send with Stop for cancellation | `run-supervisor`,2 | `pnpm -C apps/desktop test -- agent-panel/composer` |
| 19 | Offer slash commands and `@` mentions for session actions, files, symbols, and tasks | 18 | `pnpm -C apps/desktop test -- agent-panel/composer-discovery` |
| 20 | Render the toggleable 380px session research pane, including seed/run/close provenance | `artifacts`,2 | `pnpm -C apps/desktop test -- agent-panel/research-pane` |
| 21 | Render stream checkpoint markers, Restore, restored banner, and Undo without truncating the transcript | `run-supervisor`,2 | `pnpm -C apps/desktop test -- agent-panel/checkpoints` |
| 22 | Apply disclosure and cost settings without withholding an available detail level | 3,7,8 | `pnpm -C apps/desktop test -- agent-panel/disclosure-settings` |
| 23 | Reset transient expansion, rename, and menu state on session switch without shadowing incoming state | 1 | `pnpm -C apps/desktop test -- agent-panel/session-switch` |
| 24 | Collapse research below 1100px and preserve the composer down to a 520px panel width | 2,20 | `pnpm -C apps/desktop test -- agent-panel/responsive` |
| 25 | Keyboard-test the composer, blocker controls, disclosure controls, and visible focus states | 13,14,18,22 | `pnpm -C apps/desktop test -- agent-panel/a11y` |
| 26 | Compare the shipped panel against the ACP panel handoff’s structural and visual contract | 3,4,9,16,17,20,21,24 | `pnpm -C apps/desktop test -- visual/agent-panel` |
