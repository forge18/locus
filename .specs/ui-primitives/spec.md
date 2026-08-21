# ui-primitives

**Milestone** M0.5 · **Depends on** `design-system` · **Blocks** every `screens-*`, `app-shell`

## Purpose

The chrome, and only the chrome. PLAN.md is explicit that the component library is deliberately small
because every large surface here is either bespoke or brings its own DOM — so this covers buttons,
tags, cards, tables, inputs and menus, and nothing that is actually product.

## Governed by

- PLAN.md §Frontend and IPC constraints — the surface/provider table; Kobalte ships no split panes and
  no tree, correctly, because both are product
- `apps/desktop/src/ui/README.md` — copied in and owned here, never version-locked
- `docs/design_handoff_locus_desktop_ui/README.md` §Design system — the Nocturne classes to map

## Contract

shadcn-solid components over Kobalte primitives, **copied into `src/ui/`** rather than depended on.
Each maps a Nocturne class the mockup uses:

| Component | Mockup class | Notes |
| --- | --- | --- |
| `Button` | `.btn` | primary is **outlined, not filled** — accent as line, per Nocturne |
| `Tag` | `.tag`, `.tag-outline`, `.tag-neutral` | mono content; `.tag-neutral` has a min-width for column alignment |
| `Card` | `.card` | radius 7-9, `--sf` + hairline; selected is `--sf2` + accent inset ring |
| `Table` | `.table` | numerics mono and right-aligned; skeleton rows from `design-system` |
| `Input` / `Textarea` | `.input` | `--sf` ground |
| `Segmented` | `.seg` | active segment in accent |
| `Tooltip` | — | Kobalte Tooltip |
| `Dialog` / `Sheet` | — | Kobalte Dialog. **Detail opens in place as a sheet**, per PLAN.md |
| `ContextMenu`, `Tabs`, `Combobox`, `Toast` | — | Kobalte, styled |

**Not here, deliberately:** resizable split panes, the file tree, the pane manager, the canvas, the
editor, terminals. Those are product and live in `panes/`, `workflow-canvas/`, and their screens.

## Acceptance

1. Every component listed exists in `src/ui/` as source, not as an import from a published package.
2. `package.json` gains no `shadcn-solid` dependency — Kobalte is the only runtime dep added.
3. Each component reads its colors from tokens; none contains a literal hex.
4. Primary `Button` renders outlined, not filled.
5. `Table` right-aligns and mono-sets numeric columns without per-screen styling.
6. No split-pane, tree, or virtual-list component appears in `src/ui/`.

## Open

- Whether long lists need `@tanstack/solid-virtual` at M0.5 or only once real row counts arrive. The
  Sessions table is drawn at 300 rows and Runs at 612, so fixtures can answer this rather than guessing.
