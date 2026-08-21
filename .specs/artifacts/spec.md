# artifacts

**Milestone** M1 · **Depends on** `store`, `run-supervisor` · **Blocks** `media-artifacts`, `agent-prs`

## Purpose

What you review instead of tool calls. With N agents running, reading transcripts does not scale;
reading deliverables does. Artifacts also do a **second, less obvious job** — they are where context
goes to be forgotten, which is what keeps a 60KB test log out of a context window while leaving it
reachable.

M1 ships the text kinds and the comment machinery. Media kinds and derived representations arrive with
`media-artifacts` at M3.5.

## Governed by

- PLAN.md §Artifacts — the kinds, the review/reference split, the storage split
- PLAN.md §Token discipline #4 — summaries with handles, never bodies
- PLAN.md §M1 — text in Postgres and the blob tree from the first artifact

## Contract

**Kinds, split by whether a human is meant to see them:**

| | Kinds | In the inbox |
| --- | --- | --- |
| **Review** | `plan` · `diff` · `diagram` · `image` · `recording` · `walkthrough` | yes, when it needs you |
| **Reference** | `finding` · `payload` | **never** — storage with a handle |

**That split is load-bearing.** Without it the inbox fills with an agent's own scratch, and the one
surface built to protect your attention becomes the one that spends it.

**The walkthrough is the one that earns its place.** On session completion, a concise summary of what
changed with screenshots and recordings inline — the actual answer to "six agents ran overnight, now
what."

**Text is rows; media is a file the row points at.** Plans, diffs and walkthroughs live in Postgres.
Screenshots and recordings land under `/var/lib/locus/artifacts/<project>/<run>/` with the row carrying
the path, media type and `sha256`. **Backup covers both trees or neither** — that is the reason the
blob tree exists from the first artifact rather than from the first 40MB recording.

**Artifacts are commentable, and a comment steers.** Inline feedback on a plan or diff routes back into
the session that produced it and the agent responds — the run is still live while the task is
unfinished, which is when comments actually arrive. A comment left after a session's last run exited is
delivered by starting the next one.

**Same mechanism as the M7 PR flow.** A PR review comment and an artifact comment are the same thing
arriving from two places, so this is one implementation, not two.

**Compaction writes overflow as artifacts.** Anything over a threshold becomes a `payload` artifact and
leaves **a one-line summary and an id** in its place; `locus artifact get` fetches the body if it turns
out to matter. Same rule as memory, tool docs and images — the fourth surface it applies to, and the
one that catches everything the other three do not.

**Media has a retention policy; text does not.** Recordings and screenshots prune with their run after
30 days unless the run is linked to a PR or a Done task — the two cases where the evidence is the
point. Text is small enough to keep forever, and the trace depends on it surviving.

## Acceptance

1. `plan`, `diff` and `walkthrough` persist as rows; a blob-backed kind persists as a file plus a row
   carrying path, media type and `sha256`.
2. Reference kinds never appear in the inbox — asserted, not merely intended.
3. A comment on an artifact reaches the live session that produced it.
4. A comment left after the last run exited is delivered when the next run starts.
5. `locus artifact put` and `get` round-trip a plan unchanged.
6. A tool result over the threshold becomes a `payload` artifact and leaves a one-line summary and an id.
7. The summary left behind is materially smaller than the body — the test asserts a ratio, not just
   that it is shorter.
8. Backup includes the blob tree; restoring produces rows whose paths resolve.
9. A walkthrough generates from a finished session and inlines its artifacts.

## Open

- The compaction threshold. PLAN.md says "over a threshold" without naming one; it should be a setting
  with a defensible default rather than a constant chosen here.
