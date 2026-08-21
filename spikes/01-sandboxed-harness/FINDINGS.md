# Spike 1 — sandboxed harness · FINDINGS

**Status** Q1, Q2 and Q4 answered and evidenced. **Q3 is NOT EXERCISED** — see its section; it needs
one live model call and the credential to make it has not been supplied. Nothing below is graded on
intent: every verdict names the script that produced it and the artefact it wrote.

Questions and falsifiers were fixed in [QUESTION.md](QUESTION.md) **before** any of this ran.

| | Question | Verdict |
| --- | --- | --- |
| Q1 | Auth — a mechanism that keeps a long-lived secret out of the container | **Candidate A, the host credential proxy** |
| Q2 | Image — does `detect` fail the build when the binary is missing | **Yes** |
| Q3 | Events — canonical vocabulary, and `usage` with real numbers | **NOT EXERCISED** |
| Q4 | Clone — `/workspace` as a container-local clone, no mount | **Yes, including push-back** |

Reproduce all of it: `bash try-proxy.sh && bash try-broker.sh && bash try-env.sh && bash try-cred-kinds.sh
&& bash try-clone.sh && bash scan-secrets.sh && bash verify-harnesses.sh`.

---

## Q1 — Auth

**VERDICT: candidate A, the host credential proxy. The container holds a sentinel and a base URL, and
nothing else.** Candidate B is A plus a token-minting step and is kept as the mechanism for
*revocation*, not as an alternative. Candidate C is refused.

### What each candidate cost, measured the same way

Exposure numbers come from `scan-secrets.sh`, which runs one scan across all three containers in a
single invocation so the counts are comparable. It scans process environment, every PID's `environ`,
every PID's `cmdline`, the harness config dir, the workspace, the temp dirs, and finally every readable
text file in the container.

| | Candidate | Container holds | Exposure window | Revocable mid-run | Setup cost | Policy + audit at the same point | Real credential found |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **A** | Host credential proxy | a sentinel and a base URL | **none** | yes, by stopping the daemon | one host daemon per machine, nothing per project | yes | **clean** |
| **B** | Per-run token over `/run/locus.sock` | nothing at start; a TTL-bounded token once it asks | the token's TTL | **yes, per run, immediately** | A's cost, plus a harness that will fetch its own credential | yes | **clean** |
| **C** | Env var at container start | the real, long-lived credential | the entire life of the container | **no** | none | **no** | **present**, at `/proc/1/environ` |

`try-proxy.sh` 7/7 · `try-broker.sh` 9/9 · `try-env.sh` 4/4 · `try-cred-kinds.sh` 10/10.
Machine-readable in `out/*.result.json` and `out/exposure.json`.

### Why A rather than C

C is not a straw man and QUESTION.md said it had to be beaten on its own terms. It was, on three counts
that are not preferences:

1. **The credential is readable by every process in the container, for the whole run.** `try-env.sh`
   check C3 shows it survives being unset in a child shell, because it lives on PID 1 — so
   `/proc/1/environ` hands it to anything that can read it. Under PLAN.md §Credentials' lethal-trifecta
   framing, an agent reading a repo and browsing the web is precisely the process that must not also be
   holding the key.
2. **There is no revocation.** Rotating means killing the run.
3. **The call never crosses a chokepoint**, so there is no egress policy and no audit row —
   `try-env.sh` C4 asserts the audit log is empty, which is the point. PLAN.md puts policy tiers and
   per-call audit at the same place as injection; C has no such place.

A costs one host daemon and nothing per project, which was the other half of the requirement — "setting
one up should not be a per-project chore".

### Both ways in, not a choice between them

A credential design that serves only API keys serves only half the users: most people running Claude
Code are on a subscription and have no API key at all, and anyone driving it from CI has a key and no
browser. `set-credential.sh` offers both entry paths and they land at one chokepoint:

```
  ./set-credential.sh api-key  ─┐
                                ├─►  host store (outside any repo, mode 0600)  ─►  proxy injects
  ./set-credential.sh sign-in  ─┘
```

**VERDICT: one chokepoint carries both credential kinds, and the container's configuration is identical
for both.** `try-cred-kinds.sh`, 10/10:

