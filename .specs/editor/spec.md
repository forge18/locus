# editor

**Milestone** M2 · **Depends on** `spike-editor-embed`, `screens-develop`, `pane-manager`

## Purpose

CodeMirror 6 used directly, no wrapper interface. **One editor at two zoom levels** — the side pane
beside an agent and the full-window module are the same components at different sizes, sharing one
keymap, one theme, one LSP client.

A second editor for the "real" module would mean two of each and muscle memory that breaks whenever you
move between them — and would re-import the whole extension-host cost for the surface you use *least*
in an agent-first IDE.

## Governed by

- PLAN.md §Editor — CodeMirror direct, two zoom levels, the accepted gaps
- PLAN.md §The git model — the editor opens an ordinary clone
- `docs/UI_MOCKUP_REVIEW.md` · Interact

## Contract

- `@codemirror/lsp-client` — completion, hover, signature help, format, rename, jump-to-definition,
  find-references, diagnostics, and its `Workspace` abstraction for multi-file.
- `@codemirror/merge` — `MergeView` with `collapseUnchanged` and per-chunk revert. **This is the
  primary editor surface**: reviewing what an agent changed.

**The editor opens an ordinary clone.** For a **linked** repo that is your own checkout, where
`git fetch locus && git checkout agent/<run-id>` is the motion you already know. For a **managed** repo
Locus keeps one normal clone per project beside the bare remote and opens that.

**No worktrees anywhere in the design.** A clone is what the git model already produces, and adding a
second checkout mechanism would mean two ways to be on the wrong branch.

**Declared gaps, restated so they are not rediscovered:**

- **No debug UI** — no gutter, no variables pane, no step controls. `locus debug` serves the side that
  needed it, so CodeMirror's lack of one costs nothing. That was the single real gap in the CodeMirror
  trade, and it closed by moving the capability to the agent.
- **No VS Code extensions.**
- **Lezer grammar coverage thins in the tail** — Odin and GDScript have none. PLAN.md's mitigation is
  LSP semantic tokens for colour plus tree-sitter-WASM decorations for structure. **That mitigation is
  not something this milestone can import.** `spike-editor-embed` found `@codemirror/lsp-client` has
  no semantic-token support at all, so the mitigation is work, not a fallback — see `.specs/lsp` Open.
  Until it is written, a language with no Lezer grammar opens as plain text.
- **A language is a descriptor.** Grammar, server, and root-detection are declared per language and
  resolved at runtime; `.specs/lsp` owns the Locus-internal catalog and explicit user-import contract.
  Nothing here hard-codes a language list, so adding one later is a descriptor entry rather than a
  change to the editor.

## Acceptance

1. Side-pane and full-window modes are the **same components**, asserted by shared import rather than
   by looking alike.
2. One keymap and one theme serve both; a keybinding added once works in both.
3. Completions, hover and diagnostics come from a real language server.
4. `MergeView` renders an agent's pushed branch against its base, with per-chunk revert working.
5. The editor opens a clone — a linked repo opens your checkout, a managed repo opens Locus's.
6. No worktree is created anywhere.
7. All of the above pass on **WKWebView, WebView2 and WebKitGTK**. A platform not tested is recorded as
   not tested, never as passing.
8. No debug gutter, variables pane, or step control exists — asserted, because this is a decision that
   erodes by accretion.
9. A file in a language with **no** Lezer grammar opens and is editable as plain text rather than
   failing — the tail is degraded, never broken.

## Open

- Which languages get Lezer grammars at M2. The spike records what it exercised; the internal catalog
  consumes that list rather than guessing.
