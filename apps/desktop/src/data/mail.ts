import { dataProvider } from "./provider";
import type {
  MailMessage,
  MailParticipant,
  MailThreadFixture,
} from "./demo/fixtures/mail";

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
} from "./demo/fixtures/mail";
export type {
  MailMessage,
  MailParticipant,
  MailStatus,
  MailThreadFixture,
  MailVerb,
} from "./demo/fixtures/mail";

/** Becomes: invoke('mail_threads', { scope }) */
export function useMailThreads(): MailThreadFixture[] {
  return dataProvider().read?.<MailThreadFixture[]>("mail_threads") ?? [];
}

/** Becomes: invoke('mail_messages', { threadId }) */
export function useMailMessages(threadId: string): MailMessage[] {
  return (
    dataProvider().read?.<MailMessage[]>("mail_messages", { threadId }) ?? []
  );
}

/** Becomes: invoke('mail_participants', { threadId }) */
export function useMailParticipants(): MailParticipant[] {
  return dataProvider().read?.<MailParticipant[]>("mail_participants") ?? [];
}
