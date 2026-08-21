# Spike 2 — editor embed · QUESTION

Written before the experiment. Nothing below is a conclusion.

**Governed by** PLAN.md §Editor, §M0 Spike 2, §Risks — "Risk — webview inconsistency", "Risk —
terminal keyboard fidelity". Contract in `.specs/spike-editor-embed/spec.md`.

## The unknown

PLAN.md chose **CodeMirror 6 used directly** — no abstraction seam, no VSCodium — and accepted named
gaps for it. That choice rests on two things being true inside a Tauri webview, and neither has been
observed:

1. A **real language server** drives the editor through `@codemirror/lsp-client`, with the server
   supervised on the host rather than inside the webview.
2. `@codemirror/merge`'s `MergeView` renders a **real git diff** at the density the design handoff
   draws — which matters more than it sounds, because PLAN.md calls reviewing what an agent changed
   the **primary** editor job. The editor is a diff viewer first and a text editor second.

A wrapper interface would make this reversible. PLAN.md explicitly refused one — "no abstraction seam"
— so the cost of being wrong is paid at M2 in full. That is the reason this is a spike and not a
task.

## The four questions

### Q1 — LSP

Does `@codemirror/lsp-client` drive a real `rust-analyzer` inside a Tauri window across all five
features the editor needs: **completion, hover, diagnostics, go-to-definition, find-references**?

The server runs on the host as a supervised child process, with LSP traffic crossing to the webview
over `tauri::ipc::Channel` — PLAN.md's stated IPC discipline for high-frequency streams, since the
event system is the wrong tool for a stream that fires per keystroke.

Answered per feature, not in aggregate. "LSP works" is not a verdict; four working and one missing is
a different M2 than five working.

### Q2 — Merge

Does `MergeView` render a real two-commit diff with `collapseUnchanged` and **per-chunk revert**, at
the handoff's Develop density — mono 11.5px / 1.65 line height, 34px gutter?

Per-chunk revert is the part with teeth. A read-only diff is a solved problem; a diff you can accept
a hunk from is the surface PLAN.md describes, and it has to mutate a real buffer.

### Q3 — Webviews

Does either surface misbehave on **WebKitGTK**? PLAN.md names it the weakest of the three webviews and
the place Tauri's tax lands hardest.

**A platform that was not tested is recorded as not tested.** The acceptance criterion says so
explicitly, because the failure mode this question exists to prevent is an untested platform quietly
inheriting the passing verdict from the one that was.

### Q4 — Keyboard

Do **Cmd chords** reach the app rather than being swallowed by the macOS menu bar, and does **IME
composition** survive in the editor buffer?

PLAN.md says to budget the time for this and warns against scheduling it as an afternoon. Dead keys
and multi-keystroke composition are where a webview text surface usually breaks, and the terminal
panes have the same exposure.

## What sends the decision back to VSCodium

Stated now, so the result is a decision rather than a report. **Any one of these:**

1. **LSP cannot be driven from the webview.** If `@codemirror/lsp-client` cannot carry a real server's
   traffic — protocol gaps, or an IPC cost that makes per-keystroke completion unusable — then every
   semantic feature at M2 has to be rebuilt by hand, and `lsp`'s second consumer (the agent, through
   `locus lsp`) is the only part that still works.
2. **`MergeView` cannot do per-chunk revert on a real diff.** The primary editor job fails, and what
   remains is a text editor next to a diff viewer, which is the thing PLAN.md chose CodeMirror to
   avoid building.
3. **WebKitGTK breaks either surface with no workaround.** Linux stops being a supported platform, or
   the editor stops being CodeMirror. Both are decisions, not details.
4. **Keyboard fidelity cannot be reached.** If Cmd chords cannot be claimed from the OS or IME
   composition drops characters, the editor is unusable for anyone who types in a CJK or accented
   locale, and no amount of feature coverage compensates.

## The cost if it is falsified

VSCodium is the fallback and it is not free. Recorded here rather than after the fact:

- The editor stops being a component and becomes an **embedded application** — its own update channel,
  its own extension host, its own process to supervise, and a settings surface Locus does not own.
- The "one editor at two zoom levels" property in PLAN.md §Editor is lost. The review surface and the
  edit surface become different things again.
- The bundle grows by roughly two orders of magnitude over a CodeMirror dependency, and Tauri's
  argument — a small native shell — weakens with it.
- What is gained is real and should not be minimised: language coverage without Lezer grammars, a
  debugger UI that already exists, and a diff surface that has been used by millions.

## What this spike does not decide

- **Which languages get Lezer grammars at M2.** The spike records only which it actually exercised;
  PLAN.md names Odin and GDScript as having none, and whether the mitigation (LSP semantic tokens plus
  tree-sitter-WASM decorations) is needed at M2 is `editor`'s decision, made against this list.
- Which language servers ship in a base image. That is `lsp`'s open question.
- Terminal emulation fidelity beyond the keyboard path the editor shares with it.