- **K1** an `api_key` credential injects as `x-api-key` and the upstream accepts it.
- **K2** an `oauth` credential injects as `authorization: Bearer` with the `oauth-2025-04-20` beta
  header — **while the container still presents `x-api-key`**. The header on the wire is chosen by the
  credential the *host* holds, which is what makes a subscription user's run indistinguishable from an
  API-key user's from inside the container.
- **K3** an expired access token is refreshed on the host before injection; the container sees a 200 and
  never learns a refresh happened.
- **K4/K5** neither the access token nor the refresh token is present in the container, and the refresh
  token appears nowhere in the audit log.

The sign-in flow itself is not re-implemented here — `set-credential.sh sign-in` takes a token from an
existing Claude Code login. What the spike proves is the part that was in doubt: that an OAuth pair can
live host-side and be injected and refreshed without the container ever holding either half.

### What the container held, exactly, and for how long

Stated in the terms acceptance criterion 3 asks for.

- **Candidate A:** `ANTHROPIC_API_KEY=sk-locus-sentinel-000…0` and
  `ANTHROPIC_BASE_URL=http://host.docker.internal:43800`, for the life of the container. The sentinel is
  worth nothing anywhere else: `try-proxy.sh` A2 presents it directly to the upstream and gets 401.
- **Candidate B:** nothing at container start (`try-broker.sh` B1). After `locus creds get`, a token in
  `/tmp/creds.json` valid for its TTL — refused directly by the upstream (B3), killed on revoke (B4),
  and refused after expiry (B5).
- **Candidate C:** the real credential, on PID 1, for the life of the container.

The audit log records the credential **class** — `sentinel`, `run-token:valid`, `real-credential` — and
never a value. `try-proxy.sh` A5 asserts zero occurrences of the credential in the log.

### Egress policy at the injection point — the Open question, closed

QUESTION.md left open whether egress tiers belong at the same chokepoint as credential injection.
PLAN.md said they do.

**VERDICT: they do, and it is nearly free.** `locusd.mjs` carries four tiers (`none`, `model`,
`packages`, `open`) as an allowlist consulted on the same request that performs injection.
`try-proxy.sh` A5 flips the tier to `none` and the same call that returned 200 returns 403, refused
before any credential is written. One audit row per outbound call falls out of the same code path.

One consequence worth naming: **a container that presents an unrecognized credential has it forwarded
unchanged, never upgraded.** Otherwise the audited path would launder an agent's own smuggled key.

### The honest weakness this found: `/run/locus.sock` on macOS

PLAN.md's mount table gives the agent a rw `/run/locus.sock`, and §The one surface makes that socket the
only thing an agent talks to. **On this machine that mount is not possible.** Docker here is colima with
`mountType: virtiofs`, and bind-mounting a host unix socket fails outright:

```
error while creating mount source path '…/probe.sock': operation not supported
```

The agent-facing surface is unchanged — `locus-sockd` runs `socat UNIX-LISTEN:/run/locus.sock` inside
the container and relays to a host-local TCP port, so `locus` still speaks to a unix socket at the path
PLAN.md specifies. **The transport underneath is weaker and this is not a detail.** A bind-mounted
socket is authenticated by being mounted: only the container it was mounted into can reach it. A TCP
port on the host gateway is reachable by *every* container on the machine.

`LOCUS_RUN_NONCE` puts an authenticator back in that path, and `try-broker.sh` demonstrates both sides
rather than asserting one: **B6** a container without the run's nonce is refused; **B7** with
enforcement off, an impostor container mints a token freely. On Linux, where the bind mount works, the
nonce is defence in depth. On macOS it is load-bearing.

**`sandbox` must carry this as a platform difference, not absorb it.** A per-run nonce is required on
macOS and the socket path is not, on its own, the boundary there.

---

## Q2 — Image

**VERDICT: yes. `detect` fails the build, loudly, with a non-zero exit.**

`locus/base-claude` builds from `claude/Dockerfile`: Debian, git, the harness CLI installed with npm,
and `locus` / `locus-hook` / `locus-detect` / `locus-entrypoint` / `locus-sockd`. Config is not a layer —
it is materialized per run, per PLAN.md §Images.

The falsifier is the evidence. `claude/Dockerfile.nodetect` is the same file with **one** change, the
harness install layer removed, and the build fails at the detect step:

```
locus: detect FAILED — harness binary 'claude' is not present in this image.
locus: the base image is the reproducibility boundary; a missing binary is a build error,
locus: not a run-time surprise. Fix the install layer, do not skip this step.
The command '/bin/sh -c locus-detect "${HARNESS_BINARY}" "${HARNESS_DETECT}"' returned a non-zero code: 78
```

