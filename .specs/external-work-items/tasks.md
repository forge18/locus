# external-work-items — tasks

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
| 9 | Assert source edits never synchronize into the imported task | 7 | `cargo test -p locus-core work_item::no_source_sync` |
| 10 | Refuse every outbound source write before local Done | 7 | `cargo test -p locus-core work_item::no_write_before_done` |
| 11 | Create a durable completion delivery on the first local Done transition | 7 | `cargo test -p locus-core work_item::completion_outbox` |
| 12 | Post one idempotent completion comment with task locator and evidence | 11 | `cargo test -p locus-core work_item::completion_comment` |
| 13 | Resolve the external item after commenting when the provider supports it | 11,12 | `cargo test -p locus-core work_item::completion_resolves` |
| 14 | Record unsupported resolution without failing local Done | 11,12 | `cargo test -p locus-core work_item::resolution_capability_refused` |
| 15 | Retry a failed completion delivery without a source fetch or sync | 11 | `cargo test -p locus-core work_item::completion_retry_is_one_way` |
| 16 | Run conformance fixtures for arbitrary work-item provider plugins | 4,5 | `cargo test -p locus-core work_item::provider_conformance` |
| 17 | Render provider selection and item preview from Kanban import | 6 | `pnpm -C apps/desktop test -- automate/import-kanban` |
| 18 | Render provider selection and item preview from List import | 6 | `pnpm -C apps/desktop test -- automate/import-list` |
| 19 | Render the shared import confirmation with workflow selection and one-way notice | 7,17,18 | `pnpm -C apps/desktop test -- automate/import-confirmation` |
| 20 | Render completion delivery state and external-resolution result in task detail | 11,14,16 | `pnpm -C apps/desktop test -- automate/import-completion-status` |
