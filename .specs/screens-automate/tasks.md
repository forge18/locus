# screens-automate — tasks

Open question carried from the spec: the second column is "Building" in the handoff and "In Progress"
in PLAN.md. Task 3 uses one label and records which.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `<KanbanView>` header with the fixed-columns and blocked-is-a-status notes | — | `pnpm -C apps/desktop test -- kanban/header` |
| 2 | Project `.tag-neutral` chips, right-aligned | 1 | `pnpm -C apps/desktop test -- kanban/project-chips` |
| 3 | Six-column grid at 9px gaps, labels and counts; record the second column's label | 1 | `pnpm -C apps/desktop test -- kanban/six-columns` |
| 4 | Assert no add, remove, or reorder affordance exists | 3 | `pnpm -C apps/desktop test -- kanban/columns-fixed` |
| 5 | Accent label on the Waiting For Approval column head | 3 | `pnpm -C apps/desktop test -- kanban/approval-accent` |
| 6 | `<TaskCard>` with title and the 10px meta line | 3 | `pnpm -C apps/desktop test -- kanban/card` |
| 7 | Blocked variant: red `prohibit-inset` shown in place | 6 | `pnpm -C apps/desktop test -- kanban/blocked-in-place` |
| 8 | Assert a blocked card can appear in any column | 7 | `pnpm -C apps/desktop test -- kanban/blocked-is-orthogonal` |
| 9 | Stuck, waiting-approval and done card variants | 6 | `pnpm -C apps/desktop test -- kanban/card-variants` |
| 10 | `<AgentsView>` split at 356px | — | `pnpm -C apps/desktop test -- agents/layout` |
| 11 | Session list header with count, funnel and accent sort | 10 | `pnpm -C apps/desktop test -- agents/list-header` |
| 12 | `<SessionCard>`: status dot, project, mono agent, role, tokens, task, chip, tool, runs | 10 | `pnpm -C apps/desktop test -- agents/session-card` |
| 13 | Selected and stuck card states | 12 | `pnpm -C apps/desktop test -- agents/card-states` |
| 14 | Sort needs-attention first, then activity | 12 | `pnpm -C apps/desktop test -- agents/sort-order` |
| 15 | List footer with the session-you-stopped-watching note | 11 | `pnpm -C apps/desktop test -- agents/list-footer` |
| 16 | Transcript header with detach and minimize controls | 10 | `pnpm -C apps/desktop test -- agents/transcript-header` |
| 17 | `<Transcript>` at mono 11.5/1.68, colored by verb | 10 | `pnpm -C apps/desktop test -- agents/transcript` |
| 18 | Assert only the twelve canonical verbs are colorable | 17 | `pnpm -C apps/desktop test -- agents/canonical-verbs-only` |
| 19 | Prompt line with the 7x14px blinking block cursor | 17 | `pnpm -C apps/desktop test -- agents/cursor` |
| 20 | Stuck footer: guardrail card with both actions | 16 | `pnpm -C apps/desktop test -- agents/stuck-footer` |
| 21 | Waiting footer stating "Waiting ≠ idle" | 16 | `pnpm -C apps/desktop test -- agents/waiting-not-idle` |
| 22 | Assert the conditional footers are mutually exclusive and status-driven | 20,21 | `pnpm -C apps/desktop test -- agents/footer-exclusive` |
| 23 | Status bar with the PTY note and run id | 10 | `pnpm -C apps/desktop test -- agents/status-bar` |
| 24 | Session select swaps the right pane and leaves the others running | 12,17 | `pnpm -C apps/desktop test -- agents/select-does-not-close` |
| 25 | Minimize sends the session to the strip without ending it | 16 | `pnpm -C apps/desktop test -- agents/minimize-to-strip` |
| 26 | Detach stub asserting a second window, never a second webview | 16 | `pnpm -C apps/desktop test -- agents/detach-is-a-window` |
| 27 | Visual check against `screenshots/05-automate-kanban.png` and `06-automate-agents.png` | 9,26 | `pnpm -C apps/desktop test -- visual -- board sessions` |