`locus-detect` checks two things, not one: that the binary resolves, and that the harness's declared
`detect` argv exits zero. A binary that is present but broken fails the build the same way.

Built and detected: `locus/base-claude` (1.14GB), `locus/base-hermes` (623MB), `locus/verify-dsh`,
`locus/verify-hermes`.

### A correction the mount table needs

**`/locus/config` cannot be read-only for every harness.** PLAN.md's mount table says ro, and for a
harness that only reads its config home that is right. Claude Code's is not one: with
`CLAUDE_CONFIG_DIR` pointed at it, the same directory holds its transcripts — `harnesses/claude.toml`
declares `log_dir = "/locus/config/projects"` — plus its todo state and `.claude.json`. A ro mount stops
the harness starting.

The spike mounts the materialized tree ro at **`/locus/config-ro`** and the entrypoint copies it into a
writable `/locus/config`. The property that matters survives: the *source* tree is byte-identical per
run, and that is what the prompt prefix is built from. `sandbox` and `materializers` should adopt the
two-path shape rather than rediscovering it.

---

## Q3 — Events

**VERDICT: NOT EXERCISED.**

Recorded as not exercised rather than inferred, because a spike that reported captured events from a
mock would be answering a question nobody asked. What is missing is one thing only: a live model call,
which needs a credential the operator has not yet supplied.

**Everything around it is built and one command from running:**

| Piece | State |
| --- | --- |
| `locus-hook`, wired through `config/settings.json` for all nine Claude Code hook events | built |
| `normalize.mjs` — `hooks-claude`, `stream-json`, `acp` and `session-log` sources, all four mapping to the canonical vocabulary | built, syntax-checked |
| `run-session.sh` — a live claude run under candidate A, capture, normalize, then re-scan the container | built, gated on a credential |
| `run-second.sh` + `acp-client.mjs` — hermes over ACP, client on the host driving the container | built, gated on a credential |
| `locus/base-hermes` with the `[acp]` extra; `hermes acp --check` returns **"Hermes ACP check OK"** | built and verified |

To finish it:

```
./set-credential.sh api-key      # or: ./set-credential.sh sign-in
bash run-session.sh              # -> out/claude.events.json
bash run-second.sh               # -> out/second.events.json
```

Two things the normalizer already encodes, from PLAN.md, and which the live run will confirm or refute:

- **Hooks alone cannot produce `assistant`, `thinking`, or `usage`.** The hook stream sees tool calls
  with their arguments already separated but never the model's own output. Those three come from the
  transcript `harnesses/claude.toml` declares under `log_dir`, which is why that key sits beside
  `source = "hooks"`. **The `hooks` path is two sources merged, not one**, and `telemetry` should be
  built that way from the start.
- **`Stop` and `PreCompact` map to nothing** and are dropped rather than mapped onto something close,
  per "a missing verb is recorded as missing, never synthesized".

### The second harness, substituted with a reason

`.specs` named `cursor` (ACP) or `antigravity` (stream-json). Neither is obtainable here: `cursor-agent`
needs a Cursor account token, and `antigravity` ships no public CLI package. **`hermes` is the
substitute and it is a better one than either** — it installs from PyPI, passes detect, and ships an
`acp` subcommand, so it is a genuinely different harness *on the ACP capture path the spec asked for*.

---

## Q4 — Clone

**VERDICT: yes, and the round trip works. `/workspace` is a container-local clone with no mount, and the
agent's branch reaches the host remote while `main` never moves.**

`try-clone.sh` builds a bare remote on the host, serves it over `git daemon`, and runs a container with
**no `-v` and no `--mount`**:

```
workspace-is-worktree: true
workspace-head-sha:    1a542018d6c0d5ce3f535cc972bba849bc563201   (= the host commit)
workspace-branch:      agent/clone-51790
workspace-is-mount:    no                                        (findmnt finds nothing)
pushed:                agent/clone-51790
remote-branches:       agent/clone-51790 main
PASS: clone from host bare remote, no mount, branch pushed back, main untouched
```

Four assertions, each on a claim PLAN.md makes:

- the container's HEAD equals the host commit → it really cloned, rather than finding a fixture;
- `findmnt` reports nothing at `/workspace` → §The git model's "an agent cannot touch your working copy,
  because it does not have it" holds literally;
