# marketplace-installer

**Milestone** M8 · **Depends on** `marketplace-index`, `sandbox`

## Purpose

The half that puts tools in images. The index landed at M4 because agents need to *resolve* tools long
before they need to *install* them; this is where an agent's `tools` list becomes something in a
container. The only first-party Workshop CLI tool is `gh`; the installer remains the extension point for
trusted user-authored CLI-tool plugins.

## Governed by

- PLAN.md §M8 — image baking, install methods, allowlist enforcement, docs injection
- PLAN.md §Marketplace — installation stays eager, deliberately
- PLAN.md §Deliberately deferred — where the index is hosted

## Contract

Image baking from a manifest's `install` block, with `verify` confirming the tool actually landed.
Allowlist enforcement happens **at container build**, not at runtime.

**Installation stays eager, deliberately.** A tool absent from the allowlist is absent from the image,
because that is a **privilege boundary** rather than a context decision. Just-in-time applies to what an
agent *knows* — the catalog — never to what it *can reach*.

**This is what makes the tool allowlist a real boundary** rather than a policy an agent could ignore. It
is the same enforcement point the sandbox already relies on: an unlisted tool is not installed, so there
is nothing to enforce at runtime.

**Hosting, pinning and trust are settled here, not before.** PLAN.md defers them to this milestone
because the installer is what makes them real. The axis to decide on is **curation versus selection** —
a vetted catalog with quality guarantees, or an open index where manifests compete and usage data does
the ranking. Locus already collects the usage data, which points at selection, but that is an argument
to have with the installer in front of us.

## Acceptance

1. Adding `gh` or a trusted user-authored tool plugin to an agent installs it in that agent's image,
   and `verify` confirms it.
2. A failing `verify` fails the image build rather than producing an image with a missing tool.
3. A tool not in the allowlist is **not present in the image** — asserted by looking, not by policy.
4. Docs for an installed tool appear in the agent's context as a catalog line, with the body on demand.
5. Two agents with identical tool lists still share one image after baking.
6. Changing one tool's pin rebuilds the image; changing the prose body does not.
7. The hosting, pinning and trust model is decided and written down before the first non-local index is
   used.
8. No CLI tool other than `gh` is presented as a first-party Workshop integration.

## Open

- **Curation versus selection**, still. PLAN.md names the axis and the evidence pointing at selection,
  but explicitly leaves the decision to this milestone.
