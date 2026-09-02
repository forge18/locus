import type { Bot, BotRoutine } from "../../bots";

export const BOTS: Bot[] = [
  {
    id: "keeper",
    projectId: "tapestry",
    name: "Keeper",
    agentDefId: "agent-keeper",
    homeSessionId: "bot-keeper-session",
    activeRunId: "bot-keeper-run",
    branch: "bots/keeper",
    containerId: "locus-agent-bot-keeper",
    containerState: "running",
    warmUntil: null,
    lastActivityAt: "now",
    totalCostMicros: 420000,
  },
  {
    id: "night-watch",
    projectId: "tapestry",
    name: "Night Watch",
    agentDefId: "agent-night-watch",
    homeSessionId: "bot-night-watch-session",
    activeRunId: null,
    branch: "bots/night-watch",
    containerId: null,
    containerState: "cold",
    warmUntil: null,
    lastActivityAt: "18m ago",
    totalCostMicros: null,
  },
];

export const ROUTINES: BotRoutine[] = [
  {
    id: "routine-health",
    botId: "keeper",
    prompt: "Check the repository health and report only actionable drift.",
    cronExpression: "0 9 * * 1-5",
    enabled: true,
    skippedCount: 1,
    scheduleId: null,
  },
];
