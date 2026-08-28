# External work-item synchronization impact

## Target

Amend the shipped M7 `external-work-items` contract from one-way (import in, single completion
delivery out at Done) to two-way synchronization of **statuses and notes** between the Locus board and
the linked external item, for providers that declare the new sync capability. Decisions settled with
the product owner: all local notes post outward automatically; the provider plugin owns the external↔column
status mapping; status conflicts resolve last-write-wins with the decision visible; an external close
moves the card to Done with the close as its evidence.

## Dependents (9)

- `.specs/external-work-items/spec.md:3` — purpose states "the source system is not synchronized
  while work is in progress"; the contract's one-way paragraphs and acceptance 4, 6, and 9 assert the
  opposite of the new behavior.
- `.specs/external-work-items/tasks.md` — rows 9 (`no_source_sync`) and 10 (`no_write_before_done`)
  test invariants this change reverses; they become superseded, and sync tasks join the table.
- `.specs/board/spec.md` — the fold and actor model are authoritative for how sync writes land:
  `BoardActor` gains a sync variant, the Done gate needs a rule for sync-originated moves, and the
  task shape's note stream gains external attribution.
- `PLAN.md:115` (Plugins row) and the plugin-contract paragraph (~line 127) — describe work-item
  plugins as "snapshot plus a completion comment and optional resolution"; sync capabilities extend
  that sentence.
- `PLAN.md:1463` — the board task shape still names `github_issue`; the board spec's provider-neutral
  `external_issue` already won this. The sync amendment touches this block anyway; the stale name is
  corrected in the same edit.
- `crates/locus-core/src/services/board.rs:172` — `BoardActor { Human, Agent { run_id } }` needs a
  sync variant so a sync move is distinguishable in "who moved this, and when".
- `crates/locus-core/src/services/board.rs:79` — `BoardComment { author, body }` has no origin field;
  inbound external notes need attribution and an echo-suppression marker.
- `crates/locus-core/src/work_item.rs:1-4` — module doc states "Source edits never synchronize back";
  `source_edit_does_not_sync` (line 596) and the tests at 1095-1130 assert one-way behavior and flip
  to the sync contract.
- `migrations/0023_external_work_items.up.sql` / `0024` — `board.external_work_items` gains sync
  state (pull cursor, last-pushed status, note watermark); additive, no data rewrite.

## Affected Stories

- M7 `external-work-items` — the feature itself; contract, tasks, and the shipped one-way
  implementation.
- M5 `board` — fold-level amendments only (sync actor, external-evidence Done rule, note origin);
  columns, gating, and the no-direct-write rule are unchanged.
- PLAN.md §Plugins and §The board — the two architecture paragraphs that describe the touched
  surfaces.

## Test Coverage

- `work_item.rs:1095` `no_source_sync` and the outbox tests at 1118-1130 encode the invariants being
  reversed and must be rewritten as sync-contract tests (echo suppression, LWW, external-Done
  evidence).
- `board` fold tests cover `Human` and `Agent` actors only; the sync actor, its Done-evidence rule,
  and external note attribution are uncovered.
- `work_item::provider_conformance` covers snapshot/comment/resolve fixtures; sync fixtures (mapping
  declaration, pull, push, markers) do not exist.

## Risk: Medium-High

The migration is additive and the plugin port already isolates provider detail, so the footprint is
contained. The real risks are behavioral: a marker or watermark bug produces a synchronization loop
(duplicate outward notes) that nothing fails loudly on, and LWW decisions that are not recorded in the
fold would recreate exactly the "status disagreeing with its evidence" possibility the board exists to
make impossible. Both are called out as acceptance criteria in the revised spec.

## Recommended action

Amend the contracts in this order: `external-work-items/spec.md` (the authority), `tasks.md`
(supersede rows 9-10, add sync tasks with runnable `verify:`), `board/spec.md` (fold amendments),
then the two PLAN.md paragraphs. Implementation follows the task order: plugin capability and
vocabulary declaration first, pull/push second, echo suppression and LWW before any UI surface, so
the loop-prone parts land under test before they are reachable from the board.
