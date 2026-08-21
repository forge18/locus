# spike-editor-embed

**Milestone** M0 · **Depends on** none · **Blocks** `editor`, `lsp`

## Purpose

Confirm the editor decision before M2 depends on it. PLAN.md chose CodeMirror 6 used directly, with no
abstraction seam and no VSCodium, and accepted named gaps for it. This spike checks that the two things
the choice rests on actually work inside a Tauri window: a real language server over
`@codemirror/lsp-client`, and `MergeView` over a real git diff — which PLAN.md calls the **primary**
editor surface, since reviewing what an agent changed is the main job.

## Governed by

- PLAN.md §Editor — CodeMirror 6 direct, one editor at two zoom levels, the accepted gaps
- PLAN.md §M0, Spike 2
- PLAN.md §Risks — "Risk — webview inconsistency", "Risk — terminal keyboard fidelity"

## Contract

Delivers `spikes/02-editor-embed/FINDINGS.md` answering:

1. **LSP.** Does `@codemirror/lsp-client` drive a real `rust-analyzer` inside a Tauri window —
   completion, hover, diagnostics, go-to-definition, find-references — with the server supervised on
   the host rather than in the webview?
2. **Merge.** Does `@codemirror/merge`'s `MergeView` render a real git diff with `collapseUnchanged`
   and per-chunk revert, at the density the design handoff draws (mono 11.5px/1.65, 34px gutter)?
3. **Webviews.** Does either misbehave on **WebKitGTK**? PLAN.md names it the weakest of the three and
   the place Tauri's tax lands hardest.
4. **Keyboard.** Do Cmd chords reach the app rather than the macOS menu bar, and does IME composition
   survive? PLAN.md says budget the time and warns against scheduling it as an afternoon.

## Acceptance

1. `spikes/02-editor-embed/FINDINGS.md` exists and gives a verdict per question.
2. A running Tauri window shows completions and diagnostics from a real `rust-analyzer` over a real
   Rust file — a screenshot in the spike directory is the evidence.
3. A `MergeView` renders a real two-commit diff with collapsed unchanged regions, and a chunk revert
   changes the buffer.
4. The finding states the **WebKitGTK verdict explicitly**, including "not tested" if that is the truth
   — an untested platform recorded as passing is the failure this spike exists to prevent.
5. The finding names what would send the decision back to VSCodium, and what it would cost then.

## Open

- Lezer grammar coverage thins out in the tail (PLAN.md names Odin and GDScript as having none).
  Whether the mitigation — LSP semantic tokens plus tree-sitter-WASM decorations — is needed at M2 or
  can wait is not this spike's question, but the spike should note which languages it actually exercised.

## Answered during the spike

- **`@codemirror/lsp-client` has no semantic-token support.** Its exported surface is completion,
  hover, definition, declaration, type-definition, implementation, references, rename, formatting and
  signature help. There is no `semanticTokens` anywhere in its type definitions. PLAN.md:2167 leans on
  LSP semantic tokens as the mitigation for missing Lezer grammars and PLAN.md:2095 routes them over
  `Channel<T>`; neither is available from the package. Recorded in `.specs/lsp` Open and
  `.specs/editor`, because it changes M2's size rather than this spike's verdict.
- **rust-analyzer advertises `semanticTokensProvider: true`**, so the server half is there. What is
  missing is the client half.
- The spike exercised **Rust only**. LSP is one protocol and the client is language-agnostic, so this
  answers the protocol question; it does not answer per-language coverage, which is grammar and server
  availability and belongs to `.specs/lsp`.
