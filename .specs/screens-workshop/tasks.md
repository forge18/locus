# screens-workshop — tasks

Every count on Extensions and Plugins comes from the registry and trusted plugin manifests via the `fixtures` generator.
A literal in the source is a bug, not a shortcut — task 6 and task 26 assert it.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `<ExtensionsView>` header with the one-surface framing, search, and New | — | `pnpm -C apps/desktop test -- extensions/header` |
| 2 | 4x2 grid of type cards at radius 9 | 1 | `pnpm -C apps/desktop test -- extensions/grid` |
| 3 | Assert exactly eight type cards | 2 | `pnpm -C apps/desktop test -- extensions/eight-types` |
| 4 | Card footer: native vs downgraded counts, `--bad` when downgrades dominate | 2 | `pnpm -C apps/desktop test -- extensions/native-vs-downgraded` |
| 5 | The `agents` entry card: accent ring, pointer, accent `arrow-right` | 2 | `pnpm -C apps/desktop test -- extensions/agents-entry-card` |
| 6 | Assert every count is computed, no literal in the source | 4 | `bash apps/desktop/scripts/check-no-literal-counts.sh` |
| 7 | `RECENTLY EDITED` list with min-width type chips | 1 | `pnpm -C apps/desktop test -- extensions/recently-edited` |
| 8 | `MATERIALIZATION` card, amber hairline, byte-determinism note, three figures | 1 | `pnpm -C apps/desktop test -- extensions/materialization-card` |
| 9 | Adding a thirteenth TOML changes the figures with no source edit | 6,8 | `bash apps/desktop/scripts/check-counts-follow-registry.sh` |
| 10 | `<AgentDefsView>` 196px sidebar with the back link | — | `pnpm -C apps/desktop test -- agentdefs/sidebar` |
| 11 | Definition list with mono versions and the selected ring | 10 | `pnpm -C apps/desktop test -- agentdefs/list` |
| 12 | Sidebar footer: "Markdown plus a tool list. No canvas, no compile." | 10 | `pnpm -C apps/desktop test -- agentdefs/footer-note` |
| 13 | Header with filename, mono provenance, Diff and Save-as controls | 10 | `pnpm -C apps/desktop test -- agentdefs/header` |
| 14 | Assert the save action reads "Save as v5", never "Save" | 13 | `pnpm -C apps/desktop test -- agentdefs/save-is-a-version` |
| 15 | Frontmatter block with the accent left border and `#8fb8d6` keys | 10 | `pnpm -C apps/desktop test -- agentdefs/frontmatter` |
| 16 | Prose body at 13px/1.65, max 660px | 10 | `pnpm -C apps/desktop test -- agentdefs/prose` |
| 17 | Footer naming the materialization target and downgrade count | 10 | `pnpm -C apps/desktop test -- agentdefs/materialize-footer` |
| 18 | `<WorkflowView>` three panes at 180 / 890 / flex | — | `pnpm -C apps/desktop test -- workflow/layout` |
| 19 | Palette: seven draggable node chips with `Verify` marked `req` | 18 | `pnpm -C apps/desktop test -- workflow/palette` |
| 20 | Presets block with the expands-into-ordinary-nodes note | 19 | `pnpm -C apps/desktop test -- workflow/presets` |
| 21 | Canvas: 24px dot grid and the SVG edge layer with four arrow markers | 18 | `pnpm -C apps/desktop test -- workflow/canvas-grid-edges` |
| 22 | Node cards with tinted header strips and right-side state | 21 | `pnpm -C apps/desktop test -- workflow/nodes` |
| 23 | Edge label pills, the dashed loop-back edge, and the loop grouping rect | 21 | `pnpm -C apps/desktop test -- workflow/loop-visuals` |
| 24 | Zoom pill and the no-model-in-the-orchestration-path note | 21 | `pnpm -C apps/desktop test -- workflow/canvas-footer` |
| 25 | Inspector: expression builder, compiled-expression card, operand chips | 18 | `pnpm -C apps/desktop test -- workflow/inspector` |
| 26 | Assert operand chips come from the `Condition` operand list, with the Gate note | 25 | `pnpm -C apps/desktop test -- workflow/operands-are-columns` |
| 27 | Guardrails block: two steppers, a toggle, idle, wall-clock, budget, the notes | 25 | `pnpm -C apps/desktop test -- workflow/guardrails` |
| 28 | `<PluginsView>` header, subgroup legend, and Register action | — | `pnpm -C apps/desktop test -- plugins/header` |
| 29 | 4-col card grid, one card per registered plugin | 28 | `pnpm -C apps/desktop test -- plugins/grid` |
| 30 | Assert Pi and trusted user-plugin cards render, from the registry | 29 | `pnpm -C apps/desktop test -- plugins/dynamic-count` |
| 31 | Mechanism badges in their three plugin-kind variants | 29 | `pnpm -C apps/desktop test -- plugins/mechanism-badges` |
| 32 | 4-row model-tier grid with `high` in accent | 29 | `pnpm -C apps/desktop test -- plugins/tier-grid` |
| 33 | `↑ high` up-fallback marker; assert no down-fallback is ever rendered | 32 | `pnpm -C apps/desktop test -- plugins/fallback-is-up` |
| 34 | Capability bar colored per the registered plugin descriptor | 29 | `pnpm -C apps/desktop test -- plugins/capability-bar` |
| 35 | Red hairline on heavily downgraded cards; `--bad` count at 4+ | 34 | `pnpm -C apps/desktop test -- plugins/heavy-downgrade` |
| 36 | Footer: computed downgrade line and the `tui = false` rule | 28 | `pnpm -C apps/desktop test -- plugins/footer` |
| 37 | Assert the footer reports dynamic registered-plugin counts, computed | 36 | `pnpm -C apps/desktop test -- plugins/computed-counts` |
| 38 | Visual check against `screenshots/10`, `11`, `12`, `13` | 9,17,27,37 | `pnpm -C apps/desktop test -- visual -- extensions agents canvas plugins` |
