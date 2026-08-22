# spike-sandboxed-harness — tasks

Experiment, not product code. Every task ends in evidence written to the spike directory.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Write `spikes/01-sandboxed-harness/QUESTION.md` — the four questions, the two harnesses under test, and what result falsifies the sandbox model | — | `test -s spikes/01-sandboxed-harness/QUESTION.md` |
| 2 | Dockerfile for `locus/base-claude`: OS, git, the harness CLI, a `detect` step that fails the build when the binary is absent | 1 | `docker build -t locus/base-claude spikes/01-sandboxed-harness/claude` |
| 3 | Prove `detect` fails loudly — build with the install line removed and confirm a non-zero exit | 2 | `! docker build -t locus/detect-fail -f spikes/01-sandboxed-harness/claude/Dockerfile.nodetect spikes/01-sandboxed-harness/claude` |
| 4 | Bare local remote + container-local clone into `/workspace`, no mount | 2 | `docker run --rm locus/base-claude sh -c 'git -C /workspace rev-parse --is-inside-work-tree'` |
| 5 | Candidate A — host credential proxy: sentinel in the container, real key injected on the way out | 2 | `bash spikes/01-sandboxed-harness/try-proxy.sh` |
| 6 | Candidate B — short-lived token minted per run over `/run/locus.sock` | 2 | `bash spikes/01-sandboxed-harness/try-broker.sh` |
| 7 | Candidate C — env var injected at container start, as the honest baseline to beat | 2 | `bash spikes/01-sandboxed-harness/try-env.sh` |
| 8 | Scan the winning candidate's container for a persisted credential: env, filesystem, harness config dir | 5,6,7 | `bash spikes/01-sandboxed-harness/scan-secrets.sh` |
| 9 | Capture a full session's hook events and normalize them to the canonical vocabulary | 4,8 | `jq -e 'map(.kind) \| index("session_end")' spikes/01-sandboxed-harness/out/claude.events.json` |
| 10 | Confirm `usage` carries real input/output/cache numbers, not nulls | 9 | `jq -e '[.[] \| select(.usage.input > 0)] \| length > 0' spikes/01-sandboxed-harness/out/claude.events.json` |
| 11 | Repeat 2, 4, 9 for a second capture source — `cursor` (ACP) or `antigravity` (stream-json) | 9 | `jq -e 'map(.kind) \| index("tool_call")' spikes/01-sandboxed-harness/out/second.events.json` |
| 12 | Attempt `dsh` against a real binary; record UNVERIFIED with a reason if unavailable | 11 | `grep -Eq 'dsh.*(VERIFIED | UNVERIFIED)' spikes/01-sandboxed-harness/FINDINGS.md` |
| 13 | Write `FINDINGS.md`: the verdict per question, what the container held, the falsifier, the fallback | 8,10,11,12 | `bash spikes/01-sandboxed-harness/check-findings.sh` |
