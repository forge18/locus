# calibration-loop

**Milestone** M5 · **Depends on** `workflow-engine`, `planning-module`, `memory`

## Purpose

**What makes the system improve rather than merely repeat.** A failure that only ever produces a retry
teaches nothing; a failure that changes a template is paid for once.

The arbiter already classifies every failed iteration into four classes. This turns those
classifications into proposed changes — and puts every one of them behind a human gate.

## Governed by

- PLAN.md §M5 — the calibration loop and its four proposal types
- PLAN.md §When `Verify` fails, classify before retrying — the arbiter that feeds this
- PLAN.md §Memory — the same gate as promotion, for the same reason

## Contract

A retro agent reads the arbiter's classifications since the last pass and proposes **exactly four kinds
of change**, each aimed at the class that produced it:

| Recurring class | Proposal |
| --- | --- |
| Bug | promote the failing check into the project's regression set |
| Spec gap | add a clause to the relevant specialization record |
| Noise | recalibrate or quarantine the check that keeps failing for nothing |
| Ambiguity | add a topic to the interview, or a rule to the reduction pass |

**Every proposal lands in the reflection review queue and none applies without you** — the same gate the
memory keeper's promotions pass through, for the same reason. This is where the ETH Zurich finding
applies: LLM-written context measures worse at higher cost, so an agent may propose what goes into
always-on context but may not put it there.

**Specialization records are wiki `concept` pages, not a new store.** A record is curated prose about
how this project does a domain, which is what the wiki already is. **No fourth knowledge tier.**

**Applied only above a confidence threshold.** Below it the synthesis pass runs without the record,
because **a wrong domain assumption injected into a contract is worse than no assumption** — it arrives
wearing the authority of accumulated experience.

**This is the loop the field keeps finding:** a recurring bug becomes a regression test, a recurring
spec gap becomes a record clause, a recurring ambiguity becomes a compiler rule.

## Acceptance

1. The retro agent reads only classifications since the last watermark.
2. Each of the four classes produces its own proposal type and no other.
3. Every proposal lands in the review queue; **none applies automatically** — asserted per type.
4. Accepting a bug proposal adds the failing check to the project's regression set.
5. Accepting a spec-gap proposal adds a clause to a wiki `concept` page, not to a new store.
6. Accepting a noise proposal recalibrates or quarantines the check.
7. Accepting an ambiguity proposal adds an interview topic or a reduction rule.
8. Rejecting a proposal records the rejection so it is not re-proposed identically.
9. A specialization record below the confidence threshold is **not** injected into synthesis.
10. A single recurring spec gap across several tasks produces one proposal, not one per task.

## Open

- The confidence threshold for applying a specialization record. PLAN.md gives the rule and the reason
  but not the number, and it is the value that decides between accumulated wisdom and a confident wrong
  assumption.
