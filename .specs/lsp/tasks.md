# lsp — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | LSP client crate: initialize, request, notify, shutdown | — | `cargo test -p locus-core lsp::client` |
| 2 | Host supervisor: spawn one server set per project | 1 | `cargo test -p locus-core lsp::host_supervisor` |
| 3 | Multiplex one server across several editor panes | 2 | `cargo test -p locus-core lsp::multiplexes` |
| 4 | Restart a crashed server without killing the pane | 2 | `cargo test -p locus-core lsp::restarts_on_crash` |
| 5 | Stream diagnostics to the webview over `Channel<T>` | 2 | `pnpm -C apps/desktop test -- editor/diagnostics-channel` |
| 6 | In-container deployment against that run's clone | 1 | `cargo test -p locus-core lsp::in_container` |
| 7 | `locus lsp def` | 6 | `cargo test -p locus-cli lsp::def` |
| 8 | `locus lsp refs` | 6 | `cargo test -p locus-cli lsp::refs` |
| 9 | `locus lsp hover` | 6 | `cargo test -p locus-cli lsp::hover` |
| 10 | `locus lsp symbols` | 6 | `cargo test -p locus-cli lsp::symbols` |
| 11 | `locus lsp diagnostics` | 6 | `cargo test -p locus-cli lsp::diagnostics` |
| 12 | `locus lsp rename` | 6 | `cargo test -p locus-cli lsp::rename` |
| 13 | `--json` compact output on every verb | 7,8,9,10,11,12 | `cargo test -p locus-cli lsp::json_compact` |
| 14 | Agent and editor agree when the trees match | 7,2 | `cargo test -p locus-core lsp::agree_when_same_tree` |
| 15 | They diverge when the trees differ — proving separate servers | 14 | `cargo test -p locus-core lsp::differ_when_trees_differ` |
| 16 | Allowlist enforcement: no `locus lsp` without it in `tools` | 7 | `cargo test -p locus-core lsp::allowlist_enforced` |
| 17 | Built-in language descriptor schema and catalog, loaded without language branches in core | 1 | `cargo test -p locus-core lsp::catalog` |
| 18 | Import a local descriptor bundle into the user catalog; validate, copy immutably, and hash it | 17 | `cargo test -p locus-core lsp::import` |
| 19 | Detect root markers and file extensions when a repository joins a project; suggest but never import from that repository | 17 | `cargo test -p locus-core lsp::detect` |
| 20 | Explicitly enable a suggested or imported descriptor and pin its id, version, and hash in project state | 18,19 | `cargo test -p locus-core lsp::project_pin` |
| 21 | Pre-provision each enabled server for the host cache and agent image layer before first use | 20 | `cargo test -p locus-core lsp::preprovision` |
| 22 | Request and decode `textDocument/semanticTokens/full` only from servers that advertise it | 1 | `cargo test -p locus-core lsp::semantic_tokens_full` |
| 23 | Apply semantic-token delta responses against the previous result | 22 | `cargo test -p locus-core lsp::semantic_tokens_delta` |
| 24 | Render semantic tokens as CodeMirror decorations and degrade unsupported languages to editable plain text | 22 | `pnpm -C apps/desktop test -- editor/semantic-tokens` |
