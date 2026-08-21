# fixtures

**Milestone** M0.5 · **Depends on** none · **Blocks** every `screens-*`

## Purpose

The whole UI is built before its backend, which is a deliberate trade with one real cost: fixture
shapes get invented before the Postgres schemas exist, and every invented shape is a guess to
reconcile later. This feature exists to make the shapes **derived rather than invented**, so the
reconciliation at M1 is a wiring change and not a redesign.

## Governed by

- PLAN.md §Data model — the eight schemas, which are the source of every fixture type
- PLAN.md §Canonical event vocabulary (the twelve verbs) and the `usage` attribute rule
- PLAN.md §What a session is — session / run / turn
- `docs/design_handoff_locus_desktop_ui/README.md` §State Management — the real data each screen needs

## Contract

**Types come from the schemas, not from the screens.** `src/types/` mirrors the eight Postgres schemas
— `core`, `agents`, `board`, `wiki`, `memory`, `workflows`, `mail`, `market`. A screen that wants a
field the schema does not have is a signal that either the schema or the screen is wrong, and that
argument happens now rather than at M1.

Three rules that make the guess cheap to correct:

1. **One module per screen** in `src/fixtures/`, each opening with a header comment naming the schema
   it draws from and the Tauri command that will replace it:
   ```ts
   // schema: agents.sessions + agents.events
   // replaced by: invoke("sessions_list") + Channel<Event>("session_events")
   ```
2. **One accessor per data set.** Screens never import a fixture directly; they call `useSessions()`,
   `useInboxItems()`, and so on. Swapping in `invoke` is one edit inside the accessor.
3. **Two fixture sets are computed, not authored.** Workshop Harnesses and Workshop Extensions read
   `harnesses/*.toml` at build time. Those two screens are therefore correct on the first day and stay
   correct — and this is what fixes the stale "27 of 88" the handoff copy carries, without anyone
   editing a number.

**Events carry the real vocabulary.** Fixture event streams use only the twelve canonical verbs, and
`usage` is `{input, output, cache_read, cache_write}` or **null** — never zero. PLAN.md is explicit
that where a harness reports nothing, spend reads *unknown* rather than zero, and a fixture that fakes
a zero teaches the UI the wrong thing.

**Fixtures are honest about status.** A screen whose backend does not exist yet says so on screen
rather than presenting invented data as real.

## Acceptance

1. Every type in `src/types/` traces to a named schema in PLAN.md §Data model; none is screen-shaped.
2. Every module in `src/fixtures/` has the two-line header naming its schema and its future command.
3. No screen imports from `src/fixtures/` directly — only through an accessor.
4. Harnesses and Extensions fixtures are generated from `harnesses/*.toml`; regenerating after adding a
   harness changes the counts with no hand edit.
5. Those two screens report **12 harnesses and 33 downgrades**, computed — not the handoff's literal
   27 of 88.
6. Fixture event streams use only the twelve canonical verbs; a thirteenth fails the test.
7. At least one fixture session has `usage: null` so the *unknown* path is exercised.

## Open

- Whether long lists need virtualization at these fixture sizes. Sessions is drawn at 300 rows and Runs
  at 612; measuring here answers it before real data makes it urgent.
