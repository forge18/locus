# editor — tasks

Spike 2 gates this feature. If it returned a verdict against CodeMirror, this file is rewritten.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | CodeMirror 6 core in an editor pane, one theme built from tokens | — | `pnpm -C apps/desktop test -- editor/mounts` |
| 2 | One keymap module, imported by both zoom levels | 1 | `pnpm -C apps/desktop test -- editor/one-keymap` |
| 3 | Full-window editor module reusing the pane's components | 1 | `pnpm -C apps/desktop test -- editor/two-zoom-levels` |
| 4 | Assert both modes share components by import, not by resemblance | 3 | `pnpm -C apps/desktop test -- editor/shared-by-import` |
| 5 | `@codemirror/lsp-client` wired to the host LSP supervisor | 1 | `pnpm -C apps/desktop test -- editor/lsp-client` |
| 6 | Completion, hover and signature help from a real server | 5 | `pnpm -C apps/desktop test -- editor/completions` |
| 7 | Diagnostics rendered in the gutter and inline | 5 | `pnpm -C apps/desktop test -- editor/diagnostics` |
| 8 | Jump-to-definition and find-references | 5 | `pnpm -C apps/desktop test -- editor/navigation` |
| 9 | Rename and format through the server | 5 | `pnpm -C apps/desktop test -- editor/rename-format` |
| 10 | `Workspace` abstraction for multi-file | 5 | `pnpm -C apps/desktop test -- editor/workspace` |
| 11 | `MergeView` over an agent's pushed branch against its base | 1 | `pnpm -C apps/desktop test -- editor/mergeview` |
| 12 | `collapseUnchanged` with the handoff's strip styling | 11 | `pnpm -C apps/desktop test -- editor/collapse` |
| 13 | Per-chunk revert mutating the buffer | 11 | `pnpm -C apps/desktop test -- editor/revert-chunk` |
| 14 | Open a linked repo's own checkout | — | `cargo test -p locus-core editor::opens_linked_checkout` |
| 15 | Open a managed repo's Locus-side clone | 14 | `cargo test -p locus-core editor::opens_managed_clone` |
| 16 | Assert no worktree is created anywhere | 15 | `cargo test -p locus-core editor::no_worktrees` |
| 17 | Replace the Develop screen's fixture diff with the real MergeView | 11,13 | `pnpm -C apps/desktop test -- develop/real-diff` |
| 18 | Exercise 6, 7, 11 on WKWebView | 6,7,11 | `pnpm -C apps/desktop test -- webview -- wkwebview` |
| 19 | Exercise 6, 7, 11 on WebView2 | 6,7,11 | `pnpm -C apps/desktop test -- webview -- webview2` |
| 20 | Exercise 6, 7, 11 on WebKitGTK | 6,7,11 | `pnpm -C apps/desktop test -- webview -- webkitgtk` |
| 21 | Record an untested platform as untested, never as passing | 18,19,20 | `bash apps/desktop/scripts/check-webview-matrix.sh` |
| 22 | Assert no debug gutter, variables pane, or step control exists | 1 | `pnpm -C apps/desktop test -- editor/no-debug-ui` |
