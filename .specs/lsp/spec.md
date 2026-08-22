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

**Provision when a repository joins a project, not on first use.** Locus detects trusted root markers
and file extensions across the project's repositories, resolves the matching descriptors, and prepares
the host-server cache and agent-image layers before an editor pane or agent run needs them.

**Locus owns the language catalog.** Language descriptors are not marketplace plugins. Built-in
descriptors ship in app resources; a user may import a local descriptor bundle into the user catalog.
Each import is schema-validated, copied immutably, and recorded by content hash.

**Project activation is explicit and pinned.** Detection may suggest a built-in or imported descriptor,
but a repository never imports or executes one. Enabling a descriptor records its id, version, and
content hash in project state. Host and container provisioning use that frozen descriptor until the
user selects a replacement.

**`--json` on every verb**, compact, because the caller is a model.

**A language is a descriptor, not a branch in core.** Same rule the harness registry already lives
under — nothing in `crates/locus-core` names a harness, and nothing in it names a language either.
Adding Odin is a catalog entry plus, where the server needs it, a provisioned binary; a `match` on a
language name in core is a bug. This is what makes "support a lot of languages" a data problem rather
than a release problem, and it is why the per-language pieces below are declared per entry rather
than assumed:

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
1. A built-in descriptor and a locally imported descriptor pass the same schema validation.
2. An imported descriptor is copied immutably into the user catalog and addressed by content hash.
3. Repository detection suggests matching descriptors but never imports a repository-provided bundle or
   executes a repository-controlled installer.
4. Enabling a descriptor records its id, version, and hash in project state; an edited catalog source
   cannot change an enabled project until the user selects it again.
5. Detected, enabled descriptors are provisioned for the host server cache and agent-image layer before
   the first editor pane or agent run needs them.
6. The editor gets completions and diagnostics from a host-supervised server.
7. One server set per project is shared across panes rather than one per pane.
8. `locus lsp def` in a container resolves a symbol against **that run's clone**.
9. The agent and the editor give the **same answer** for the same symbol when the trees match, and
   **different** answers when they diverge — the second half is what proves the servers are separate.
10. An agent without `locus lsp` in its allowlist cannot invoke it.
11. `--json` output is compact and parses.
12. A server crash is restarted by the supervisor without taking the pane with it.
13. A verb the server does not advertise returns `unsupported`, distinguishable from an empty result.
14. Semantic tokens render through CodeMirror decorations; a server without token support, or a language
    without a Lezer grammar, degrades to editable plain text rather than failing.

## Open

- **Semantic tokens must be implemented.** `@codemirror/lsp-client` ships no semantic-token support,
  so Locus owns `textDocument/semanticTokens/full`, delta handling, and the CodeMirror decoration
  layer. These are M2 tasks, not an imported fallback.
