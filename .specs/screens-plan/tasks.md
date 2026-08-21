# screens-plan — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Three-pane frame: 216 / flex / 296 | — | `pnpm -C apps/desktop test -- plan/layout` |
| 2 | Plan list with the three sections and their counts | 1 | `pnpm -C apps/desktop test -- plan/list-sections` |
| 3 | Selected plan card: accent ring, `circle-notch`, step label, project right | 2 | `pnpm -C apps/desktop test -- plan/list-selected` |
| 4 | Approved section dimmed with the `--ok` landed-tasks line | 2 | `pnpm -C apps/desktop test -- plan/list-approved` |
| 5 | List footer carrying the one-approval rule | 2 | `pnpm -C apps/desktop test -- plan/one-approval-rule` |
| 6 | `<Breadcrumb>` — eight steps, three states, current derived from data | 1 | `pnpm -C apps/desktop test -- plan/breadcrumb` |
| 7 | `<Message>` with mono-initial avatar, role caption, bubble at max 600px | 1 | `pnpm -C apps/desktop test -- plan/message` |
| 8 | Avatar grounds: `--blue` agents, `#5c4413` auditor | 7 | `pnpm -C apps/desktop test -- plan/avatar-grounds` |
| 9 | Human replies right-aligned on `--sf3` at max 560px | 7 | `pnpm -C apps/desktop test -- plan/human-reply` |
| 10 | `<ScopeDecision>` inline card with both actions | 7 | `pnpm -C apps/desktop test -- plan/scope-decision` |
| 11 | Assert the scope decision is inline, not a modal or gate | 10 | `pnpm -C apps/desktop test -- plan/scope-is-inline` |
| 12 | Auditor finding variant with the red-tinted border | 7 | `pnpm -C apps/desktop test -- plan/auditor-finding` |
| 13 | Live line: pulsing dot plus the current-activity sentence | 1 | `pnpm -C apps/desktop test -- plan/live-line` |
| 14 | Conversation footer: input, blinking accent caret, mono ACP label | 1 | `pnpm -C apps/desktop test -- plan/footer-input` |
| 15 | `DRAFT OUTPUTS` rail: spec, numbered tasks, tool chips with the outline variant | 1 | `pnpm -C apps/desktop test -- plan/draft-outputs` |
| 16 | `<Recommendation>` card: 21px confidence, open count, ratchet note | 15 | `pnpm -C apps/desktop test -- plan/recommendation` |
| 17 | Confidence renders its raising condition alongside the figure | 16 | `pnpm -C apps/desktop test -- plan/confidence-has-condition` |
| 18 | Approve button states the task count it would land | 16 | `pnpm -C apps/desktop test -- plan/approve-states-count` |
| 19 | Visual check against `screenshots/03-plan.png` | 18 | `pnpm -C apps/desktop test -- visual -- plan` |
