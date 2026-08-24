# setup-revision — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Store the harness allow-list as an ordered list, not a set, so router precedence is preserved | — | `cargo test -p locus-core project::harness_order_preserved` |
| 2 | Reject an agent default whose harness has no working adapter | 1 | `cargo test -p locus-core project::agent_default_requires_adapter` |
| 3 | Render the Harnesses table with adapter-less rows listed and unselectable | 1,2 | `pnpm -C apps/desktop test -- setup/harnesses-table` |
| 4 | Render the Add-harness picker scoped to Workshop-configured harnesses, both empty states, and the router-precedence footer | 3 | `pnpm -C apps/desktop test -- setup/harnesses-add` |
| 5 | Persist repo reassignment: retag runs, artifacts, and memory facts to the new project while the old project tag stays on the historical record | — | `cargo test -p locus-core project::repo_reassign_retags_without_deleting` |
| 6 | Enforce one project per repo | 5 | `cargo test -p locus-core project::repo_single_project` |
| 7 | Render the Repos table with the reassignment caret and footer copy | 5,6 | `pnpm -C apps/desktop test -- setup/repos` |
| 8 | Guard the active default output style against being switched off with no replacement chosen | — | `cargo test -p locus-core harness::materialize::extensions::default_style_guard` |
| 9 | Confirm disabling an extension or entry removes it from materialization without deleting the authored definition | 8 | `cargo test -p locus-core materialize::disable_does_not_delete_definition` |
| 10 | Render the seven extension tab groups with tri-state group masters and per-item toggles | 8,9 | `pnpm -C apps/desktop test -- setup/extensions-groups` |
| 11 | Render the output-style "set default" chip and the no-switch-off-without-replacement guard | 8,10 | `pnpm -C apps/desktop test -- setup/extensions-default-style` |
| 12 | Confirm project tool scope add/remove operates independently of per-role tool scope | — | `cargo test -p locus-core tools::project_scope_add_remove` |
| 13 | Rebuild the project image once per tool-scope change, never per run | 12 | `cargo test -p locus-core sandbox::image_rebuild_once_per_tool_change` |
| 14 | Render CLI tools catalog search, project membership list, and the rebuild-once footer | 12,13 | `pnpm -C apps/desktop test -- setup/cli-tools` |
| 15 | Track `base.md` version, edit time, and run count together as one record | — | `cargo test -p locus-core project::base_context_single_file_metadata` |
| 16 | Render the token budget meter, History, Save, and the over-budget reading | 15 | `pnpm -C apps/desktop test -- setup/base-context` |
| 17 | Model the Memory group's three sections — Short-Term, Long-Term, Artifacts — over project-scoped facts | — | `cargo test -p locus-core memory::persistence_groups` |
| 18 | Offer delete only on Long-Term and Artifacts, never Short-Term | 17 | `cargo test -p locus-core memory::delete_scoped_to_long_term_and_artifacts` |
| 19 | Model Specs & Tasks items carrying the plan body and a nested task list | — | `cargo test -p locus-core planning::spec_item_with_task_list` |
| 20 | Model Research items as source and synthesis entries with no delete control | — | `cargo test -p locus-core wiki::research_items_no_delete` |
| 21 | Page each Persistence section at four items | 17,19,20 | `cargo test -p locus-core project::persistence_page_size` |
| 22 | Render the three Persistence groups with expand-to-body and the intro copy | 17,19,20,21 | `pnpm -C apps/desktop test -- setup/persistence-groups` |
| 23 | Render delete controls scoped to Long-Term and Artifacts items only | 18,22 | `pnpm -C apps/desktop test -- setup/persistence-delete` |
| 24 | Wire a Specs & Tasks task row to navigate to its board card | 19,22 | `pnpm -C apps/desktop test -- setup/persistence-specs-tasks` |
| 25 | Render the "Show all n" / "Show fewer" paging toggle per section | 21,22 | `pnpm -C apps/desktop test -- setup/persistence-paging` |
| 26 | Embed the per-project Analytics tab defined by `analytics-revision` without restating its contract | — | `pnpm -C apps/desktop test -- setup/analytics-tab` |
| 27 | Render the Setup header: project name, locator, Settings/Persistence/Analytics segmented control, Archive, Rename | 3,22,26 | `pnpm -C apps/desktop test -- setup/header` |
| 28 | Add a "Superseded by" pointer line to `desktop-project-operations` naming the three-way split | 27 | `grep -q "Superseded by" .specs/desktop-project-operations/spec.md` |