- `agent/<run-id>` exists on the host remote → merge-back is a push, not a bespoke path;
- the remote's `main` is unmoved → "Locus never works in main/master" is enforced, not documented.

The last is enforced in `locus-entrypoint`, which refuses to exec the harness at all if HEAD resolves to
`main` or `master`.

The task-13 one-liner (`docker run --rm locus/base-claude sh -c 'git -C /workspace rev-parse
--is-inside-work-tree'`) passes standalone against a bare fixture baked into the image, so the claim is
checkable with no host setup.

---

## dsh and hermes — the other half of this spike

PLAN.md:2248 makes this Spike 1's second job. Both were marked UNVERIFIED and both harness files said
so. `verify-harnesses.sh` installs each in a container, runs its detect, and checks each TOML claim
against the binary. **15 claims checked: 9 VERIFIED, 4 REFUTED, 2 previously unrecorded.** Full record
in `out/harness-verify.json`.

### dsh — VERIFIED, with two refuted fields

Installed from `@deepseek-ai/dsh@0.1.0-rc.7`; `dsh --version` exits 0.

| Claim | Verdict |
| --- | --- |
| `binary = "dsh"` | **VERIFIED** |
| `detect = ["--version"]` | **VERIFIED** — prints `0.1.0-rc.7` |
| `home_env = "DSH_HOME"` | **VERIFIED** — `$DSH_HOME/profiles` is the documented profile root |
| `tui = false` | **VERIFIED** for the headless profile |
| `[launch] argv = []` | **REFUTED** |
| `[models] flag = "--model"` | **REFUTED** |
| `@deepseek-ai/dsh-hooks-claude-code` | **PUBLISHED** at `0.0.1-rc.5`, not exercised |
| `@deepseek-ai/dsh-tool-subagent` | **PUBLISHED** at `0.0.1-rc.1`, not exercised |

- **`argv = []` is wrong.** Bare `dsh` exits with `--profile <name> is required`. The headless entry
  point is `dsh --profile headless "<task>"` — "answer one task, print the final assistant message, and
  exit", which is exactly the one-session-one-terminal shape. `[launch] argv` should be
  `["--profile", "headless"]`.
- **`--model` does not exist.** Neither the launcher nor the headless profile accepts it. The model is
  profile configuration: the composed tree carries `@deepseek-ai/dsh-agent-default-model` with
  `provider` and `model` in its config, so selection is a `--patch` overlay. `harness-registry`'s model
  resolution needs a strategy that is not a flag for this harness, which is a registry-shape question,
  not a dsh question.
- **A `tui` profile also exists.** `tui = false` is therefore an assertion about the *launch
  configuration*, not about the binary — which is PLAN.md's "the registry enforces it, not the harness",
  now with a concrete case behind it.

### hermes — VERIFIED, with its hook mechanism refuted

Installed from PyPI `hermes-agent[acp]`; `hermes --version` prints `Hermes Agent v0.19.0 (2026.7.20)`.
The `portal` subcommand ("Set up Nous Portal") confirms it is the Nous Research agent the file names.

| Claim | Verdict |
| --- | --- |
| `binary = "hermes"` | **VERIFIED** |
| `detect = ["--version"]` | **VERIFIED** |
| `[models] flag = "--model"` | **VERIFIED** — `-m MODEL, --model MODEL` |
| `[memory] native = "~/.hermes/memories/"` | **VERIFIED** — present on a fresh install, beside `hooks/`, `sessions/`, `skills/`, `SOUL.md` |
| `tui = false` | **VERIFIED**, but only with an explicit flag |
| `[launch] argv = ["chat", "--query-file", "-"]` | **REFUTED** |
| `[hooks] generated = "…/plugin.yaml"` | **REFUTED** |

- **There is no `--query-file`.** The non-interactive form is `hermes chat --cli -Q -q "<prompt>"`:
  `-q/--query` is "Single query (non-interactive mode)", `--cli` selects the non-TUI frontend, `-Q` is
  "Quiet mode for programmatic use".
- **`tui = false` needs `--cli` in argv.** Bare `hermes chat` is interactive; `--tui` and `--cli` are
  both explicit flags. Omitting `--cli` gets a TUI, which the registry refuses.
