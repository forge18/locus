import type { Envelope } from "./envelope";
import { dataProvider } from "./provider";

export type BotContainerState = "cold" | "running" | "warm";
export type RoutineExecutionStatus =
  | "running"
  | "completed"
  | "failed"
  | "skipped";
export type RoutineAttribution = "routine-fired" | "test-run";

export interface Bot {
  id: string;
  projectId: string;
  name: string;
  agentDefId: string;
  homeSessionId: string;
  activeRunId?: string | null;
  harness?: string | null;
  branch: string;
  containerId: string | null;
  containerState: BotContainerState;
  warmUntil: string | null;
  lastActivityAt: string | null;
  totalCostMicros: number | null;
}

export interface BotRoutine {
  id: string;
  botId: string;
  prompt: string;
  cronExpression: string;
  enabled: boolean;
  skippedCount: number;
  scheduleId: string | null;
}

export interface BotRoutineExecution {
  id: string;
  botId: string;
  scheduledFor: number;
  status: RoutineExecutionStatus;
  result: { passed: boolean; summary: string } | null;
  attribution: RoutineAttribution;
  testRun: boolean;
}

export function botsList(projectId: string): Promise<Envelope<Bot[]>> {
  return dataProvider().query<Bot>("bots_list", { projectId });
}

export function createBot(
  projectId: string,
  markdown: string,
): Promise<Envelope<Bot>> {
  return dataProvider().queryOne<Bot>("bot_create", { projectId, markdown });
}

export function botRoutines(
  projectId: string,
  botId: string,
): Promise<Envelope<BotRoutine[]>> {
  return dataProvider().query<BotRoutine>("bot_routines", {
    projectId,
    botId,
  });
}

export function botRoutineExecutions(
  projectId: string,
  botId: string,
): Promise<Envelope<BotRoutineExecution[]>> {
  return dataProvider().query<BotRoutineExecution>("bot_routine_executions", {
    projectId,
    botId,
  });
}

export function setBotRoutineEnabled(
  projectId: string,
  routineId: string,
  enabled: boolean,
): Promise<Envelope<void>> {
  return dataProvider().queryOne<void>("bot_routine_set_enabled", {
    projectId,
    routineId,
    enabled,
  });
}

export function updateBotRoutine(
  projectId: string,
  routineId: string,
  prompt: string,
  cronExpression: string,
): Promise<Envelope<BotRoutine>> {
  return dataProvider().queryOne<BotRoutine>("bot_routine_update", {
    projectId,
    routineId,
    prompt,
    cronExpression,
  });
}

export function deleteBotRoutine(
  projectId: string,
  routineId: string,
): Promise<Envelope<void>> {
  return dataProvider().queryOne<void>("bot_routine_delete", {
    projectId,
    routineId,
  });
}

export function testBotRoutine(
  projectId: string,
  routineId: string,
): Promise<Envelope<BotRoutineExecution>> {
  return dataProvider().queryOne<BotRoutineExecution>("bot_routine_test", {
    projectId,
    routineId,
  });
}

export function sendBotPrompt(
  projectId: string,
  botId: string,
  prompt: string,
): Promise<Envelope<void>> {
  return dataProvider().queryOne<void>("bot_prompt", {
    projectId,
    botId,
    prompt,
  });
}
