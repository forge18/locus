# ui-primitives — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Add Kobalte; confirm no shadcn package is added as a dependency | — | `bash apps/desktop/scripts/check-no-shadcn-dep.sh` |
| 2 | `Button` — outlined primary, secondary, ghost, block | 1 | `pnpm -C apps/desktop test -- ui/button` |
| 3 | Assert primary renders outlined rather than filled | 2 | `pnpm -C apps/desktop test -- ui/button-outlined` |
| 4 | `Tag` with `outline` and `neutral` variants, mono content, min-width on neutral | 1 | `pnpm -C apps/desktop test -- ui/tag` |
| 5 | `Card` with `selected` applying the accent inset ring over `--sf2` | 1 | `pnpm -C apps/desktop test -- ui/card-selected` |
| 6 | `Table` with mono right-aligned numeric columns by column type | 1 | `pnpm -C apps/desktop test -- ui/table-numerics` |
| 7 | `Table` skeleton state wired to `design-system`'s `<SkeletonRows>` | 6 | `pnpm -C apps/desktop test -- ui/table-skeleton` |
| 8 | `Input` and `Textarea` on `--sf` | 1 | `pnpm -C apps/desktop test -- ui/input` |
| 9 | `Segmented` control with accent active segment | 1 | `pnpm -C apps/desktop test -- ui/segmented` |
| 10 | `Tooltip` over Kobalte | 1 | `pnpm -C apps/desktop test -- ui/tooltip` |
| 11 | `Sheet` over Kobalte Dialog — opens over the current category, never a new window | 1 | `pnpm -C apps/desktop test -- ui/sheet-in-place` |
| 12 | `Tabs`, `ContextMenu`, `Combobox`, `Toast` styled to tokens | 1 | `pnpm -C apps/desktop test -- ui/kobalte-styled` |
| 13 | Assert no literal hex in `src/ui/` | 2,4,5,6,8,9,12 | `bash apps/desktop/scripts/check-no-raw-hex.sh apps/desktop/src/ui` |
| 14 | Assert `src/ui/` contains no split-pane, tree, or virtual-list component | 13 | `bash apps/desktop/scripts/check-ui-scope.sh` |
