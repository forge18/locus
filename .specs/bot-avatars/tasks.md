# bot-avatars — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `@dicebear/core` + `@dicebear/collection` in `apps/desktop`; derived-avatar helper: (style, seed) → in-memory memoized data URI, transparent background | — | `pnpm -C apps/desktop test -- avatars/derive` |
| 2 | Determinism: same id and style render byte-identically; seed is the bot id, so a rename keeps the robot | 1 | `pnpm -C apps/desktop test -- avatars/determinism` |
| 3 | Bot list rows and the collapsed 40px strip show the avatar; live state stays a readable ring/badge | 2 | `pnpm -C apps/desktop test -- avatars/bot-list` |
| 4 | Bot view header chrome: avatar + name + harness; `AgentPane.tsx` gains no avatar code and no Bots-specific props | 3 | `pnpm -C apps/desktop test -- avatars/bot-header` |
| 5 | `bots.avatar_style` in `core.settings`, default `bottts`, read app-wide | 1 | `pnpm -C apps/desktop test -- avatars/style-setting` |
| 6 | Settings single-select over every style shipped in `@dicebear/collection` with creator and license per entry; changing it re-renders immediately | 5 | `pnpm -C apps/desktop test -- avatars/style-picker` |
| 7 | The active style's creator and license are visible in Settings | 6 | `pnpm -C apps/desktop test -- avatars/attribution` |
| 8 | Theme conformance: transparent background holds under both `data-theme` values | 3 | `pnpm -C apps/desktop test -- avatars/themes` |
| 9 | Zero persistence: no avatar column, row, file, or materializer change | 1 | `! rg -qi avatar migrations/ crates/` |
| 10 | design-revision bots view contract and `docs/UI_MOCKUP_REVIEW.md` note the avatar | 3 | `rg -ni avatar .specs/design-revision/spec.md docs/UI_MOCKUP_REVIEW.md` |
