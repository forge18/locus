# fixtures — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | `src/types/core.ts` — projects, repos, settings | — | `pnpm -C apps/desktop exec tsc --noEmit` |
| 2 | `src/types/agents.ts` — agent_defs, sessions, runs, events, artifacts, comments | — | `pnpm -C apps/desktop exec tsc --noEmit` |
| 3 | `src/types/event.ts` — the twelve canonical verbs as a union, `usage` nullable | 2 | `pnpm -C apps/desktop test -- types/event-vocabulary` |
| 4 | `src/types/board.ts`, `wiki.ts`, `workflows.ts`, `mail.ts`, `market.ts`, `memory.ts` | — | `pnpm -C apps/desktop exec tsc --noEmit` |
| 5 | Assert every exported type names its schema in a doc comment | 1,2,4 | `bash apps/desktop/scripts/check-types-cite-schema.sh` |
| 6 | `scripts/gen-plugin-fixtures.ts` — parse the registry and trusted plugin manifests into fixtures | — | `pnpm -C apps/desktop exec tsx scripts/gen-plugin-fixtures.ts` |
| 7 | Generated fixture reports Pi plus trusted user harness plugins | 6 | `pnpm -C apps/desktop test -- fixtures/harness-count` |
| 8 | Generated fixture reports dynamic downgrade counts across registered entries | 6 | `pnpm -C apps/desktop test -- fixtures/downgrade-count` |
| 9 | Per-plugin mechanism badge and model tiers derived from the descriptor | 6 | `pnpm -C apps/desktop test -- fixtures/harness-mechanisms` |
| 10 | Extensions fixture: per-type native vs downgraded counts, from the same parse | 6 | `pnpm -C apps/desktop test -- fixtures/extension-counts` |
| 11 | Authored fixtures for inbox, status, plan, develop, board, sessions, telemetry, runs, artifacts, wiki, workflow | 1,2,4 | `pnpm -C apps/desktop test -- fixtures/all-present` |
| 12 | Every fixture module carries the schema + future-command header | 11 | `bash apps/desktop/scripts/check-fixture-headers.sh` |
| 13 | Session event streams use only the twelve verbs | 3,11 | `pnpm -C apps/desktop test -- fixtures/only-canonical-verbs` |
| 14 | At least one session carries `usage: null` for the unknown path | 11 | `pnpm -C apps/desktop test -- fixtures/usage-unknown-exists` |
| 15 | One accessor per data set in `src/data/` | 11 | `pnpm -C apps/desktop test -- data/accessors` |
| 16 | Lint: no screen imports `src/fixtures/` directly | 15 | `bash apps/desktop/scripts/check-no-direct-fixture-import.sh` |
| 17 | `<FixtureNotice>` marking a screen whose backend does not exist yet | 15 | `pnpm -C apps/desktop test -- fixtures/notice` |
| 18 | Measure render cost of the 612-row Runs table and record whether virtualization is needed | 11 | `pnpm -C apps/desktop test -- fixtures/large-table-budget` |
