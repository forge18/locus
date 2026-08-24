import {
  MAIL_HANDOFF_COPY,
  MAIL_MESSAGES,
  MAIL_PARTICIPANT_NOTE,
  MAIL_PARTICIPANTS,
  MAIL_STATUSES,
  MAIL_STORAGE_COPY,
  MAIL_TABS,
  MAIL_THREADS,
  MAIL_VERBS,
  MAIL_WAIT_BANNER,
  MAIL_WAIT_INVARIANT,
  MAIL_WAIT_LIVE_LINE,
  SELECTED_MAIL_THREAD_ID,
} from '../fixtures/mail'

export {
  MAIL_HANDOFF_COPY,
  MAIL_MESSAGES,
  MAIL_PARTICIPANT_NOTE,
  MAIL_PARTICIPANTS,
  MAIL_STATUSES,
  MAIL_STORAGE_COPY,
  MAIL_TABS,
  MAIL_THREADS,
  MAIL_VERBS,
  MAIL_WAIT_BANNER,
  MAIL_WAIT_INVARIANT,
  MAIL_WAIT_LIVE_LINE,
  SELECTED_MAIL_THREAD_ID,
}
export type { MailMessage, MailParticipant, MailStatus, MailThreadFixture, MailVerb } from '../fixtures/mail'

/** Future seam: invoke('mail_threads', { scope }) */
export function useMailThreads() {
  return MAIL_THREADS
}

/** Future seam: invoke('mail_messages', { threadId }) */
export function useMailMessages(threadId: string) {
  return MAIL_MESSAGES.filter((message) => message.threadId === threadId)
}

/** Future seam: invoke('mail_participants', { threadId }) */
export function useMailParticipants() {
  return MAIL_PARTICIPANTS
}
