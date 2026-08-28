import { invoke } from "@tauri-apps/api/core";

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

export function botsList(projectId: string) {
  return invoke<Bot[]>("bots_list", { projectId });
}

export function createBot(projectId: string, markdown: string) {
  return invoke<Bot>("bot_create", { projectId, markdown });
}

export function botRoutines(botId: string) {
  return invoke<BotRoutine[]>("bot_routines", { botId });
}

export function botRoutineExecutions(botId: string) {
  return invoke<BotRoutineExecution[]>("bot_routine_executions", { botId });
}

export function setBotRoutineEnabled(routineId: string, enabled: boolean) {
  return invoke<void>("bot_routine_set_enabled", { routineId, enabled });
}

export function updateBotRoutine(
  routineId: string,
  prompt: string,
  cronExpression: string,
) {
  return invoke<BotRoutine>("bot_routine_update", {
    routineId,
    prompt,
    cronExpression,
  });
}

export function deleteBotRoutine(routineId: string) {
  return invoke<void>("bot_routine_delete", { routineId });
}
