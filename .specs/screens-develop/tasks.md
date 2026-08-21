# screens-develop — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Three-column frame at 206 / flex / 252, resizable | — | `pnpm -C apps/desktop test -- develop/layout` |
| 2 | `<FileTree>` with 20px/34px indents at 11.5px | 1 | `pnpm -C apps/desktop test -- develop/tree` |
| 3 | Tree branch header: accent `git-branch`, mono branch, caret | 2 | `pnpm -C apps/desktop test -- develop/tree-header` |
| 4 | Selected file on `--sf2` radius 5 with the accent status badge | 2 | `pnpm -C apps/desktop test -- develop/tree-selected` |
| 5 | Tree footer naming the linked repo and its checkout path | 2 | `pnpm -C apps/desktop test -- develop/tree-footer` |
| 6 | 30px editor tab bar; active tab `--bg` with the accent top inset and close `x` | 1 | `pnpm -C apps/desktop test -- develop/tabs` |
| 7 | `collapseUnchanged` toggle and the accent chunk count | 6 | `pnpm -C apps/desktop test -- develop/chunk-count` |
| 8 | `<SideBySideDiff>` at mono 11.5/1.65 with a 34px gutter and a 12px sign column | 1 | `pnpm -C apps/desktop test -- develop/diff-layout` |
| 9 | Added and removed row tints at the documented rgba values | 8 | `pnpm -C apps/desktop test -- develop/diff-tints` |
| 10 | Collapsed `⋯ N unchanged lines` strips | 8 | `pnpm -C apps/desktop test -- develop/diff-collapsed` |
| 11 | Diff headers: `HEAD · main` in `--mu2`, the agent branch in accent | 8 | `pnpm -C apps/desktop test -- develop/diff-headers` |
| 12 | Token coloring: keywords `#8fb8d6`, comments `--mu` | 8 | `pnpm -C apps/desktop test -- develop/diff-tokens` |
| 13 | 52px footer with both actions, the mono LSP line, and the note | 1 | `pnpm -C apps/desktop test -- develop/footer` |
| 14 | `<GitPanel>` frame on `--bg-deep` with the ahead/behind header | 1 | `pnpm -C apps/desktop test -- develop/git-header` |
| 15 | Branch block with provenance line | 14 | `pnpm -C apps/desktop test -- develop/git-branch-block` |
| 16 | Staged and unstaged sections with their bulk-action links | 14 | `pnpm -C apps/desktop test -- develop/git-sections` |
| 17 | File rows: colored status letter, mono truncated path, right-aligned counts | 16 | `pnpm -C apps/desktop test -- develop/git-rows` |
| 18 | Current file's row highlighted on `--sf2` | 17 | `pnpm -C apps/desktop test -- develop/git-current-file` |
| 19 | Stage and unstage per file | 17 | `pnpm -C apps/desktop test -- develop/stage-file` |
| 20 | Stage and unstage per hunk, moving only that hunk | 19,8 | `pnpm -C apps/desktop test -- develop/stage-hunk` |
| 21 | History: 7px dots, accent halo on newest only, mono sha/author/age | 14 | `pnpm -C apps/desktop test -- develop/git-history` |
| 22 | Git footer: commit field, Commit, Push, and the ownership note verbatim | 14 | `pnpm -C apps/desktop test -- develop/git-footer-note` |
| 23 | Visual check against `screenshots/04-develop.png` | 13,22 | `pnpm -C apps/desktop test -- visual -- develop` |
