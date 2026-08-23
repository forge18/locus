# app-shell

> **Historical M0.5 contract.** The v1 four-band shell is superseded for new work by
> `.specs/design-desktop/spec.md` §Shell and screen inventory.

**Milestone** M0.5 · **Depends on** `design-system`, `ui-primitives` · **Blocks** every `screens-*`

## Purpose

The four bands present on every screen: title bar, category rail, per-category tab bar, and the
running-agent strip. This is the frame that makes one window hold every project — PLAN.md refuses a
window per project because that would rebuild the fragmentation the whole design exists to remove.

## Governed by

- `docs/design_handoff_locus_desktop/README.md` §Shell
- `.specs/design-desktop/spec.md`
- PLAN.md §Navigation — one window, project as a filter; the inbox count on the rail
- PLAN.md §Sessions do not all fit, so most are strips

## Contract

The window **fills its host**, `--bg`, column flex. The handoff's 1440x900 was the
size of the mockup's frame, not a constraint on the app — see `design-system`
§Layout fills its host. Every width below is a preference that flexes, not a
constant.

**1. Title bar** — 42px, `--bg-deep`, bottom hairline.

- macOS traffic lights (11px: `#ed6a5e`, `#f4bf50`, `#61c554`), then `LOCUS` at 12px/500 uppercase,
  letter-spacing .14em, `--mu`.
- Centered **locator bar**: `clamp(240px, 40vw, 520px)` x 26, `--sf`, radius 6, hairline. Magnifier, mono `locus://` in `--mu2`,
  then the current path in `--mu`, right-aligned `Cmd-K` in a 4px hairline box. This is the app's
  addressing surface, not decoration — see `navigation`.
- Project filter: `All projects · N` with a funnel icon in accent. **A filter, never a switcher.**
- Right: pulsing accent dot + `N running`.

**2. Category rail** — `clamp(68px, 6vw, 92px)`, `--bg-deep`, right hairline, 6px padding, 2px gaps. **Seven items**,
Phosphor 19px over a 9.5px label: Inbox (`tray`, badge), Plan (`compass`), Develop (`code`), Automate
(`lightning`), Review (`chart-bar`), Workshop (`wrench`), Wiki (`book-bookmark`). Active is `#293947` +
`inset 0 0 0 1px rgba(255,187,57,.55)` + accent text; inactive `--mu`. Rail foot: `git-branch` and
`user-circle` in `--mu2`.

The inbox badge is a 15px accent pill, text `#1d2731` at 700/9.5px, absolutely positioned top 5 right 9
— the one place weight goes above 500, because it is a pill and not text.

**The rail highlights by category, not by view.** Drilling into a sub-screen keeps its category lit.

**3. Tab bar** — 40px, `linear-gradient(var(--sf), var(--bg))`, bottom hairline. Left: the current
category label at 12px/500 uppercase .1em `--mu`. Then only the tabs belonging to that category (3px
10px, radius 6; active `#314454` + `inset 0 0 0 1px rgba(255,187,57,.5)` + `--tx`). Right: the mono
locator for the current view + `arrows-out-simple`.

**4. Strip** — 54px footer, `--bg-deep`, top hairline. A vertical `STRIP` label, then one compact card
per running agent: project · agent · role over status · tool · tokens. `--sf`, radius 7, hairline;
**red-bordered when stuck**; dimmed with a `terminal-window` icon for your own shell ("no agent · no
cost"). Right: `sorted by needs-attention, then activity`.

**Ordering is needs-attention first, then activity** — never by project and never alphabetically,
either of which would put the same session in the same place whether or not anything is happening.

**The strip persists across categories.** Leaving Automate is not a reason to lose sight of what is
running.

## Acceptance

1. All four bands render on all fourteen screens; none is per-screen markup.
2. The rail shows seven items and lights by **category** — opening Workshop Agent-definitions keeps
   Workshop lit, not a new entry.
3. The inbox badge shows a count and disappears at zero. Silence is legible from anywhere.
4. The tab bar shows only the current category's tabs; Plan, Develop and Wiki show none.
5. The strip sorts needs-attention first, then activity — asserted against a fixture where those two
   orders differ.
6. A stuck strip card carries a red border; a human shell card is dimmed and reads "no agent · no cost".
7. The project control filters and never navigates — selecting one project does not leave the view.

## Open

- Traffic lights are drawn for macOS. What the title bar does on Windows and Linux is undecided;
  nothing here blocks it, but the mockup only answers for one platform.
