# external-work-items — tasks

Revised by [IMPACT_EXTERNAL_SYNC](../IMPACT_EXTERNAL_SYNC.md): the one-way invariant is superseded by
two-way status and note synchronization. Tasks 9 and 10 tested the reversed invariants and are
superseded, not deleted — their history stays here. Tasks 21–32 carry the sync contract.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Define opaque plugin identity, normalized snapshot, and completion capabilities | — | `cargo test -p locus-core work_item::contract_types` |
| 2 | Persist configured work-item plugin ID, host, and project identity | 1 | `cargo test -p locus-core work_item::provider_configuration` |
| 3 | Implement the external-work-item plugin registry and capability refusal | 1,2 | `cargo test -p locus-core work_item::adapter_selection` |
| 4 | Expose GitHub through the work-item plugin port | 3, forge-providers:26 | `cargo test -p locus-core work_item::plugin_adapter_bridge` |
| 5 | Implement GitHub issue import through the `gh` CLI plugin | 3 | `cargo test -p locus-core work_item::plugin_snapshot_contract` |
| 6 | Preview a normalized external item without creating local state | 3 | `cargo test -p locus-core work_item::preview` |
| 7 | Import a confirmed snapshot as a local task with selected workflow | 6, task-orchestration:5 | `cargo test -p locus-core work_item::import_creates_task` |
| 8 | Refuse duplicate plugin-identity import and open the existing task | 7 | `cargo test -p locus-core work_item::deduplicates_import` |
| 9 | ~~Assert source edits never synchronize into the imported task~~ | 7 | superseded by the sync contract — see tasks 22, 25 |
| 10 | ~~Refuse every outbound source write before local Done~~ | 7 | superseded by the sync contract — see tasks 23, 24 |
| 11 | Create a durable completion delivery on the first local Done transition | 7 | `cargo test -p locus-core work_item::completion_outbox` |
| 12 | Post one idempotent completion comment with task locator and evidence | 11 | `cargo test -p locus-core work_item::completion_comment` |
| 13 | Resolve the external item after commenting when the provider supports it | 11,12 | `cargo test -p locus-core work_item::completion_resolves` |
| 14 | Record unsupported resolution without failing local Done | 11,12 | `cargo test -p locus-core work_item::resolution_capability_refused` |
| 15 | Retry a failed completion delivery without a source fetch or sync | 11 | `cargo test -p locus-core work_item::completion_retry_is_one_way` |
| 16 | Run conformance fixtures for arbitrary work-item provider plugins | 4,5 | `cargo test -p locus-core work_item::provider_conformance` |
| 17 | Render provider selection and item preview from Kanban import | 6 | `pnpm -C apps/desktop test -- automate/import-kanban` |
| 18 | Render provider selection and item preview from List import | 6 | `pnpm -C apps/desktop test -- automate/import-list` |
| 19 | Render the shared import confirmation with workflow selection and sync notice | 7,17,18 | `pnpm -C apps/desktop test -- automate/import-confirmation` |
| 20 | Render completion delivery state and external-resolution result in task detail | 11,14,16 | `pnpm -C apps/desktop test -- automate/import-completion-status` |
| 21 | Sync capability: provider-declared status vocabulary with bidirectional column mapping | — | `cargo test -p locus-core work_item::sync_capability` |
| 22 | Pull external status changes and notes since a persisted cursor | 21,30 | `cargo test -p locus-core work_item::sync_pull` |
| 23 | Push a normalized column (and blocked where declared) through the provider mapping | 21 | `cargo test -p locus-core work_item::sync_push_status` |
| 24 | Post local notes outward with attribution and the Locus marker | 21 | `cargo test -p locus-core work_item::sync_push_note` |
| 25 | Echo suppression: an inbound Locus-marked note is recorded, never re-appended or re-posted | 22,24 | `cargo test -p locus-core work_item::echo_suppression` |
| 26 | Resolve a two-sided status change last-write-wins with the decision as evidence | 22 | `cargo test -p locus-core work_item::status_conflict_lww` |
| 27 | Sync-originated Done records the completion as satisfied without a duplicate comment | 22,11 | `cargo test -p locus-core work_item::external_done_satisfied` |
| 28 | Sync moves land as `task.moved` with the sync actor through the fold; sync Done without external evidence is refused | 21, board:20 | `cargo test -p locus-core board::sync_moves_through_fold` |
| 29 | Inbound external notes append `Commented` events with external author and origin | 22, board:20 | `cargo test -p locus-core board::external_note_origin` |
| 30 | Sync state on `board.external_work_items`: pull cursor, last-pushed status, note watermark | — | `cargo test -p locus-core store::external_sync_state` |
| 31 | Render sync state, the conflict decision, and the Sync-now control in task detail | 23,26,27 | `pnpm -C apps/desktop test -- automate/sync-status` |
| 32 | Conformance fixtures for sync-capable provider plugins | 4,5,21–24 | `cargo test -p locus-core work_item::provider_conformance_sync` |

**Sync batch status:** tasks 21–32 are implemented. Docker-backed persistence verification remains environment-dependent when the local Docker daemon is unavailable.
