# lsp

**Milestone** M2 · **Depends on** `editor`, `sandbox`

## Purpose

Semantic navigation for two consumers with one implementation. **The Rust client is shared; the server
is local to the code it is answering about** — which is the whole subtlety, because the host's language
servers index *your* working copy and an agent's questions are about a different tree.

PLAN.md's argument for giving agents this at all: an agent grepping for a symbol reads ten files to
answer what one LSP call answers exactly. Cheaper in tokens *and* correct on overloads, shadowing and
re-exports.

## Governed by

- PLAN.md §Agents need real tools — where each server runs
- PLAN.md §Editor — language servers on the host, one set per project, shared across panes
- PLAN.md §Marketplace — on demand, not always on

## Contract

**Two deployments, one client:**

| Deployment | Server runs | Indexes |
| --- | --- | --- |
| Editor panes | host, supervised, one set per project, multiplexed | your working copy |
| `locus lsp` | **the agent's container** | that run's clone |

```
locus lsp def|refs|hover|symbols|diagnostics|rename
```

**Agents' containers do not run the host's language servers, and the host does not answer for the
agent's tree.** A server answering about the wrong checkout gives confidently wrong line numbers.

**On demand, not always on.** `locus lsp` is a tool in an agent's allowlist, resolved from the
marketplace like any other. An agent that does not need it does not get it and pays nothing.

**`--json` on every verb**, compact, because the caller is a model.

**A language is a plugin, not a branch in core.** Same rule the harness registry already lives under —
nothing in `crates/locus-core` names a harness, and nothing in it names a language either. Adding
Odin is a manifest entry plus, where the server needs it, a plugin; a `match` on a language name in
core is a bug. This is what makes "support a lot of languages" a data problem rather than a release
problem, and it is why the per-language pieces below are declared per entry rather than assumed:

| Per language, declared | Why it cannot be assumed |
| --- | --- |
| the server binary and how it is installed | `rust-analyzer` ships with a toolchain; `gopls` is a `go install`; others are npm, pip, or a release tarball |
| its launch argv and root-detection files | `Cargo.toml`, `go.mod`, `package.json`, `pyproject.toml` — a server given the wrong root indexes nothing and reports no error |
| which of the six verbs it actually answers | capabilities differ per server; `locus lsp refs` against a server with no `referencesProvider` must say unsupported, not return empty |
| whether a Lezer grammar exists | see `editor` — this is the gap that decides whether highlighting works at all |

**A verb a server cannot answer is reported unsupported, never as an empty result.** Same rule as
telemetry's missing verbs: an empty `refs` list and "this server has no references provider" are
different facts, and collapsing them makes an agent believe a symbol is unused.

## Acceptance

0. **Adding a language touches no file in `crates/locus-core`.** Asserted the way the harness rule is:
   grep core for language names and find none.
1. The editor gets completions and diagnostics from a host-supervised server.
2. One server set per project is shared across panes rather than one per pane.
3. `locus lsp def` in a container resolves a symbol against **that run's clone**.
4. The agent and the editor give the **same answer** for the same symbol when the trees match, and
   **different** answers when they diverge — the second half is what proves the servers are separate.
5. An agent without `locus lsp` in its allowlist cannot invoke it.
6. `--json` output is compact and parses.
7. A server crash is restarted by the supervisor without taking the pane with it.
8. A verb the server does not advertise returns `unsupported`, distinguishable from an empty result.

## Open

- **Semantic tokens have no implementation to import.** `spike-editor-embed` found that
  `@codemirror/lsp-client` ships no semantic-token support whatsoever — its surface is completion,
  hover, definition, references, rename, formatting and signature help, and that is the whole list.
  PLAN.md:2167 names LSP semantic tokens as *the* mitigation for languages with no Lezer grammar, and
  PLAN.md:2095 already routes them over `Channel<T>` as though they exist. Whoever owns this milestone
  writes `textDocument/semanticTokens/full`, its delta form, and the CodeMirror decoration layer, or
  the tail languages get no colour at all. **PLAN.md is wrong on this point and should be corrected
  rather than worked around.**
- Which language servers ship in a base image by default. PLAN.md makes them marketplace entries, so
  the honest answer may be none — but that makes `locus lsp` unavailable until an agent asks for it,
  which should be a deliberate choice rather than an accident.