- **Hooks are not a generated Python plugin.** `hermes hooks` describes itself as inspecting
  "**shell-script hooks** declared in `~/.hermes/config.yaml`". This is `entries-in` against
  `config.yaml` — the same shape as claude's `settings.json` — **not** the pi/omp generated-extension
  shape. hermes therefore needs no materializer plugin for hooks, and `materializers` has one fewer
  plugin to write than PLAN.md assumes.
- **New, and a trap: hooks have a first-use consent allowlist** at
  `~/.hermes/shell-hooks-allowlist.json`. Nobody is attached in a container, so a run must pass
  `--accept-hooks` (or `HERMES_ACCEPT_HOOKS=1`) or **the hooks silently never fire** — telemetry would
  read as an agent that did nothing. `locus/base-hermes` sets the env var for exactly this reason.
- **New: hermes ships an `acp` subcommand.** `telemetry.source` is a choice for this harness, not a
  constraint, and it is why hermes can serve as the ACP second capture source.

---

## What would falsify the sandbox model, revisited

The five falsifiers were fixed in QUESTION.md before anything ran.

| | Falsifier | Result |
| --- | --- | --- |
| 1 | No mechanism keeps a long-lived credential out of the container | **Not triggered.** A and B both scan clean, for both credential kinds |
| 2 | The harness cannot be redirected | **Not triggered for claude** — `ANTHROPIC_BASE_URL` is honoured by construction in candidate A. **Open for hermes**: `run-second.sh` records which base-URL variable it honours, and that run has not happened |
| 3 | `detect` cannot fail a build | **Not triggered.** Exit 78, with the reason |
| 4 | The clone model does not hold | **Not triggered.** Clone, no mount, push-back, `main` unmoved |
| 5 | `usage` is unavailable or fabricated | **UNRESOLVED.** This is Q3, and it is the one falsifier still live |

**The fallback, if 5 does trigger:** `usage` is null for that harness and spend reads *unknown*, per "a
missing verb is recorded as missing, never synthesized". The cost is named in PLAN.md and is not small —
agent trust is weighted by tokens per passing run, and the dashboard cannot tell a good run from an
expensive one without the number. It does not invalidate the container model; it invalidates the
dashboard's claim to rank runs.

**The fallback, if 2 triggers for hermes:** that harness runs under candidate C with its exposure
recorded as accepted risk, or it is not supported at M1. It does not change the model for harnesses that
can be redirected — which is the honest reason two harnesses were required rather than one.

---

## What this spike changes elsewhere

Each is a concrete edit, not an observation.

| Where | Change |
| --- | --- |
| PLAN.md §Containers, mount table | `/locus/config` cannot be ro for every harness. Two paths: ro source at `/locus/config-ro`, writable `/locus/config` |
| PLAN.md §Credentials | Name both entry paths — API key and sign-in — and the host store. The proxy chooses the outbound header from the credential's kind |
| `.specs/sandbox` | A per-run nonce is required wherever `/run/locus.sock` cannot be bind-mounted. Platform difference, not an implementation detail |
| `.specs/sandbox` | Egress tiers confirmed at the injection chokepoint. `none` / `model` / `packages` / `open` are the tier names the spike used |
| `.specs/telemetry` | The `hooks` path is two sources merged — hook stream plus transcript. `assistant`, `thinking` and `usage` come only from the transcript |
| `harnesses/dsh.toml` | `[launch] argv = ["--profile", "headless"]`; `[models]` is not a flag; drop the "not installed on this machine" comment |
| `harnesses/hermes.toml` | `[launch] argv = ["chat", "--cli", "-Q", "-q"]`; hooks are `entries-in` against `config.yaml`, not a generated plugin; record the consent allowlist |
| `.specs/materializers` | One fewer plugin: hermes hooks need no generated extension |
| `.specs/harness-registry` | Model resolution needs a non-flag strategy — dsh selects its model through a profile patch |

## Artefacts

| File | What it holds |
| --- | --- |
| `out/exposure.json` | the per-candidate scan verdict, with candidate C as the positive control |
| `out/{proxy,broker,env,cred-kinds}.result.json` | each candidate's checks, machine-readable |
| `out/harness-verify.json` | all 15 dsh/hermes claims with verdict and observation |
| `out/*.audit.ndjson` | one row per outbound call: tier, decision, credential class, never a value |
| `out/clone/container.log` | the no-mount clone transcript |
