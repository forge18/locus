# F1 implementation status

F1 cannot be completed safely from the current runtime without inventing a proxy delivery mechanism. The existing Docker adapter only creates one agent container with `network_mode`; it has no network create/attach lifecycle, no sidecar launch abstraction, and no Docker build context for a vendored image. Changing the agent to an internal network without first launching a dual-homed proxy would break all model and research egress rather than enforce it.

Independent progress in the shared working tree:

- `crates/locus-core/src/sandbox/egress.rs` now owns `DestinationAllowlists` and tests Model/Packages/Open/None policy selection.
- `.specs/security/spec.md` records the chosen dual-network, vendored-sidecar topology, Open-tier research behavior, and no-default package registries.

Concrete implementation required next:

1. Add a vendored Linux proxy image and deterministic Docker build context.
2. Extend `ContainerRuntime` with ensure/create internal and egress networks, start/reuse a project proxy sidecar dual-homed across them, and remove it after its final run.
3. Inject only the sidecar endpoint into agent containers attached solely to the internal network.
4. Make the proxy authenticate each run, apply `DestinationAllowlists`, support HTTP/HTTPS CONNECT for Open research, audit decisions, and reject all non-proxy traffic.
5. Add Docker-backed lifecycle tests; current tests use `ContainerRuntime` fakes and cannot prove packet confinement.

```acceptance-report
{
  "criteriaSatisfied": [
    {"id":"criterion-1","status":"not-satisfied","evidence":"A real sidecar lifecycle cannot be added without the missing image-build and Docker-network abstractions; a proxy environment variable alone is bypassable."},
    {"id":"criterion-2","status":"satisfied","evidence":"The implementation seam and the policy-test evidence are recorded above."}
  ],
  "changedFiles": [
    ".specs/security/spec.md",
    "crates/locus-core/src/sandbox/egress.rs"
  ],
  "testsAddedOrUpdated": [
    "crates/locus-core/src/sandbox/egress.rs"
  ],
  "commandsRun": [
    {"command":"cargo test -p locus-core egress::tiers","result":"passed","summary":"Destination tier policy test passed."}
  ],
  "validationOutput": ["cargo fmt and git diff --check passed before the handoff."],
  "residualRisks": ["F1 remains unenforced: current agents still use a single unrestricted project network."],
  "noStagedFiles": true,
  "diffSummary": "Added destination policy and recorded the selected topology; no misleading proxy wiring was added.",
  "reviewFindings": ["blocker: crates/locus-core/src/runtime/container.rs: ContainerRuntime has no sidecar, Docker network lifecycle, or vendored-image build contract."],
  "manualNotes": "Do not mark F1 complete until the sidecar is running and agents are attached only to its internal network."
}
```
