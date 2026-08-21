// schema: agents.sessions + board.tasks + workflows.guardrail_trips
// replaced by: invoke("inbox_list") + emit("inbox_changed")

import type { ArtifactKind } from '../types/agents'

/** What kind of interruption this is. The inbox is the only interruption surface. */
export type InboxKind = 'gate' | 'ask' | 'guardrail'

export interface InboxItem {
  id: string
  kind: InboxKind
  title: string
  project: string
  agent: string
  role: string
  branch: string
  /** Where the work opens: the item resolves here, the work opens where it lives. */
  opensAt: string
  ageMinutes: number
  /** The artifact the reader is being asked to approve, if there is one. */
  artifactKind: ArtifactKind | null
  body: string[]
  callout: string | null
}

export const INBOX_ITEMS: InboxItem[] = [
  {
    id: 'in-1',
    kind: 'gate',
    title: 'Gate — approve plan before implementation',
    project: 'tapestry',
    agent: 'planner@3',
    role: 'planner',
    branch: 'agent/8f21-notify',
    opensAt: 'locus://tapestry/plan',
    ageMinutes: 26,
    artifactKind: 'plan',
    body: [
      'Add a notification channel to `crates/tapestry-core/src/notify.rs`, behind the existing `Sink` trait.',
      'Thread the channel through `Supervisor::spawn` so a run can report without holding the bus lock.',
      'Backfill `tests/notify/channel.rs` with the exhaustion case and the closed-receiver case.',
      'Leave the HTTP sink alone — it is out of scope and the open question below says why.',
    ],
    callout: 'Widening to the HTTP sink was raised and kept out. It stays as open[1].',
  },
  {
    id: 'in-2',
    kind: 'ask',
    title: 'locus ask — which migration path?',
    project: 'loom-db',
    agent: 'builder@4',
    role: 'builder',
    branch: 'agent/3c04-index',
    opensAt: 'locus://loom-db/develop',
    ageMinutes: 12,
    artifactKind: null,
    body: [
      'The index rebuild can run online with a partial index, or offline in one pass.',
      'Online is slower and leaves a window where reads see both indexes.',
    ],
    callout: null,
  },
  {
    id: 'in-3',
    kind: 'guardrail',
    title: 'Guardrail — kill & reassign, 3 stuck iterations',
    project: 'weaver',
    agent: 'builder@4',
    role: 'builder',
    branch: 'agent/5a71-parser',
    opensAt: 'locus://weaver/session/5a71/run/9c02',
    ageMinutes: 4,
    artifactKind: 'report',
    body: [
      'Three iterations ended with the same failing verify: `cargo test -p weaver parser::`.',
      'The guardrail has stopped the loop rather than spending a fourth.',
    ],
    callout: 'Reassigning opens a new session on the same task and branch, carrying a handoff.',
  },
]

export interface ResolvedItem {
  id: string
  kind: InboxKind
  title: string
  ageMinutes: number
}

export const RESOLVED_TODAY: ResolvedItem[] = [
  { id: 'rs-1', kind: 'gate', title: 'Gate — approve plan for texere ingest', ageMinutes: 94 },
  { id: 'rs-2', kind: 'ask', title: 'locus ask — keep the old column?', ageMinutes: 141 },
  { id: 'rs-3', kind: 'guardrail', title: 'Guardrail — budget warning, tapestry', ageMinutes: 200 },
]
