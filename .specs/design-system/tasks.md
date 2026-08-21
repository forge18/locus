# design-system — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `src/styles/tokens.css` — every handoff token as a custom property on `:root` | — | `bash apps/desktop/scripts/check-tokens.sh` |
| 2 | Vendor Inter (400, 500) into `src/assets/fonts/` with a local `@font-face` | — | `pnpm -C apps/desktop test -- fonts-local` |
| 3 | Vendor JetBrains Mono into `src/assets/fonts/` with a local `@font-face` | — | `pnpm -C apps/desktop test -- fonts-local` |
| 4 | Vendor Phosphor regular + fill as an SVG sprite in `src/assets/icons/` | — | `pnpm -C apps/desktop test -- icons-local` |
| 5 | `<Icon>` component reading the sprite, sized 9-19px | 4 | `pnpm -C apps/desktop test -- icon-renders` |
| 6 | Assert no external host is referenced anywhere in the built CSS or JS | 2,3,4 | `bash apps/desktop/scripts/check-offline.sh` |
| 7 | `src/styles/type.css` — the scale, with 500 as the only emphasis weight | 1,2,3 | `pnpm -C apps/desktop test -- type-scale` |
| 8 | Assert no rule anywhere sets a font weight above 500 | 7 | `bash apps/desktop/scripts/check-no-bold.sh` |
| 9 | `src/styles/motion.css` — `pulse` and `blink`, and nothing else | 1 | `bash apps/desktop/scripts/check-keyframes.sh` |
| 10 | Interaction states: hover, pressed, and the accent focus outline as shared classes | 1 | `pnpm -C apps/desktop test -- interaction-states` |
| 11 | Assert no element resolves to a browser default focus ring | 10 | `pnpm -C apps/desktop test -- no-default-focus-ring` |
| 12 | `<SkeletonRows>` at real row height, for the tables | 1 | `pnpm -C apps/desktop test -- skeleton-no-reflow` |
| 13 | `<EmptyPane reason="...">` — the reason prop is required, not optional | 1 | `pnpm -C apps/desktop test -- empty-requires-reason` |
| 14 | `<InlineError>` in `--bad`, carrying cause and next action | 1 | `pnpm -C apps/desktop test -- inline-error` |
| 15 | Accent is a single source: changing `--ac` re-themes rings, dots, tabs and numerals together | 1,10 | `pnpm -C apps/desktop test -- accent-single-source` |
| 16 | Assert no hardcoded hex outside `src/fixtures/` and `src/styles/tokens.css` | 1 | `bash apps/desktop/scripts/check-no-raw-hex.sh` |
| 17 | Raise secondary text, hairlines and the status pair to WCAG AA on every ground | 1 | `bash apps/desktop/scripts/check-contrast.sh` |
| 18 | Type sizes become a token scale with an 11px floor and a 14px body | 7 | `bash apps/desktop/scripts/check-type-scale.sh` |
| 19 | The window fills its host; pane widths clamp and card grids reflow | 1 | `bash apps/desktop/scripts/check-responsive.sh` |
