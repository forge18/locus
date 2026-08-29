# bot-avatars

**Milestone** M6 · **Depends on** `bots`

## Purpose

Every bot gets a face: a DiceBear avatar derived from its id, never stored. Creation is free,
renaming keeps the robot, and nothing new persists anywhere — the avatar is a pure function of
(style, bot id) rendered in the webview. The style is one app-wide setting, defaulting to Bottts,
chosen from the styles shipped in the bundle, never fetched.

This is UI-only by construction: no Rust change, no migration, no agent-facing surface. Avatars are
for the human; agents never see one and nothing materializes one.

## Governed by

- PLAN.md §Model routing — settings policy lives in `core.settings`, not in code; `bots.avatar_style`
  is one more key there
- PLAN.md §UI components — the Settings control is a Kobalte/shadcn-solid select, not bespoke DOM
- `.specs/bots/spec.md` — the bot list row ("live dot, name, harness, and last activity") gains the
  avatar; the Agent Pane stays **unmodified**; avatars live in bots screen chrome, never panel code
- `.specs/design-revision/spec.md` — no new views; the bots view's row anatomy changes and header
  chrome is added inside the existing two-pane screen
- `.specs/theme-system/spec.md` — avatars must read on both themes; rendered with a transparent
  background under the `data-theme` contract

## Contract

### Derived, never stored

The avatar is `createAvatar(style, { seed: bot_id })` from `@dicebear/core` and
`@dicebear/collection`, rendered as a data-URI `<img>`. Same id and style render byte-identically —
the library's determinism, asserted by test. The seed is the **bot id, not the name**: renaming a
teammate keeps its robot; delete-and-recreate with a new id gets a new one. There is no column, no
frontmatter field, no file on disk, no IPC; the only input is the id already on the row.

### One style setting

`bots.avatar_style` in `core.settings`, default `bottts`, app-wide — not per project; a bot's look is
not project state. Settings gains one single-select listing every style shipped in
`@dicebear/collection`, each entry showing creator and license from the collection's own metadata.
Changing it re-renders every bot immediately — nothing is stored, so there is nothing to migrate and
nothing to restart. Styles ship in the JS bundle; the app never calls `api.dicebear.com`.

**Attribution is a license requirement, not decoration.** Bottts is by Pablo Stanley under CC BY 4.0.
The Settings picker shows creator and license per style, and the active style's attribution stays
visible. The list is whatever the collection ships, attributed — adding a style whose license cannot
be satisfied is out of scope.

### Surfaces — list and header only

- **Bot list rows** (246px): the avatar joins live dot, name, harness, and last activity.
  **Collapsed 40px strip**: the avatar stands in for the row, with live state as a ring or badge on
  it, still readable at a glance.
- **Bot view header**: the bots screen's own chrome above the right pane carries avatar, name, and
  harness. `panes/AgentPane.tsx` receives **no avatar code and no Bots-specific props** — the bots
  spec's "unmodified" clause holds.
- Nothing else changes. The rail category icon, bot pickers in schedules or dispatch, and
  workflow-canvas nodes are out of scope. The helper is written so a later surface can call the same
  function with any stable id, but none of them are drawn here.

### Rendering rules

Transparent background so one avatar reads on Dark and Light alike. Generated strings are memoized
**in memory** per `(style, seed)` inside the helper, so a list render generates each robot once;
the memo lives in the webview session and is never persisted — regeneration is deterministic and
cheap, so the cache is an optimization, never a correctness mechanism. Sized per surface (row,
strip, header) from one helper. No Tauri IPC, no Rust, no Postgres.

## Supersedes

Nothing. The bots view contract gains row anatomy and header chrome; the view count stays thirty.

## Acceptance

1. A bot's avatar is a pure function of bot id and the style setting; same id and style render
   byte-identically.
2. Renaming a bot never changes its avatar; a new id gets a new robot.
3. With no setting present, every bot renders in Bottts.
4. Settings lists the shipped styles with creator and license per entry; choosing one re-renders all
   bots immediately.
5. The active style's creator and license are visible in Settings.
6. List rows and the collapsed strip show the avatar; live state remains readable.
7. The bot view header shows avatar, name, and harness; `AgentPane.tsx` contains no avatar or
   Bots-specific code.
8. Avatars read correctly under both `data-theme` values.
9. No schema, store, or materializer change exists anywhere for this feature.

## Open

- Whether other named-agent surfaces (Interact list, canvas nodes, pickers) adopt the same derivation
  later — deliberately undrawn; the helper accepts any stable id.
- Whether the style picker previews a live sample per style — nice, not required; the metadata list
  is the contract.
