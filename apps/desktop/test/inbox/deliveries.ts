import type { InboxDelivery } from "../../src/data/inbox";

/** Store-shaped pending deliveries — the rows `inbox_list` returns. */
export const PENDING: InboxDelivery[] = [
  {
    id: "d-0000",
    threadId: "t-0000",
    subject: "Plan gate: human sign-off",
    body: "The plan is ready for review. Approve to start the build.",
    senderKind: "agent",
    project: "tapestry",
    createdAt: new Date(Date.now() - 26 * 60000).toISOString(),
  },
  {
    id: "d-0001",
    threadId: "t-0001",
    subject: "Question: which schema?",
    body: "Two schemas match the request. Pick one before I migrate.",
    senderKind: "agent",
    project: "tapestry",
    createdAt: new Date(Date.now() - 141 * 60000).toISOString(),
  },
  {
    id: "d-0002",
    threadId: "t-0002",
    subject: "Guardrail tripped in runs",
    body: "A finished run examined the guardrail and needs a decision.",
    senderKind: "agent",
    project: "loom-db",
    createdAt: new Date(Date.now() - 5 * 60000).toISOString(),
  },
];

export const PENDING_TAPESTRY = PENDING.filter(
  (item) => item.project === "tapestry",
);
