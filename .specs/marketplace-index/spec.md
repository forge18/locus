# marketplace-index

**Milestone** M4 · **Depends on** `agent-definitions` · **Blocks** `marketplace-installer`

## Purpose

The resolver, not the installer. Agents need the *index* long before they need image baking: reading
manifests to validate an agent's `tools` list and inject docs is cheap and lands here; baking and
install land at M8. Workshop ships only the GitHub CLI (`gh`) as a first-party tool plugin; the index
remains open to trusted user-authored tool plugins.

The whole point is **just-in-time knowledge**: a one-line catalog per allowlisted tool, with the page
arriving only when an agent asks.

## Governed by

- PLAN.md §Marketplace — the manifest, the catalog, the two-milestone split
- PLAN.md §Token discipline #4 — summaries with handles, never bodies

## Contract

A git repo of manifests, one per CLI:

```toml
name    = "gh"
summary = "GitHub's official command-line interface"
install = { brew = "gh" }
verify  = "gh --version"
docs    = "docs/gh.md"
caps    = ["source-control", "github"]
```

User-authored manifests use the same schema and trust boundary. A manifest is not first-party merely
because it parses; only `gh` is seeded in the built-in catalog.

**Until M8 the index is a local directory of manifests**, read from disk. Where it is hosted, how it is
pinned, and who is trusted to publish are questions the installer makes real and the resolver does not.

**Catalog, not content.** Locus injects **name plus one line, roughly 15 tokens each**, and every body
is fetched on demand through `locus tools docs <name>`. Fifteen allowlisted tools cost about 225 tokens
instead of 3,000, and the difference is recovered by any agent that actually reads a page.

The line an agent needs to *choose* a tool is short: what it does and when to reach for it. The page it
needs to *use* one — flags, output shape, examples — is only worth its tokens once the choice is made.

**This is the same move the field made for MCP** under three names — Anthropic's Tool Search Tool,
Cloudflare's Code Mode, the MCP-code-execution pattern — all versions of *stop loading tool definitions
you aren't using*, against a reported 55K+ tokens of schema consumed before work begins. Reaching it
from a CLI is a catalog line and a `docs` verb rather than an architecture.

**Installation stays eager, deliberately.** A tool absent from the allowlist is absent from the image,
because that is a **privilege boundary** rather than a context decision. Just-in-time applies to what an
agent *knows*, never to what it *can reach*.

**Blurbs are a tuning surface.** Anthropic reported a 40% cut in task completion time from having an
agent evaluate and rewrite tool descriptions, which makes `summary` and `docs` worth iterating on. The
index is git-backed, so a better description is a commit — and the event store already holds what it
would be measured against.

## Acceptance

1. The built-in manifest parses as `gh`; trusted user manifests parse from a local directory and
   validate against the same schema.
2. An agent's `tools` list resolves against the index; an unresolvable name fails at save with the name.
3. The injected catalog is **name plus one line only** — no flags, no examples, no schema.
4. Catalog cost is measured: fifteen tools land near 225 tokens, asserted rather than assumed.
5. `locus tools docs <name>` returns the full page on demand.
6. `locus tools list` shows only what is allowlisted.
7. A tool not in the allowlist cannot be reached, even if it exists in the index.
8. Docs for a tool are injected only when it is allowlisted.

## Open

- **Curation versus selection** — a vetted catalog with quality guarantees, or an open index where
  manifests compete and usage data ranks them. Locus already collects the usage data, which points at
  selection, but PLAN.md defers the argument to M8 with the installer in front of it.
