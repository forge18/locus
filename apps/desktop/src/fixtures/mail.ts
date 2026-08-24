import type { Message } from '../types/mail'

export type MailStatus = 'waiting' | 'open' | 'replied' | 'you' | 'drained'
export type MailVerb = 'send' | 'read' | 'reply' | 'wait'

export interface MailThreadFixture {
  id: string
  project: string
  status: MailStatus
  subject: string
  from: string
  to: string
  messageCount: number
  blocking: string | null
}

export interface MailParticipant {
  name: string
  runId: string
  state: MailStatus
}

export const MAIL_STATUSES: MailStatus[] = ['waiting', 'open', 'replied', 'you', 'drained']
export const MAIL_VERBS: MailVerb[] = ['send', 'read', 'reply', 'wait']

export const MAIL_WAIT_BANNER = 'builder@4 is in mail wait — 8m of a 15m timeout'
export const MAIL_WAIT_INVARIANT = 'State is waiting, not idle. The idle guardrail will not fire.'
export const MAIL_WAIT_LIVE_LINE = 'blocked here · returns empty at 15m and the run resumes'
export const MAIL_PARTICIPANT_NOTE = 'Different containers, one address space. Mail survives a harness swap mid-project.'
export const MAIL_HANDOFF_COPY = 'The moment ownership transfers it stops being mail and becomes a handoff, with a payload the successor reads instead of this thread'
export const MAIL_STORAGE_COPY = 'Agent-to-agent mail is stored, not ephemeral. When a run goes wrong the question is usually what one agent told another — and it was invisible until here.'

export const MAIL_THREADS: MailThreadFixture[] = [
  { id: 'thread-1', project: 'tapestry', status: 'waiting', subject: 'Notify payload contract', from: 'builder@4', to: 'reviewer@2', messageCount: 4, blocking: '8m · mail wait' },
  { id: 'thread-2', project: 'weaver', status: 'open', subject: 'Parser fixture question', from: 'builder@4', to: 'auditor@2', messageCount: 2, blocking: null },
  { id: 'thread-3', project: 'loom-db', status: 'replied', subject: 'Index verification result', from: 'auditor@2', to: 'builder@4', messageCount: 3, blocking: null },
  { id: 'thread-4', project: 'tapestry', status: 'you', subject: 'Review requested', from: 'reviewer@2', to: 'you', messageCount: 1, blocking: null },
  { id: 'thread-5', project: 'loom-db', status: 'drained', subject: 'Ownership transferred', from: 'builder@4', to: 'builder@5', messageCount: 5, blocking: null },
]

export const MAIL_PARTICIPANTS: MailParticipant[] = [
  { name: 'builder@4', runId: 'r-9f21', state: 'waiting' },
  { name: 'reviewer@2', runId: 'r-9f22', state: 'open' },
  { name: 'you', runId: 'human', state: 'you' },
]

export interface MailMessage extends Message {
  verb: MailVerb
}

export const MAIL_MESSAGES: MailMessage[] = [
  { id: 'message-1', threadId: 'thread-1', from: 'builder@4', to: ['reviewer@2'], body: 'The listener re-reads the row named by NOTIFY.', artifactIds: [], state: 'delivered', sentAt: '2026-08-20T14:20:00Z', verb: 'send' },
  { id: 'message-2', threadId: 'thread-1', from: 'reviewer@2', to: ['builder@4'], body: 'Please confirm the payload remains id-only.', artifactIds: ['a-1'], state: 'read', sentAt: '2026-08-20T14:23:00Z', verb: 'read' },
  { id: 'message-3', threadId: 'thread-1', from: 'builder@4', to: ['reviewer@2'], body: 'Waiting for the verify result before I continue.', artifactIds: [], state: 'delivered', sentAt: '2026-08-20T14:24:00Z', verb: 'reply' },
  { id: 'message-4', threadId: 'thread-1', from: 'builder@4', to: ['reviewer@2'], body: 'No new mail yet; waiting.', artifactIds: [], state: 'delivered', sentAt: '2026-08-20T14:26:00Z', verb: 'wait' },
]

export const SELECTED_MAIL_THREAD_ID = 'thread-1'
export const MAIL_TABS = ['All', 'Waiting', 'To you'] as const
