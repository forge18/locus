# review-qa

**Milestone** M0.7 · **Depends on** `design-revision`, `shell-revision` · **Blocks** M6 automation surfaces

## Purpose

Review's landing view is no longer Telemetry — it is QA: whether the project's tests, linters, LSP
diagnostics, and agent reviews are currently passing. It answers "is this project healthy right now",
not "walk me through what happened", which stays Analytics → Telemetry's job.

Findings are the durable output of a scheduled or manual check run, never a live stream — a run
replaces its check source's previous result set, it does not merge with it. Sending a finding to the
inbox tracks it as a to-do; it does not move the finding or take it off this screen. The four groups —
Unit tests, Linters, LSP diagnostics, Agent reviews — are pluggable check sources, not four hardcoded
implementations: QA aggregates over `.specs/linters`, `.specs/lsp`, and `.specs/agent-prs` without
duplicating any of the three.

## Governed by

- `docs/UI_MOCKUP_REVIEW.md` §Review — QA (new surface) — the source contract for this screen
- PLAN.md §The Workflow Canvas — `Verify` as the runnable success criterion; "reviewer agents need no
  special machinery"
- PLAN.md §The user inbox — every item resolves to something it is about; silence is the default
- PLAN.md §Agents need real tools, not just a shell — `locus lsp`
- PLAN.md §M6 — Automation and discoverability — cron → workflow, overlap skipped rather than queued
- `.specs/linters/spec.md` — `locus lint`, the tool-facing surface the Linters group projects
- `.specs/lsp/spec.md` — `locus lsp diagnostics`, and unsupported reported distinct from empty
- `.specs/agent-prs/spec.md` — the self-review pass the Agent reviews group reuses

## Contract

### Project scoping

QA follows the selected project, unlike Analytics, which ignores the project selector entirely.
Switching the project switcher reloads all four groups against the newly selected project; nothing on
this screen is cross-project.

### Schedule and manual Refresh

A schedule control offers exactly **Manual, Push, Hourly, Daily**, persisted per project and defaulting
to Manual. **Refresh** runs a check immediately regardless of the selected schedule — it is not gated
by, and does not reset, the schedule. Hourly and Daily reuse the same cron → workflow mechanism as
`.specs/schedules`: overlap is skipped, never queued, so a check that is still running when its next
firing arrives records that firing as skipped rather than queuing behind it. Manual Refresh and every
scheduled firing call the same run entry point — there is no second, schedule-only code path.

### Check sources — four groups, pluggable

| Group | Tool attribution |
| --- | --- |
| Unit tests | vitest · cargo nextest |
| Linters | clippy · eslint · ruff |
| LSP diagnostics | rust-analyzer · tsserver |
| Agent reviews | reviewer@2 · custom prompt |

Each group is a registered check-source descriptor — name, tool-attribution label, kind, adapter —
not a `match` arm on the group's name. This is the same convention `harness-registry` and `.specs/lsp`
already carry: nothing in `crates/locus-core` names a specific tool inside a branch keyed on it. Adding
a fifth group is a descriptor plus an adapter function; it touches no existing adapter.

- **Unit tests** has no `warn` severity — a test passes or it fails.
- **Linters** projects `services::lint`'s `LintReport` (`.specs/linters/spec.md`) one-for-one into
  findings; QA does not re-implement linter discovery or execution, only reads its result.
- **LSP diagnostics** calls `locus lsp diagnostics` (`.specs/lsp/spec.md`) per language in the project.
  A verb a server does not support surfaces as a `warn` finding naming the gap — never as an empty,
  passing result, per lsp's unsupported-vs-empty rule.
- **Agent reviews** runs the self-review pass already defined in `.specs/agent-prs/spec.md` — the same
  function, not a second implementation — with findings tagged by the reviewing agent id and its custom
  prompt.

Each group renders its tool-attribution line and a pass/fail/warn summary count alongside its findings.

### Finding shape

A finding carries: severity (`fail` or `warn` — exactly one of the two, never neither), a title, the
project and a location, and a one-line explanation. Every finding belongs to the check run that
produced it; a run's findings for a check source atomically replace that source's previous result set,
so no finding from run *n − 1* is ever visible beside one from run *n* for the same source.

### Send to Inbox

Each finding carries a **Send to Inbox** / **Sent to Inbox** toggle. Sending creates an inbox item
whose locator resolves to the finding — the same "every item resolves to something" contract the inbox
already carries (PLAN.md §The user inbox) — and does **not** remove, hide, or mutate the finding row.
The finding stays listed on this screen regardless of what happens to the inbox item: resolving,
dismissing, or otherwise closing the inbox item never clears the finding. Only a later check run whose
result set omits the finding removes it from QA.

### Footer

Rendered verbatim: "Not real-time — findings reflect the last scheduled or manual run. Sending a
finding to Inbox tracks it as a to-do; it stays listed here too."

## Supersedes

| Existing surface | Replacement |
| --- | --- |
| Review's landing view was Telemetry (`screens-review`, M0.5, historical) | Review's landing view is now `qa`, this spec |
| Telemetry as a tab under Review | moves under Analytics, as the Analytics → Telemetry sub-tab (`design-revision` §Screen inventory) |

This is a new surface — nothing in `.specs/` describes QA before this spec. `screens-review` is
already marked historical and superseded by `design-desktop`; this table records QA's relationship to
it without editing that file.

## Acceptance

1. Switching the selected project reloads all four groups; Analytics's project-independence is
   unaffected.
2. The schedule control offers exactly Manual, Push, Hourly, Daily, persisted per project, defaulting
   to Manual.
3. Refresh triggers a check run regardless of the selected schedule value.
4. Manual Refresh and a scheduled firing invoke the same run entry point — asserted by shared code
   path, not by similar behavior.
5. An Hourly or Daily check still running when its next firing arrives records that firing as skipped,
   never queued.
6. A check run's findings for one check source atomically replace that source's previous result set —
   no finding from the prior run for that source is visible after the new run lands.
7. Each of the four groups renders its tool-attribution line and a pass/fail/warn summary.
8. A finding always carries exactly one of `fail` or `warn`, a title, a project and location, and a
   one-line explanation.
9. The Linters group's findings match `services::lint::LintReport` results one-for-one.
10. An LSP verb a server does not support surfaces as a `warn` finding, distinguishable from an empty
    diagnostics result.
11. The Agent reviews group calls the same self-review function `.specs/agent-prs` defines — asserted
    by shared implementation.
12. Send to Inbox creates an inbox item resolving to the finding's locator; the finding remains listed
    in QA unchanged.
13. Resolving or dismissing the resulting inbox item does not remove the finding from QA.
14. Adding a fifth check-source group requires no edit to an existing adapter and no new `match` arm on
    a group name in `crates/locus-core`.
15. The footer text renders verbatim in every group's populated and empty state.

## Open

- What **Push** fires on. The mockup names it as a schedule option alongside Hourly and Daily but does
  not say whether it means a push to any branch in any project repo, or only to the run's own branch —
  `.specs/repo-manager`'s local-remote model is the likely home for the answer, not this spec.
- Whether a manual Refresh issued while a scheduled run is already in flight for the same check source
  is skipped like an overlapping cron firing, or preempts it. `.specs/schedules` settles overlap for
  cron; it does not say whether a manual trigger is "another firing" for this purpose.
- Whether the reviewer agent and its custom prompt are configured per project or per check source when
  a project runs more than one Agent reviews configuration. The mockup shows one (`reviewer@2`) and
  does not say whether a second is possible.
