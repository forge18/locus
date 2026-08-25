# marketplace-index — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Manifest schema: name, summary, install, verify, docs, caps | — | `cargo test -p locus-core market::manifest_schema` |
| 2 | Read manifests from a local directory | 1 | `cargo test -p locus-core market::reads_local_dir` |
| 3 | Validate a manifest, rejecting a malformed one by name | 2 | `cargo test -p locus-core market::validates` |
| 4 | Persist to the `market` schema | 2 | `cargo test -p locus-core market::persists` |
| 5 | Resolve an agent's `tools` list against the index | 4 | `cargo test -p locus-core market::resolves_tools` |
| 6 | Unresolvable name fails at save, naming it | 5 | `cargo test -p locus-core market::rejects_unknown_tool` |
| 7 | Build the catalog: name plus one line per allowlisted tool | 5 | `cargo test -p locus-core market::catalog` |
| 8 | Assert the catalog carries no flags, examples or schema | 7 | `cargo test -p locus-core market::catalog_is_a_line` |
| 9 | Measure catalog cost: fifteen tools near 225 tokens | 7 | `cargo test -p locus-core market::catalog_cost` |
| 10 | Inject the catalog into the run's context | 7 | `cargo test -p locus-core market::catalog_injected` |
| 11 | `locus tools list` showing only allowlisted tools | 7 | `cargo test -p locus-cli tools::list` |
| 12 | `locus tools docs <name>` returning the full page on demand | 4 | `cargo test -p locus-cli tools::docs` |
| 13 | Assert docs are injected only for allowlisted tools | 12 | `cargo test -p locus-core market::docs_only_when_allowlisted` |
| 14 | Assert a non-allowlisted tool is unreachable even when indexed | 5 | `cargo test -p locus-core market::allowlist_is_a_boundary` |
| 15 | Seed the built-in index with `gh`; core-image dependencies are not Workshop plugins | 4 | `cargo test -p locus-core market::seeded` |
