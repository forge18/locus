# workshop-plugins

**Milestone** M1.5 · **Depends on** `workshop-revision`, `harness-registry`, `marketplace-index` · **Blocks** plugin-backed Workshop runtime work

## Purpose

Narrow Workshop's first-party surface without closing the extension point. Workshop has two subgroups:

- **Plugins** — `CLI Tool`, `Harness`, and `Provider`.
- **Extensions** — the existing extension editor and Workflows. The extension editor remains the same
  eight types: agents, skills, rules, base-context, commands, hooks, output-styles, and linters.

Plugins are executable integrations. Extensions remain Locus-authored content materialized by core.
The first-party roster is intentionally small; users can add other plugins without a Locus core change.

## Governed by

- `PLAN.md` §Plugins — one manifest and one JSON-RPC 2.0 executable over stdio
- `PLAN.md` §The one surface — extensions are authored once and materialized per run
- `PLAN.md` §ACP — agent sessions normalize to the ACP event surface
- `.specs/workshop-revision/spec.md` — the Extensions subgroup and shared editor
- `.specs/harness-registry/spec.md` — harness launch, layout, and selection invariants
- `.specs/marketplace-index/spec.md` — CLI manifest and allowlist boundaries

## First-party scope

Only these plugins ship with Locus for now:

| Plugin kind | First-party plugin | Canonical identity |
| --- | --- | --- |
| CLI Tool | GitHub CLI | `gh` |
| Harness | Pi | `pi` |
| Provider | OpenAI API / ChatGPT models | `openai` |
| Provider | Claude models | `anthropic` |
| Provider | OpenRouter | `openrouter` |

"ChatGPT" means the OpenAI API provider and its ChatGPT model catalog. A separate ChatGPT consumer-app
login integration is not part of this scope.

The first-party roster is an allowlist of shipped plugins, not a limit on the architecture. User plugins
may add other CLI tools, harnesses, and providers through the same admission and capability contract.
No user plugin may add UI code; the first-party UI renders manifest and RPC data through known views.

`forge-providers` is a separate remote-forge integration boundary, not a Workshop model-provider plugin.
Its provider-neutral port is not expanded or renamed by this scope reduction.

## Common plugin contract

Every plugin has a signed or explicitly trusted manifest and one executable speaking JSON-RPC 2.0 over
stdio. The executable is a separate process; it has no direct Postgres, Tauri, or filesystem authority
outside the capability request it is handling.

The common manifest envelope is deliberately small:

```toml
protocol = "locus.plugin.v1"
kind = "harness"             # cli_tool | harness | provider
id = "pi"
version = "1.0.0"
executable = "locus-plugin-pi"
capabilities = ["harness.session", "harness.materialize"]
permissions = ["model_catalog"]
```

The host performs this lifecycle before any kind-specific call:

1. `plugin.initialize` — negotiate protocol version and return the plugin descriptor.
2. `plugin.describe` — return kind, identity, version, capabilities, and schema versions.
3. `plugin.health` — return readiness and a bounded diagnostic summary.
4. `plugin.shutdown` — terminate cleanly at the end of the host session.

Capability calls are namespaced (`harness.*`, `provider.*`, or `cli_tool.*`) and carry typed JSON
objects. The host rejects malformed or required-unsupported capabilities, ignores optional capabilities
it does not know, bounds every call, and never infers behavior from a plugin name. Responses are data;
the first-party UI owns rendering.

### Harness flexibility

The harness contract is capability-based rather than a fixed method list. A harness descriptor must
expose the minimum session capability needed for selection and declare its transport, launch, config,
and event capabilities. Optional capabilities may include model discovery, permission handling, resume,
checkpoints, usage reporting, or materialization. A harness may implement native ACP, an ACP adapter, or
a materializer-backed configuration; the host maps all supported session events to the canonical ACP
vocabulary.

Adding a harness capability must not require a new core match on the harness id. Core validates the
capability schema, routes the request, and preserves unknown optional data for diagnostics. This keeps
Pi's plugin useful as the reference implementation without making the contract brittle for user-written
harnesses.

### Kind-specific boundaries

- **CLI tools** declare install, verify, documentation, digest, and runtime permission metadata. The
  existing Minisign and image-allowlist gates remain mandatory. `gh` is the only first-party tool.
- **Providers** declare model discovery, verification, endpoint/authentication metadata, and model
  aliases. Credentials remain OS-keychain references resolved by the host broker; raw secrets never
  cross the plugin or persistence boundary. First-party providers are `openai`, `anthropic`, and
  `openrouter`.
- **Harnesses** declare launch/session/materialization/event capabilities. The host owns run identity,
  container placement, ACP normalization, and policy; the plugin owns harness-specific mechanics.
  `pi` is the only first-party harness.

## Acceptance

1. Workshop navigation exposes exactly `Plugins` and `Extensions` subgroups.
2. Plugins exposes only CLI Tool, Harness, and Provider; Extensions exposes the existing eight editor
   types plus Workflows.
3. The shipped plugin catalog contains only `gh`, `pi`, `openai`, `anthropic`, and `openrouter`.
4. A user plugin with a valid trusted manifest can be discovered without a core source change.
5. The host rejects a protocol/version or required-capability mismatch before dispatching a call.
6. Unknown optional capabilities do not prevent a plugin from loading and remain available in diagnostics.
7. Every plugin call is JSON-RPC 2.0 over stdio, bounded, and data-only; plugins cannot render UI or
   write directly to Locus persistence.
8. A harness can declare optional capabilities without changing a central harness-name match, and all
   supported session events arrive through the canonical ACP vocabulary.
9. Provider credentials remain keychain references, and CLI tools remain signature- and allowlist-gated.
10. A non-first-party plugin is either a user-installed plugin or absent; it is never silently treated as
    a built-in integration.

## Open

- The exact signature/trust store UX for user plugins remains owned by the marketplace installer. The
  admission rule is not open: untrusted executable plugins are refused.
- The first-party plugin executable packaging and version pins are implementation details; the manifest
  and capability envelope are the compatibility contract.
