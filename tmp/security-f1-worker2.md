F1 cannot be safely implemented as a partial sidecar: a Dockerfile/proxy without per-run authentication, policy delivery, internal-network lifecycle, and a deterministic pinned build is a bypassable security control. No F1 production files were retained from this attempted implementation step.

```acceptance-report
{
  "criteriaSatisfied": [{"id":"criterion-1","status":"not-satisfied","evidence":"No safe complete F1 implementation was possible in this run."},{"id":"criterion-2","status":"satisfied","evidence":"Existing policy tests and the missing runtime seams were verified."}],
  "changedFiles": ["crates/locus-core/src/sandbox/egress.rs", ".specs/security/spec.md"],
  "testsAddedOrUpdated": ["crates/locus-core/src/sandbox/egress.rs"],
  "commandsRun": [{"command":"cargo test -p locus-core egress::tiers","result":"passed","summary":"Tier destination policy test passed."}],
  "validationOutput": ["cargo fmt and git diff --check passed before this handoff"],
  "residualRisks": ["F1 remains unenforced until the authenticated dual-network sidecar lifecycle exists."],
  "noStagedFiles": true,
  "diffSummary": "Recorded selected F1 topology and added destination policy only.",
  "reviewFindings": ["blocker: ContainerRuntime has no sidecar/network lifecycle or deterministic vendored image build contract."],
  "manualNotes": "Do not ship a proxy image without per-run authentication and internal-only agent networks."
}
```
