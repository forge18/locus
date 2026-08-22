# screens-automate

> **Historical M0.5 contract.** V2 separates project Automate from global Dispatch; new work follows
> `.specs/design-v2/spec.md`.

**Milestone** M0.5 · **Depends on** `app-shell`, `navigation`, `fixtures` · Views `board`, `sessions`

## Purpose

Where work is assigned and watched. Two tabs — **Kanban** then **Agents**, in that order. PLAN.md puts
sessions here because a session lives with the agent it belongs to, and puts the board here because
the board is what assigns them.

## Governed by

- PLAN.md §The board — six fixed columns, `blocked` as a status, the two gating rules
- PLAN.md §Sessions do not all fit, so most are strips
- PLAN.md §Workflow guardrails — waiting ≠ idle; kill and reassign at three stuck iterations
- `.specs/design-v2/spec.md` §Project, plan, and dispatch policy

## Contract

### Kanban (first tab)

Header: "Fixed columns across every project", `prohibit-inset` + "blocked is a status, not a column",
right-aligned project `.tag-neutral` chips.

**Six fixed columns** in a 6-col grid, 9px gaps: Ready · Building · Testing · Reviewing · Waiting For
Approval · Done. Column head is a 10px uppercase label (accent for Waiting For Approval) plus a count
in `--mu2`.

Cards: `--sf` + hairline, radius 7, 11.8px title, then a 10px meta line — accent project · repo, mono
verify command, `reviewer@2 · read-only tools`, `Gate: reviewer agent`. Variants:

- **blocked** — a red `prohibit-inset`, shown *on* the card wherever it sits
- **stuck** — red inset ring plus `stuck 3/3 · 102.3k`
- **waiting approval** — accent ring and the note "an inbox item, not a place to go looking"
- **done** — `opacity:.86` with `--ok` "evidence: 2 runs, 41 events"

**There is no add-column affordance, and that is the feature.** Columns are fixed across every project.

### Agents (second tab)

**Left 356px.** Header `AGENTS · N running · one session each` with funnel and accent sort icons. Cards
on `#22303c` radius 8: status dot, project 12px/500, mono agent, role, right-aligned mono tokens; a
task line at 11.5px/76% opacity; then a status chip, the mono current tool, and a right-aligned run
count. Selected is `#293947` + `inset 0 0 0 1px rgba(255,187,57,.55)`; stuck cards carry a red hairline.

Footer: "Sorted by needs-attention, then activity. **Selecting one does not close the others — a
session you stopped watching is not a session you ended.**"

**Right.** Header: dot, project, mono agent, role, truncated task, status chip, mono locator,
`arrows-out-simple` (detach to its own Tauri window) and `minus` (minimize to strip). Body is a mono
11.5px/1.68 event stream colored by verb — accent tool calls, `#8fb8d6` thinking, `--ok` pass, `--bad`
error — ending in a prompt line with a 7x14px blinking accent block cursor.

Conditional footers, driven by the session's status:

- **stuck** → red-tinted guardrail card: "kill & reassign after 3 stuck iterations", the handoff
  summary, and "Hand off to reviewer@2" / "Let it run"
- **waiting** → `--sf` card with `hourglass-medium` and **"Waiting ≠ idle."**

Status bar: mono "PTY attached from the host · one session per terminal" + the run id.

**Detach opens a second Tauri window, never a second webview.** PLAN.md is explicit: multiwebview is
behind an unstable flag; multi-window is ordinary.

## Acceptance

1. Exactly six columns render, with no affordance to add, remove, or reorder one.
2. A blocked card shows its status as an icon **without moving column** — proving blocked is orthogonal
   to progress.
3. Selecting a different session swaps the transcript, header and footer; the other sessions keep
   running in the list and the strip.
4. The stuck footer appears only for stuck sessions and the waiting footer only for waiting ones.
5. The waiting footer states "Waiting ≠ idle" — the two are never rendered the same.
6. Transcript lines are colored by verb, and only the twelve canonical verbs appear.
7. Session list sorts needs-attention first, then activity.
8. Minimize moves the session to the strip without ending it.

## Open

- The handoff draws the Kanban columns as Ready / Building / Testing / Reviewing / Waiting For Approval
  / Done, while PLAN.md §The board names the second column **In Progress**. Same column, two labels —
  one wins, and it should be settled before the board is wired at M5.
