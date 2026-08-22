# run-supervisor

**Milestone** M1 · **Depends on** `sandbox`, `materializers`, `agent-definitions`, `telemetry`

## Purpose

Spawn, stream, normalize, persist, cancel — and hold the session/run/turn model that everything above
depends on. PLAN.md's split exists for one reason: **the session is what survives the reset.** The
Ralph-loop pattern needs something that persists across context resets and something that does not; the
run is what resets, the session is what accumulates.

Also owns the property that makes the whole thing trustworthy across restarts: **every start
reconciles.**

## Governed by

- PLAN.md §What a session is — the session / run / turn table
- PLAN.md §Process topology — `locusd` outlives the window; every start reconciles
- PLAN.md §Handoffs — the trigger points a session's ownership changes at

## Contract

```
Project
└── Session          a durable, named thread of work with ONE agent
    ├── Run          one container lifetime = one ACP session
    │    └── Turn    one prompt → one response
    └── Run          (after a loop reset: new container, same session)
```

| | Session | Run |
| --- | --- | --- |
| Bounded by | you closing it | the container exiting |
| Holds | agent@version, its branch, the board task, core-memory base, pane state | events, usage, exit status, artifacts, **the resolved model id** |
| Resumable | yes — by starting another run | no |
| Cost | the sum of its runs | measured directly |

**A Locus session is not the harness's session.** The harness's own session maps to a *run*, and
**resume belongs to Locus**: the next run is primed from the session's own events. Where a harness has
a native session id the core stores it on the run and hands it back — an optimization, not the
mechanism.

**A run you drive yourself is not a session.** Same pane type, no agent, no events, no cost.

**`locusd` outlives the window.** Closing the app detaches the UI and nothing else. Runs keep streaming
into Postgres and reopening re-attaches to state that never stopped.

**Every start reconciles.** On boot, runs marked `running` are compared against Docker: container alive
→ re-attach its stream; container gone → close the run as `aborted`, emit the event, and put it in the
inbox. Without this a crash leaves rows claiming to be running forever, and the dashboard slowly fills
with work that ended weeks ago.

**Pause means the loop stops being fed, not that a process is frozen.** The supervisor lets the current
turn finish, holds before the next iteration, and notifies; the container stays up so its state is
inspectable. `SIGSTOP` mid-request would leave sockets half-written and a model call in flight.

## Acceptance

1. A session survives its run ending, and a second run in the same session inherits branch, task and
   memory base.
2. The resolved model id is on the run row, not the tier.
3. Killing `locusd` mid-run and restarting re-attaches to the live container and resumes streaming.
4. Killing the container instead closes the run as `aborted`, emits the event, and files an inbox item.
5. Closing the app window leaves the run streaming into Postgres.
6. A human-driven run produces no session row, no events, and no cost attribution.
7. Resume primes a new run from the session's events, and works on a harness with no native session id.
8. Pause lets the current turn finish and leaves the container up.
9. Cancel stops the run and records the reason.

## Open

- Whether a session can be reassigned to a different agent without a handoff. PLAN.md says a session
  belongs to exactly one agent and that handoff opens a *new* session — so the answer is probably no,
  but it is not stated as an invariant.
