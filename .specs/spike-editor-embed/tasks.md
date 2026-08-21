# spike-editor-embed — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Write `spikes/02-editor-embed/QUESTION.md` — the four questions and what sends the editor decision back to VSCodium | — | `test -s spikes/02-editor-embed/QUESTION.md` |
| 2 | Minimal Tauri + Solid app in the spike dir with CodeMirror 6 mounted on a real file | 1 | `pnpm -C spikes/02-editor-embed build` |
| 3 | Spawn and supervise `rust-analyzer` on the host, piped to the webview over `tauri::ipc::Channel` | 2 | `pnpm -C spikes/02-editor-embed test -- lsp-handshake` |
| 4 | Wire `@codemirror/lsp-client`: completion, hover, diagnostics, definition, references | 3 | `pnpm -C spikes/02-editor-embed test -- lsp-features` |
| 5 | Capture `screenshots/lsp.png` showing real completions and a real diagnostic | 4 | `test -s spikes/02-editor-embed/screenshots/lsp.png` |
| 6 | `MergeView` over a real two-commit diff from this repo, with `collapseUnchanged` | 2 | `pnpm -C spikes/02-editor-embed test -- mergeview-renders` |
| 7 | Per-chunk revert mutates the buffer and the change is observable | 6 | `pnpm -C spikes/02-editor-embed test -- mergeview-revert` |
| 8 | Style the MergeView to the handoff's Develop density and capture `screenshots/diff.png` | 6 | `test -s spikes/02-editor-embed/screenshots/diff.png` |
| 9 | Run 4 and 6 on WKWebView; record the result | 4,6 | `grep -q 'WKWebView:' spikes/02-editor-embed/FINDINGS.md` |
| 10 | Run 4 and 6 on WebKitGTK in a Linux container or VM; record pass, fail, or not-tested with a reason | 4,6 | `grep -q 'WebKitGTK:' spikes/02-editor-embed/FINDINGS.md` |
| 11 | Cmd-chord test: register an accelerator in Rust, confirm it reaches the app with no default menu | 2 | `pnpm -C spikes/02-editor-embed test -- cmd-chord` |
| 12 | IME composition and dead-key test in the editor buffer | 2 | `pnpm -C spikes/02-editor-embed test -- ime-composition` |
| 13 | Write `FINDINGS.md`: verdict per question, languages exercised, the VSCodium falsifier and its cost | 5,7,8,9,10,11,12 | `bash spikes/02-editor-embed/check-findings.sh` |
