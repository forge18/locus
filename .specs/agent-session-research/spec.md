# agent-session-research

**Milestone** M1.5 · **Depends on** `artifacts`, `planning-module`, `memory`

## Purpose

Make `finding` artifacts a per-session research feed: inherited from planning, visibly attributed by
provenance, and promoted to memory only through an explicit session-close review.

## Acceptance

1. The feed is scoped to one session and labels seed, this-run, and session-close provenance.
2. Child task sessions inherit planning findings without promoting them.
3. Only a reviewed finding may enter long-term memory at session close.
