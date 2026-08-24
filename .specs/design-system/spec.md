# design-system

> **Historical M0.5 contract.** `design-desktop` and `theme-system` supersede its v1 handoff reference
> for new work. This remains the record of the completed fixture baseline.

**Milestone** M0.5 · **Depends on** none · **Blocks** every `screens-*`, `ui-primitives`, `app-shell`

## Purpose

One visual system, defined once, that every screen reads from. The design handoff ships final values —
this feature turns them into CSS custom properties and vendored assets, and rules on the four state
families the handoff explicitly says "were not drawn". Undecided states get re-invented per screen,
which is how a dense professional tool ends up looking like fourteen different apps.

## Governed by

- `docs/UI_MOCKUP_REVIEW.md` §Navigation, §Screens
- `.specs/design-desktop/spec.md` and `.specs/theme-system/spec.md`
- PLAN.md §Frontend and IPC constraints — the component library is deliberately small

## Contract

**Tokens on `:root`.** The grounds and the accent come over from the handoff table
verbatim. Secondary text, hairlines and the status pair **do not** — see *Contrast*
below.

```
--bg #1d2731   --bg-deep #151d25   --sf #22303c   --sf2 #293947   --sf3 #314454
--blue #083c5d   --blue-lit #0d5480   --ac #ffbb39
--tx #eef2f6   --mu rgba(238,242,246,.78)   --mu2 rgba(238,242,246,.62)
--line rgba(238,242,246,.14)   --line2 rgba(238,242,246,.24)
--ok #68ad91   --bad #df8a7d   --ok-solid #4fa07f   --bad-solid #d4614f
--fm 'JetBrains Mono', ui-monospace, Menlo, monospace
```

**Contrast: every text colour clears WCAG AA on every ground it sits on.** The
handoff's `--mu2` at `.34` measures **2.46:1** on `--sf3` and never exceeds 2.90:1
anywhere — it fails AA on every surface in the app, and it carries most of the
metadata. `--mu` at `.56` fails on `--sf2` and `--sf3`. `#4fa07f` and `#d4614f`
measure 3.77 and 3.18 on `--sf2`, where both are used for text.

The alphas are raised to the value that clears 4.5:1 across all five grounds, and
the status pair is lightened until it passes. `--mu2` stays a step below `--mu`, so
the hierarchy the handoff drew survives — it just no longer buys the step with
legibility. `--ok-solid` / `--bad-solid` keep the original hues for fills and rings,
where nothing has to be read. `scripts/check-contrast.sh` enumerates every
colour/ground pairing and fails below AA.

`--ac` is a theme token, not a constant — the mockup exposes `accent` as a tweakable prop and it stays
one.

**Type.** Inter body, weight 400 with **500 for every emphasis, never 600+**. Mono for every locator,
branch, path, model name, session id, table numeric, and code.

**The scale is a set of tokens with an 11px floor**, not the handoff's literals. The
handoff draws 9-27px and calls the density deliberate; at 9.5px — where two thirds of
the app's text sat — "dense" reads as unreadable on a real display. Every step is
lifted by about a quarter and the hierarchy is unchanged:

```
--t-micro 11   --t-label 12   --t-meta 13   --t-body 14   --t-row 15
--t-lead 16    --t-title 19   --t-metric-sm 22   --t-metric 26   --t-metric-lg 34
```

Sizes are tokens so the whole scale moves in one edit. A raw `px` font-size anywhere
outside `tokens.css` is refused by `scripts/check-type-scale.sh`.

**Geometry.** Radii 5-6px small controls, 7-9px cards, 11px window. Gaps 2/4/6/9/14px. Shadows only:
`0 8px 22px rgba(0,0,0,.45)` and `0 10px 26px rgba(0,0,0,.5)` on canvas nodes,
`0 0 0 1px rgba(238,242,246,.13), 0 30px 80px rgba(0,0,0,.6)` on the window.

**Selection is an inset ring** — `box-shadow: inset 0 0 0 1px var(--ac)` over `--sf2`. Never an outer glow.

**Assets are vendored, not linked.** Inter, JetBrains Mono, and Phosphor (regular + fill) ship in
`apps/desktop/src/assets/`. The mockup uses a CDN; a desktop app must work offline.

**Animation is two keyframes and nothing else** — `pulse` (2s, opacity 1→.25, live dots) and `blink`
(1.1s, hard on/off, text carets).

**The four undrawn state families, ruled here:**

| Family | Ruling |
| --- | --- |
| Interaction | hover lifts one surface step (`--sf`→`--sf2`), pressed goes one further (`--sf2`→`--sf3`), focus is `outline: 2px solid var(--ac); outline-offset: 2px` — never a browser default ring |
| Loading | **no spinner on the inbox** — silence is the default there. Tables get skeleton rows at the real row height so nothing reflows. Panes that stream get their content, not a placeholder |
| Empty | every empty pane states **a reason**, never "No items". "No agent has run today" and "Nothing needs you" are different sentences and mean different things |
| Error | inline on the surface that failed, in `--bad`, carrying what failed and what to do. Never a toast for something the user is looking at |

**Layout fills its host.** The handoff states a fixed 1440x900 with no responsive
behaviour — true of a picture of an app, wrong for one. The window fills what it is
given; the drawn pane widths become `clamp()` preferences rather than constants that
overflow a smaller window and strand whitespace in a larger one; card grids use
`auto-fit` with a `minmax` floor. Scrollbars are thin and appear on hover, because a
permanent chunky bar is what a layout that did not fit looks like.
`scripts/check-responsive.sh` refuses a pinned pane.

## Acceptance

1. Every ground and accent in the handoff table exists as a custom property on `:root`
   with the exact value given; each departure from it is present *and explained in
   `tokens.css`* — `scripts/check-tokens.sh` asserts both.
2. No screen hardcodes a hex value — a grep for `#` in `src/screens/` finds only fixture data.
3. Inter, JetBrains Mono, and Phosphor load with the network disabled.
4. Exactly two `@keyframes` blocks exist in the whole app.
5. Focus on every interactive element is the accent outline; no element shows a browser default ring.
6. Changing `--ac` alone re-themes the app — selection rings, live dots, active tabs, and metric
   numerals all move together.
7. Every text token clears WCAG AA against every ground it appears on, enumerated
   rather than sampled.
8. No `px` font-size exists outside the scale, and no step of the scale is below 11px.
9. Nothing is pinned to 1440x900; every pane width is a preference and every card grid
   reflows.

## Open

- Whether a light theme is ever wanted. The handoff is dark-only and Nocturne is a dark system; nothing
  here forecloses it, but nothing here builds for it either.
- Whether AAA (7:1) is worth reaching for on body text. `--mu` already clears it on
  every ground; `--mu2` and the status pair would need another lift, and that costs
  the tonal separation between the three text levels.
- The v1 screenshots were removed with the handoff. The desktop captures are the visual reference;
  new `visual --` checks conform to `.specs/design-desktop/` and `.specs/theme-system/`.
