# screens-dashboard

**Milestone** M0.5 · **Depends on** `app-shell`, `navigation`, `fixtures` · Views `inbox`, `status`

## Purpose

The two views of the category that is *mine*: what I need to do, and what I need to know. PLAN.md
names Dashboard the one category defined by whose it is rather than what it holds, and gives it a rule
the whole design leans on — **a decision resolves in place; work routes out.**

## Governed by

- PLAN.md §The user inbox — silence is the default
- PLAN.md §Navigation — Dashboard is mine; Dashboard is now, Review is after
- `docs/design_handoff_locus_desktop_ui/README.md` screens 1 and 2

## Contract

### Inbox (default view)

Two panes. **Left 392px**, right hairline.
- `NEEDS YOU` in accent + `N items · silence is the default`. Cards at radius 7: selected is `--sf2` +
  accent inset ring with a `seal-check` fill icon, age right-aligned in `--mu2`, and a subline of
  project · agent · mono branch. Others are `--sf` + hairline — a `question` icon for `locus ask`, a
  `warning-octagon` in `--bad` for a guardrail trip.
- `RESOLVED TODAY`, three rows at `opacity:.6`.
- **Right pane**: accent kind tag + 15px title, a metadata row (mono locator · agent · role · gate),
  then the item body. For a plan gate: an accent `PLAN` label over numbered steps at 12.5px/1.6 with
  mono inline paths, an `info` callout on `--sf`, and a 64px textarea under "Comment steers the agent
  that made it".
- **Footer bar** (`--bg-deep`, top hairline): primary "Approve & release the loop", secondary "Send
  back with comment", right note "Resolves here · the work opens where the work lives".

**The behavioral rule, which is the point of the screen:** approving **resolves in place** and releases
the agent loop. The item's *work* opens where that work lives — Plan, Develop, or Review — via the
locator, never by growing a second copy of that surface here.

**Silence is the default.** A session working normally puts nothing here. An item that only reports
that something happened is a notification, not inbox work, and does not belong.

### Status

Scrolling column, 15/18px padding, 14px gaps.
- **Six metric cards** in a 6-col grid: Running (`N panes · N strip`), **Waiting on me** as the accent
  card (`--sf2` + accent ring, accent label and numeral, `oldest 26m`), Verify pass %, Cache read %,
  Tokens today, Guardrail trips with `1 kill & reassign` in `--bad`. Numerals 27px/500, unit suffixes
  15px `--mu`, labels 10px uppercase .1em.
- **Runs by hour** (1.55fr): 12 stacked bars, 118px tall, 5px gaps — accent passed, `--bad` failed,
  `--blue-lit` aborted, stacked bottom-up, mono hour axis.
- **Wants attention** (1fr): stuck (red inset ring, `warning-octagon` fill, accent `Reassign` link),
  idle (`moon`), waiting (`hourglass-medium`, **"waiting: gate — not idle"**).
- **Project table**: Project / Repos / Running / In review / Verify / Tokens today / Cache / Last event.
  Verify colored `--ok`/`--bad`, numerics mono.

**Status does not grow a query tool.** Digging into a run that went wrong is Review's job; keeping them
apart is what stops Status becoming a second Review and Review growing a live view nobody watches.

## Acceptance

1. Inbox with no items renders "Nothing needs you", not an empty list and not a spinner.
2. Approving an item resolves it in place — the view does not change and the rail does not move.
3. The "open the work" action navigates by locator, and lands in Plan, Develop or Review.
4. The waiting row reads "waiting: gate — not idle" and is visually distinct from the idle row.
5. Runs-by-hour stacks bottom-up in the three states with the documented colors.
6. Cache read renders *unknown*, not 0%, for a project whose fixture has `usage: null`.
7. Status contains no search field, no filter chips, and no facet control.

## Open

- Whether "resolved today" should be a fixed window or a count. The handoff draws three rows and says
  nothing about overflow.
