# guardrails — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Guardrail config with the seven defaults, overridable per workflow | — | `cargo test -p locus-core guard::defaults` |
| 2 | Record the effective value on the run | 1 | `cargo test -p locus-core guard::effective_recorded` |
| 3 | `max_iterations` counter and stop | 1 | `cargo test -p locus-core guard::max_iterations` |
| 4 | Record which guardrail tripped | 3 | `cargo test -p locus-core guard::trip_is_attributed` |
| 5 | Forced reflection injected before a retry | 3 | `cargo test -p locus-core guard::reflection_before_retry` |
| 6 | Reflection is visible in the event stream | 5 | `cargo test -p locus-core guard::reflection_observable` |
| 7 | Stuck detection over three iterations with no progress | 3 | `cargo test -p locus-core guard::stuck_detection` |
| 8 | Kill-and-reassign producing a handoff | 7 | `cargo test -p locus-core guard::reassign_hands_off` |
| 9 | `waiting` state with a reason on the run | — | `cargo test -p locus-core guard::waiting_state` |
| 10 | Caller 1: `locus ask` sets waiting | 9 | `cargo test -p locus-core guard::waiting_from_ask` |
| 11 | Caller 2: `mail wait` sets waiting | 9 | `cargo test -p locus-core guard::waiting_from_mail` |
| 12 | Caller 3: a debug breakpoint sets waiting | 9 | `cargo test -p locus-core guard::waiting_from_debug` |
| 13 | Caller 4: a `Gate` sets waiting | 9 | `cargo test -p locus-core guard::waiting_from_gate` |
| 14 | Idle detection at 60s of no events while not waiting | 9 | `cargo test -p locus-core guard::idle_at_60s` |
| 15 | Assert none of the four waiting callers trips idle | 10,11,12,13,14 | `cargo test -p locus-core guard::waiting_never_idle` |
| 16 | Idle icon on the tile | 14 | `pnpm -C apps/desktop test -- strip/idle-icon` |
| 17 | Toast once per idle stretch, never repeatedly | 14 | `cargo test -p locus-core guard::idle_toast_once` |
| 18 | Usage recorded on every run regardless of budget | — | `cargo test -p locus-core guard::usage_always_recorded` |
| 19 | Optional wall-clock ceiling | 1 | `cargo test -p locus-core guard::wall_clock` |
| 20 | Optional token budget with auto-pause at 85% | 18 | `cargo test -p locus-core guard::budget_pauses_at_85` |
| 21 | Auto-pause notifies rather than draining silently | 20 | `cargo test -p locus-core guard::pause_notifies` |
| 22 | Pause finishes the current turn and keeps the container up | 20 | `cargo test -p locus-core guard::pause_is_not_sigstop` |
| 23 | Holding is recorded as an event | 22 | `cargo test -p locus-core guard::hold_is_an_event` |
