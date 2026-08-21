# harness-registry

**Milestone** M1 · **Depends on** `store` · **Blocks** `materializers`, `run-supervisor`, `telemetry`

## Purpose

Load `harnesses/*`, validate them, and resolve a model tier into an actual model. This is where
PLAN.md's first carried-over rule lives: **nothing in core names a harness.** Adding one is a TOML
file, plus a materializer plugin only where that harness's config is code.

The twelve TOMLs are already written. This feature is the loader, the validator, and the tier policy.

## Governed by

- PLAN.md §Harness contract — every entry complete, nothing inherited
- PLAN.md §Model routing — mechanism in the file, policy in the UI
- PLAN.md §Containers — the registry enforces `tui = false`, not the harness

## Contract

**Schema.** `name`, `binary`, `detect`, `[launch]`, `[telemetry]`, `[models]`, `[layout]`, and where
present `[config]` and `[auth]`.

**`[layout]` must declare all eight extensions.** An omitted extension is refused — PLAN.md names the
worst failure as a file present, plausible, and loaded by nobody, and an undeclared extension is how a
capability silently fails to arrive. Where a harness has no native mechanism the entry **says so**
rather than being left out, and carries `weaker_than_native` naming the loss.

**`tui = false` is required and enforced here.** A harness file claiming `true` is refused at
registration. That is why the field is required rather than defaulted.

**`locus harness lint`** refuses: an undeclared extension, an unknown `via` strategy, `tui = true`, a
missing `[telemetry].source`, a source outside `hooks | acp | stream-json | session-log`, and a
downgrade with no `weaker_than_native`.

**Model routing.** The file carries only mechanism:
```toml
[models]
flag      = "--model"
list_argv = ["models", "list"]   # optional
```
Policy is `core.settings`, keyed by harness and tier. Three rules:
- **A missing tier falls back UP, never down.** `xhigh` on a harness with no `xhigh` gets `high`.
  Falling down would answer a hard question with a cheap model and read as a bad agent rather than a
  bad setting.
- **Unset means the harness's default** — no `flag` is passed, so a newly registered harness runs.
- **The resolved model id is recorded on the run**, not the tier.

**No `[capabilities]` block and no `[model_routing]`.** What a harness can do is universal; which model
a tier means is a setting.

## Acceptance

1. All twelve TOMLs load and pass `locus harness lint`.
2. A file with an omitted extension is refused, with a message naming the extension.
3. A file with `tui = true` is refused at registration.
4. A downgrade without `weaker_than_native` is refused.
5. `grep -rn` over `crates/locus-core/src` finds **no harness name** outside the registry loader's
   tests and fixtures.
6. Tier resolution falls back up; a test asserts `xhigh → high` and that no path ever falls down.
7. An unset tier passes no `flag`, and the run still starts.
8. The resolved model id lands on the run row.
9. `list_argv` populates the tier combobox; a harness without it accepts free text.

## Open

- `dsh` and `hermes` are UNVERIFIED against running binaries and their files say so. Confirming them is
  Spike 1's other half; until then the registry loads them and the lint passes on declaration alone.
