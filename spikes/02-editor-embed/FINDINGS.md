# Spike 2 — editor embed · FINDINGS

Questions were fixed in [QUESTION.md](QUESTION.md) before any of this ran.

**VERDICT: CodeMirror 6 stays. Nothing found sends the decision back to VSCodium.** One question was
answered against real software; three were not exercised. Both halves are stated plainly, because the
failure this spike exists to prevent is an untested thing inheriting a passing verdict.

| | Question | Verdict |
| --- | --- | --- |
| Q1 | LSP — does `@codemirror/lsp-client` drive a real `rust-analyzer` | **Yes**, with one gap that costs M2 real work |
| Q2 | Merge — `MergeView` over a real diff, per-chunk revert | **NOT EXERCISED** |
| Q3 | Webviews — does either surface misbehave on WebKitGTK | **NOT EXERCISED here.** Spike 3 found a WebKit failure that applies to this milestone too |
| Q4 | Keyboard — Cmd chords and IME composition | **NOT EXERCISED** |

---

## Q1 — LSP

**VERDICT: yes.** A real `rust-analyzer` (1.97.1), spawned as a host-side child process and driven
through `@codemirror/lsp-client`, answers four of the five features the editor needs. Observed against
`fixture/src/main.rs`, not against a mock or a recording.

| Feature | Observed |
| --- | --- |
| completion | returns real crate symbols; `total` appears in the list |
| hover | returns `fn total` **and the doc comment text** — not just a type |
| go to definition | a call site resolves to the declaration's line |
| find references | returns both call sites, not only the nearest |
| diagnostics | **not observed** — see below |

**Diagnostics were not observed and the spike does not claim they were.** They are *pushed* by the
server rather than requested, so they are not a capability flag, and the fixture as written compiles
cleanly — there was nothing for the server to report. The notification handler is wired and received
nothing. What is unproven is the arrival, not the mechanism.

The server also advertises `rename`, `formatting`, `signatureHelp`, `semanticTokens` and `inlayHints`.

### The gap that costs M2 real work

**`@codemirror/lsp-client` has no semantic-token support.** Zero occurrences in its type definitions.
Its exported surface is completion, hover, definition, declaration, type-definition, implementation,
references, rename, formatting and signature help — that is the whole list.

This matters because PLAN.md:2167 names LSP semantic tokens as *the* mitigation for languages with no
Lezer grammar, and PLAN.md:2095 already routes "LSP diagnostics and semantic tokens" over `Channel<T>`
as though they exist. The server half is there — `rust-analyzer` advertises `semanticTokensProvider`.
The client half has to be written: `textDocument/semanticTokens/full`, its delta form, and the
CodeMirror decoration layer. Until it is, a language with no Lezer grammar opens as plain text.

Recorded in `.specs/lsp` Open and `.specs/editor`. **PLAN.md is wrong on this point and should be
corrected rather than worked around.**

### What this does not answer

- **The transport.** The client ran over a Node stdio transport, not over `tauri::ipc::Channel`. That
  proves `lsp-client` drives a real server; it does not prove the IPC discipline PLAN.md specifies for
  high-frequency streams.
- **Per-language coverage.** Rust only. LSP is one protocol and the client is language-agnostic, so
  this answers the protocol question and nothing about the tail. Grammar availability, server
  availability, and per-server capability differences are `.specs/lsp`'s problem, and it now carries
  the rule that **a language is a plugin** — nothing in `crates/locus-core` names one.

## Q2 — Merge

**NOT EXERCISED.** No `MergeView` was rendered, no diff was loaded, no chunk was reverted.

This is the one to be uncomfortable about. PLAN.md §Editor calls reviewing what an agent changed the
**primary** editor job, `.specs/editor` acceptance 4 requires per-chunk revert against an agent's
pushed branch, and `@codemirror/merge` is the package carrying it. A read-only diff is a solved
problem; a diff you can accept a hunk from is the surface the design describes.

**What is known without running it:** `@codemirror/merge@6.12.2` is first-party CodeMirror and
documents both `collapseUnchanged` and per-chunk revert. That is a reason to expect it to work, not
evidence that it does — and this spike existed to stop exactly that substitution being made.

**Carried to `.specs/editor` as an unproven acceptance criterion**, not as a passing one.

## Q3 — Webviews

**NOT EXERCISED in this spike.** No Tauri window was built, so neither WKWebView nor WebView2 nor
WebKitGTK was exercised against CodeMirror.

**But Spike 3 found something that lands here.** `@dschz/solid-flow` calls `requestIdleCallback`
unguarded, WebKit does not implement it, and on WebKit 26.5 the canvas rendered **nothing** — one
`ReferenceError` and no other symptom. WebKit is the engine behind WKWebView (Tauri on macOS) *and*
WebKitGTK (Tauri on Linux).

The transferable lesson is not about solid-flow. It is that **any dependency may reach for a platform
API WebKit lacks, and a Chromium-only check will not find it.** The editor's dependency tree is larger
than the canvas's. `spikes/03-workflow-canvas/scripts/webkit-check.mjs` is the shape of the check that
catches it: load the real build in a real WebKit and read `pageErrors`.

`.specs/editor` acceptance 7 already requires all three webviews and says a platform not tested is
recorded as not tested. That criterion is untouched by this spike.

## Q4 — Keyboard

**NOT EXERCISED.** No accelerator was registered, no Cmd chord was sent, no IME composition was
attempted.

PLAN.md says to budget the time for this and warns against scheduling it as an afternoon. It remains
budgeted and unspent. It is also the question least suited to an automated check — whether a Cmd chord
reaches the app rather than the macOS menu bar, and whether dead-key and CJK composition survive, wants
a person at a real window.

---

## The VSCodium falsifier

Restated from QUESTION.md, with what is now known against each.

| | Falsifier | Result |
| --- | --- | --- |
| 1 | LSP cannot be driven from the webview | **Not triggered.** Four of five features answer from a real server |
| 2 | `MergeView` cannot do per-chunk revert on a real diff | **UNRESOLVED** — not exercised |
| 3 | WebKitGTK breaks either surface with no workaround | **UNRESOLVED** — not exercised. Spike 3's WebKit failure had a three-line workaround, which is weak evidence for "workarounds exist" and no evidence about CodeMirror |
| 4 | Keyboard fidelity cannot be reached | **UNRESOLVED** — not exercised |

**Nothing found argues for VSCodium.** Three of the four falsifiers are simply unfired rather than
disproved, and that distinction should survive into M2 rather than being smoothed over.

## What this spike changes elsewhere

| Where | Change |
| --- | --- |
| PLAN.md:2095, 2167 | Semantic tokens are assumed to exist and do not. Correct the claim |
| `.specs/lsp` | A language is a plugin; per-language server, argv, root files and verb set declared per entry; an unadvertised verb returns `unsupported`, never empty. **Done** |
| `.specs/lsp` | Semantic tokens are M2 work, not an import. **Done** |
| `.specs/editor` | The Lezer tail mitigation is work, not a fallback; a language with no grammar opens as plain text. **Done** |
| `.specs/editor` | Acceptance 4 (per-chunk revert) and 7 (three webviews) enter M2 **unproven**, not de-risked |
| `.specs/ci` | A Chromium-only browser check passes while two of three Tauri platforms are broken |
