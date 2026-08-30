import { configureProjectsStub } from "../projects/provider-stub";
import { PENDING } from "./deliveries";
import type { InboxDelivery } from "../../src/data/inbox";

/** A resolved-today row the host would return for a drained delivery. */
export const RESOLVED_TODAY: {
  id: string;
  subject: string;
  body: string;
  project: string;
  resolvedAt: string;
}[] = [
  {
    id: "d-9000",
    subject: "Resolved earlier today",
    body: "The decision is recorded on the thread.",
    project: "tapestry",
    resolvedAt: new Date(Date.now() - 3600000).toISOString(),
  },
];

/**
 * Configure the provider stub with the inbox rows the view tests need, plus the
 * standard projects list so the filter renders. Returns the seeded deliveries
 * so assertions can reference ids without re-importing.
 */
export function configureInboxStub(
  overrides: {
    inboxList?: InboxDelivery[];
    inboxResolvedToday?: typeof RESOLVED_TODAY;
  } = {},
): InboxDelivery[] {
  const inboxList = overrides.inboxList ?? PENDING;
  configureProjectsStub({
    inboxList,
    inboxResolvedToday: overrides.inboxResolvedToday ?? RESOLVED_TODAY,
    inboxThroughput: {
      pending: inboxList.length,
      resolvedToday: (overrides.inboxResolvedToday ?? RESOLVED_TODAY).length,
    },
  });
  return inboxList;
}
