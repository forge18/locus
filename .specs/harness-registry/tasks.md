# harness-registry — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | TOML schema types for `[launch]`, `[telemetry]`, `[models]`, `[layout]`, `[config]`, `[auth]` | — | `cargo test -p locus-core registry::schema_parses` |
| 2 | Loader reading `harnesses/*.toml` and `harnesses/*/`-with-a-plugin | 1 | `cargo test -p locus-core registry::loads_all_twelve` |
| 3 | Validate: all eight extensions declared | 2 | `cargo test -p locus-core registry::rejects_missing_extension` |
| 4 | Validate: `via` strategy is one of the six known | 2 | `cargo test -p locus-core registry::rejects_unknown_strategy` |
| 5 | Validate: `tui = true` refused at registration | 2 | `cargo test -p locus-core registry::rejects_tui_true` |
| 6 | Validate: `[telemetry].source` present and in the known set | 2 | `cargo test -p locus-core registry::rejects_bad_source` |
| 7 | Validate: a downgrade must carry `weaker_than_native` | 2 | `cargo test -p locus-core registry::rejects_unexplained_downgrade` |
| 8 | `locus harness lint` wiring all five validators | 3,4,5,6,7 | `cargo run -p locus-cli -- harness lint` |
| 9 | Assert core names no harness outside registry tests and fixtures | 2 | `bash scripts/check-no-harness-names-in-core.sh` |
| 10 | Registry query API: by name, by telemetry source, by declared verb set | 2 | `cargo test -p locus-core registry::queries` |
| 11 | `core.settings` tier-to-model table, keyed by harness and tier | — | `cargo test -p locus-core models::settings_table` |
| 12 | Tier resolution with up-fallback | 11 | `cargo test -p locus-core models::falls_back_up` |
| 13 | Assert no code path ever falls back down | 12 | `cargo test -p locus-core models::never_falls_down` |
| 14 | Unset tier passes no `flag` and the run still starts | 11 | `cargo test -p locus-core models::unset_uses_harness_default` |
| 15 | Record the resolved model id on the run row | 12 | `cargo test -p locus-core models::resolved_id_on_run` |
| 16 | `list_argv` discovery, with free text where absent | 11 | `cargo test -p locus-core models::list_argv_discovery` |
| 17 | Settings → Harnesses grid reading 11-16 over IPC | 16 | `pnpm -C apps/desktop test -- settings/harness-tiers` |
| 18 | Compute the registry-wide entry and downgrade counts the UI reads | 2,7 | `cargo test -p locus-core registry::counts_are_96_and_33` |
