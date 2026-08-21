# handoffs

**Milestone** M3 · **Depends on** `mail`, `run-supervisor`, `guardrails`

## Purpose

The guardrails already kill and reassign after three stuck iterations, and a session already belongs to
exactly one agent. Put together, those mean work changes hands regularly and currently arrives with
**nothing** — the successor inherits a branch and a task and rediscovers everything else.

A handoff is that missing payload. This is not a new mechanism; it is the payload for one that already
exists.

## Governed by

- PLAN.md §Handoffs — the payload, the four triggers, why it is neither mail nor invoke
- PLAN.md §Workflow guardrails — kill and reassign at three stuck iterations

## Contract

```
locus handoff <agent> --why …
```

Ends the current session and opens a new one **on the same task and the same branch**, linked by
`handed_off_from`, carrying one structured artifact:

```
handoff
  goal              what this work is for, restated
  done[]            what is finished, each with the evidence
  remaining[]       what is not, in the order it should be taken
  attempted[]       what was tried and did not work — the expensive half
  decisions[]       choices already made, so they are not re-litigated
  open[]            questions the successor inherits
  branch · task · artifacts[]
```

**The successor reads the handoff, never the predecessor's transcript.** A transcript is long, mostly
irrelevant, and replaying it hands over the confusion along with the context.

**`attempted[]` is the part that pays for the mechanism.** Without it the next agent's first act is to
retry what just failed.

**Four triggers, one mechanism:**

| Trigger | Who decides |
| --- | --- |
| Stuck — three iterations with no progress | the guardrail |
| Context exhausted | the run supervisor |
| The work needs a different role | the workflow graph |
| You reassign it | you |

**A handoff is not mail and not `locus agent invoke`.** Mail is between agents that both keep working;
invoke is a nested run that returns to its caller. **A handoff transfers ownership and does not come
back.**

## Acceptance

1. A handoff ends the predecessor's session and opens the successor's on the same task and branch.
2. `handed_off_from` links them, and the chain is traversable in both directions.
3. The successor's context contains the handoff payload and **not** the predecessor's transcript —
   asserted by checking what was actually injected.
4. `attempted[]` is required and non-empty when the trigger was stuck — a handoff after three failures
   with nothing attempted is a bug.
5. All four triggers produce the same artifact shape.
6. The guardrail's kill-and-reassign produces a handoff rather than dropping the work.
7. A handoff does not return to its predecessor — no path exists for it to.
8. The predecessor's artifacts are referenced, not copied.

## Open

- Whether a handoff can cross projects. PLAN.md scopes memory as never cross-project and a session to a
  project, which implies no — but it is not stated for handoffs.
