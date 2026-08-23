# Locus Architectural Review

Full-review pass over the entire repo — 17.5k Rust lines (core / CLI / Tauri host) and 13.3k
TypeScript — against the `tech-arch-review` skill references
(`references/rust.md` + `references/patterns-rust.md`, `references/typescript.md` +
`references/patterns-typescript.md`).

Four parallel full-module review passes ran and every load-bearing finding was independently
verified against source before inclusion. One pass was discarded as unreliable (see [Method
note](#method-note)) and its findings re-checked manually.

- **Date:** review-only, no files changed
- **Scope:** `crates/locus-core`, `crates/locus-cli`, `apps/desktop/src`, `apps/desktop/src-tauri`
- **Verdict:** no blockers. Gaps cluster in three places: **resource unboundedness**,
  **atomicity**, and **boundary leak**.

---

## Rust core

### Warnings

#### `locus-cli/src/sock.rs:397` — unbounded inbound allocation from untrusted length

`read_frame` does `vec![0; length as usize]` where `length` is a `u32` read directly off the wire.
A peer framing ~4 GiB forces a multi-GiB allocation (OOM / hang). No `MAX_FRAME` bound exists.

- **Root cause:** length read before any cap.
- **Fix:** cap the frame before allocation; `bail!` past it.
- **Note:** routes to `rust-review` per the security boundary (memory-exhaustion).

#### `runtime/normalize.rs:42-44` — per-row commit, no transaction

`persist_normalized_events` does `for event { store.persist_event(...).await? }` — one commit per
row. If row *n* fails, rows 0..n-1 are already durably committed; a partial event stream lands on a
run that downstream assumes is ordered and complete. `?` restores nothing (patterns-rust §
Unit-of-work).

- **Fix:** funnel all inserts into one Postgres tx, committed once; `Err` before commit aborts
  atomically.

#### `runtime/run.rs:303,307` — spawn rollback gap

`spawn_at_port` calls `start_container` then `attach_pty`. If attach fails after the container
started, the container is left **running** while `run.status` stays `Queued` — no `stop_container`
on that path. `spawn_persisted` releases the port on failure but not the container.

- **Fix:** treat spawn as a unit — on any post-start failure, stop the container and roll back
  status/port before returning `Err`.

#### `store/agents.rs:48-66` — non-atomic next-version allocation

`save_agent_definition` computes `COALESCE(MAX(version),0)+1` inside the INSERT CTE, under no lock.
Two concurrent saves of one name compute the same version; the second violates `UNIQUE(name,version)`
as an opaque `anyhow` error with no retry.

- **Fix:** `SELECT … FOR UPDATE`, a per-name advisory lock, or catch the unique violation and
  recompute once.

#### `store/mod.rs:4,302` — `pool()` leaks the driver; the "only sqlx-aware layer" invariant isn't held

The module doc claims this is the only sqlx-aware layer, but `pub fn pool()` is public and
services/runtime reach the concrete driver directly (`services/provider.rs:420`,
`services/agents.rs:325`, `runtime/session.rs:230`). The stated contract and the surface disagree.

- **Fix:** make `pool()` `pub(crate)`; add `Store` methods for external callers.

#### `harness/materialize/mod.rs:494` — panic on config data

`as_object_mut().expect("…remains an object")` assumes each key-path intermediate is a JSON object.
Keys come from runtime-loaded harness TOML; an extension collision can feed a scalar into a path,
panicking the run-critical materialize path instead of returning `Err`.

- **Fix:** return `MaterializeError` on a non-object intermediate.

#### `sandbox/credential_proxy.rs:106,230,325` — three unbounded/ungated paths

- `runs` map (keyed by run_id) grows forever — never pruned on teardown, so revoked runs keep their
  nonce accepted indefinitely.
- in-memory `audit` Vec never trimmed — grows for process lifetime.
- upstream `client.request().send()` has **no timeout**; a hung upstream stalls the single proxy
  thread and every run's model request behind it.

- **Fixes:** prune on run teardown; make audit a bounded ring; set connect/read timeouts + body cap.

#### `services/telemetry.rs:188` — unbounded event journal

`events: Arc<Mutex<Vec<Event>>>` appends forever, retaining each `raw: Value` (full ACP records incl.
tool outputs). `events_for` then does an O(n) scan per run. On a long-lived host this grows without
bound.

- **Fix:** bound/prune the journal (the bounded broadcast is the correct live consumer) or back it
  with a capped per-run store.

#### `services/ask.rs:36-37` — non-atomic two-stage command

`deliver_question(&request)?` commits, then `mark_waiting(...)?` can fail — an orphaned filed
question whose run was never blocked; a half-result.

- **Fix:** one atomic method on the inbox, or roll back `deliver_question` when `mark_waiting`
  errors.

#### IPC seam flattens typed errors to `String`

`apps/desktop/src-tauri/src/lib.rs:201,233,282,289,297,333` and `crates/locus-cli/src/main.rs`.

The core has strong typed enums (`ToolAdmissionError`, `RegistryLoadError`, harness selection), then
every Tauri handler and CLI dispatch returns `Result<_, String>` / `anyhow`. The frontend can only
string-match to recover — can't branch on not-found vs permission vs replay.

- **Fix:** a small serde error-enum `{ kind, message }` at each process boundary.

#### Advertised-but-stub services; CLI allowlist runs ahead

`services/mod.rs` declares `pub mod board/mail/memory/wiki` — each a single doc comment, zero code —
while `locus-cli/src/sock.rs:27-105` allowlists 70+ verbs including `memory.*`, `mail.*`,
`wiki.*`. An agent calling `memory recall` passes the gate then gets a deferred "unknown verb" at the
daemon, not a capability. The built surface and advertised surface disagree.

- **Fix:** keep verbs in the table only when a handler exists; mark the stub modules explicitly.

#### `locus-cli/src/sock.rs` — verb dispatch on raw `String`

`AgentSocketRequest.verb: String` + `route(run_id, verb: &str, ...)` routes on a raw string; a typo
is a silent capability-path mismatch, and callers can't branch on failure kind.

- **Fix:** a serde `Verb` enum validated at the socket edge.

### Suggestions

- **Duplicate `InProcessBus`** — `store/bus.rs:68-84` redefines a near-identical `InProcessBus<T>`
  after the bus was consolidated to the crate root; only its own tests reach it. Delete and use
  `crate::bus`.
- **`harness/materialize/strategy.rs:9-13` trait never dispatched polymorphically** — the `Strategy`
  trait exists only for a shape test; each impl is called directly. Keep the concrete structs, drop
  the trait.
- **`planning.rs:228-349` alias surface** — five `for_*` methods each forward to a differently-named
  twin (`for_spec_only` = `spec_only`, etc.), doubling the public API (patterns-rust §5). Pick one
  name each.
- **`planning.rs:42-44` `Requirement` pub fields** — blank id/body reachable without the
  `EditableSpec::new` guard. Private fields + `Requirement::new` that bails on empty.
- **`credential_proxy.rs:71,158` fixed port `43800` sits inside the allocator range `[43000,43999]`**
  — `PortAllocator.allocate()` can hand the same port to another container. Move it out of range or
  mark it reserved.
- **`ports.rs` TOCTOU** — `allocate` reserves a number but not the socket; the bind can be squatted.
  Have allocate hold the socket.
- **Rate-limit map** — `services` `calls: Mutex<HashMap<...>>` drops expired deque fronts but never
  removes empty/stale run keys.
- **`services/artifact.rs` in-memory shim** — `ArtifactStore` is a `BTreeMap` (M1 seam); an artifact's
  "durable" bodies vanish on restart. Fine for M1 if understood as non-durable.
- **`lib.rs` test hard-codes `reports.len()==11`, `losses==29`** — adding a harness breaks cardinality,
  not behavior. Assert structure, not counts.
- **`canary.rs:33` `AtomicU64` global temp-name counter** — benign, but module-level state dodging
  ownership.
- **`lib.rs:151-156` placeholder DTOs** — `author: "you"`, `created_at: String::new()`, fabricated
  `derived_text`. Fine seeded; must not linger.

### Informational

- `InProcessBus::publish` `.unwrap_or(0)` — a slow subscriber silently drops (broadcast overflow);
  fine fire-and-forget, not lossless.
- Store → services module cycle — store imports logic types; adapters depending inward is correct;
  the reverse `pool()` leak is the real problem.
- `audits.rs` self-built tokio runtime in `new` — pervasive but defensible.
- `telemetry.rs` closed `EventVerb` vocab + deterministic lint discovery — strong positives.
- `provider.rs` `ProviderBroker<K: OsKeychain>` — exemplar port-and-adapter; secret only resolved on
  egress, `redact()` scrubs errors.
- `image.rs` key omits prompt/config deliberately — asserted. Correct.
- `dispatch.rs` / `providers.rs` transactions (FOR UPDATE, commit-on-end, atomic catalog replace) —
  correct.

### Clean files (verified)

`docker.rs`, `egress.rs`, `image.rs`, `mounts.rs`, `ports.rs` (±TOCTOU), `projects.rs`,
`routing.rs`, `model_tiers.rs`, `adapter.rs`, `selection.rs`, `registry.rs`,
`materialize/tree.rs`, `extensions.rs`, `report.rs`, `agents.rs`, `project.rs`, `tools.rs`,
`workflow.rs` (minimal IPC-persistence `Value` — not a finding).

---

## TypeScript app

### Warnings

#### `screens/review/RunsView.tsx:35-36` — fabricated metrics presented as measured

`cache` = `Math.round(tokens * 0.84)`; `spend` = `tokens / 50_000` — invented magic-number
derivatives shown as real columns. This contradicts the app's own invariant ("a missing verb is
recorded as missing, never synthesized"; `orUnknown` convention used lines away).

- **Fix:** source from real fields or render `unknown`; don't fabricate.

#### `screens/plan/PlanView.tsx:53-61` — subscription leak

`onMount` opens a Tauri telemetry channel and never unsubscribes — no `onCleanup`. Leaving/re-entering
Plan re-subscribes; the live core stream accumulates and `setMessages` keeps firing after unmount.
Same seam in `AgentPane` / `ShellPane` (not mounted yet).

- **Fix:** give `streamFromCore` a teardown/cancel path and call it in `onCleanup`.

#### `nav/` — dual resolver + hand-maintained bidirectional mapping

Two parallel grammars (`locator.ts` `locus://` + `desktop-locator.ts`) bridged by
`shell/Shell.tsx:22-56` with two hand-maintained tables (`desktopViews` 30 entries, `desktopRoutes`
14) and no inverse assertion. The desktop route set derives from a **fixture**; the legacy set from
`nav/views.ts` — two independent "canonical" sources that can silently diverge. Marked transitional,
but nothing binds the round-trip.

- **Fix:** one resolver, or an invariant that the reverse mapping round-trips.

### Suggestions

- **`data/agent-defs.ts:88` non-null assertion on a fixture lookup** — `EXTENSION_COUNTS.find(...)!`
  throws if a generated fixture omits entries. Fix: `?? { native: 0, downgraded: 0 }`.
- **`AgentDefsView.tsx:29` `as` cast on unknown IPC data** — `frontmatter.memory as { scope?: string }`
  plants an unchecked claim. Fix: an `isMemoryScope` guard.
- **`GuardrailsView.tsx:75,83,92` cast-the-compiler** — nested `Show`/`fallback` casts instead of
  `Switch`/`Match` on `control.kind`.
- **`AgentPane.tsx:12` / `ShellPane.tsx:15` unhandled rejection before cleanup is registered** — a
  rejected invoke leaks the subscription and shows nothing. Fix: try/catch → `InlineError`; register
  cleanup before the await.
- **`ProjectRail.tsx:27` unguarded `JSON.parse(localStorage.getItem(...) ?? "{}")`** — corrupt stored
  JSON crashes the rail during render. Fix: safe parse.
- **`ArtifactsView.tsx:59-68` out-of-order fetch race** — no staleness guard; a slow earlier response
  clobbers a newer one on A→B→A. Fix: per-artifact token.

### Strengths

Consistently discriminated-union state, a pure CQS pane reducer, the boundary-uniform `locus://`
resolver, and **zero silent catches** — every `catch` sets a typed error state surfaced via
`InlineError`.

---

## Full-pass verdict

Consistent with the skill's idiom references throughout: DI-by-function-parameters (not a container),
typed errors at domain seams (`ToolAdmissionError`, `RegistryLoadError`), correct transactions in
dispatch/providers, byte-deterministic materializer, thin IO adapters, no harness-named-in-core.

The gaps cluster in three places:

1. **Resource unboundedness** — socket frame, telemetry journal, credential-proxy maps, audit vec,
   rate-limit map.
2. **Atomicity** — normalize persist, run spawn rollback, ask two-stage, agent-version allocation.
3. **Boundary leak** — `pool()`, IPC-to-String, the stub-service verb table, fabricated TS metrics.

Highest-leverage fixes (both order-of-now, before the proxy coexists with the allocator under load):
`sock.rs` frame cap (memory-exhaustion) and the credential-proxy upstream timeout (availability).

---

## Method note

One of the four parallel passes (the runtime pass) hallucinated a framing that the entire codebase
was "not compilable Rust but a design language" and produced a false positive (claimed
`container.rs` spawns a fire-and-forget thread with no join — the source has `.join()` at
`container.rs:126,220`). That pass's findings were **not** trusted wholesale; each load-bearing item
was re-verified against source by inspection. The findings retained from the runtime area
(`normalize.rs`, `run.rs`, `sock.rs`, verb-dispatch) were confirmed directly. Sandbox, store,
services, and TypeScript passes were individually verified on their headline claims.
