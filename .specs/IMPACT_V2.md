# v2 design-handoff impact

## Target

`docs/design_handoff_locus_v2/` supersedes the removed v1 handoff. It changes the application shell,
project scope model, planning-to-board handoff, dispatch control, runtime policy, credential ownership,
tool installation, harness routing, and workflow authoring.

## Dependents

- `PLAN.md` — navigation, credentials, tools, model routing, planning, guardrails, and workflows.
- `DESIGN.md` — the UI authority; its v1-derived shell and proposed decisions conflict with v2.
- `.specs/app-shell`, `.specs/navigation`, `.specs/design-system`, `.specs/fixtures` — shell and
  fixture contracts.
- `.specs/screens-*` — v1 screen inventory is replaced by 31 v2 screens.
- `.specs/harness-registry`, `.specs/sandbox`, `.specs/agent-definitions`, `.specs/materializers` —
  providers, aliases, routing, adapter gate, image tool sets, and extension selection.
- `.specs/planning-module`, `.specs/board`, `.specs/guardrails`, `.specs/workflow-engine`,
  `.specs/schedules` — decompose-to-card mapping, queue priority/caps, stop-all, and workflow
  governance.
- `.specs/memory`, `.specs/tool-compaction`, `.specs/dashboard-metrics` — the new Memory and Project
  Analytics views consume their existing data contracts.

## Affected stories

- M0.5 shell and all fixture screens need a v2 replacement rather than incremental visual changes.
- M1 must add provider/keychain ownership and image-tool configuration before harness runs can use the
  v2 routing model.
- M3–M6 must gain planning decomposition, dispatch policy, and workflow governance contracts before
  their UI surfaces are implementable.

## Test coverage

- Existing desktop fixtures and visual tests cover v1 views only; they do not cover v2’s project
  scope, provider configuration, dispatch state, plan decomposition, or workflow governance.
- Core tests cover the current harness TOML and agent tool allowlist, but not provider aliases,
  keychain references, project overrides, or priority scheduling.

## Risk: High

This is a cross-cutting product-contract change. The provider design directly contradicts PLAN.md’s
"Locus holds no model API keys" decision, and the project-scoped rail contradicts the existing
all-project filter navigation model.

## Decisions adopted

- Provider credentials remain OS-keychain references and reach a run only through the host broker;
  no provider secret enters a container.
- The selected-project rail replaces the all-project navigation filter.
- User-uploaded CLI tools must verify with Minisign against trusted public keys in Locus settings; unsigned or untrusted uploads are rejected.
- M0.6 ships both v2 Dark and a cool-neutral Light theme through one semantic-token contract.

## Recommended action

Adopt v2 as the visual authority through the dedicated M0.6 reconciliation milestone. Amend affected
feature contracts before implementation; do not port the HTML/JS authoring scaffold or retrofit v1
screen tasks.
