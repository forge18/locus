import { RUN_ROWS } from "../fixtures/runs";
import {
  DISPATCH_PROJECTS,
  NEVER_AUTORUN_EXCLUSIONS,
  SCHEDULE_EXECUTIONS,
  SCHEDULES,
  STOP_ALL_AGENT_COUNT,
  STOP_ALL_RESTORE_MINUTES,
  VERIFY_VOCABULARY,
  autorunMasterState,
} from "../fixtures/dispatch";

export {
  DISPATCH_PROJECTS,
  NEVER_AUTORUN_EXCLUSIONS,
  SCHEDULE_EXECUTIONS,
  SCHEDULES,
  STOP_ALL_AGENT_COUNT,
  STOP_ALL_RESTORE_MINUTES,
  VERIFY_VOCABULARY,
  autorunMasterState,
};
export type { AutorunState, PermissionPosture } from "../fixtures/dispatch";

export interface DispatchStopAllResult {
  snapshotId: string;
  stoppedRuns: number;
}

/** Ask the run supervisor to stop all dispatch work and preserve its restore snapshot. */
export async function stopAllDispatch(writeHandoffs = true) {
  const { invoke } = await import("@tauri-apps/api/core");
  return (invoke as <T>(command: string, args: unknown) => Promise<T>)(
    "dispatch_stop_all",
    { writeHandoffs },
  ) as Promise<DispatchStopAllResult>;
}

/** Becomes: invoke('dispatch_runs', { query }) */
export function useDispatchRuns() {
  return RUN_ROWS;
}

/** Becomes: invoke('dispatch_autorun', { projectId }) */
export function useDispatchProjects() {
  return DISPATCH_PROJECTS;
}

/** Becomes: invoke('dispatch_schedules', { projectId }) */
export function useDispatchSchedules() {
  return SCHEDULES;
}

/** Becomes: invoke('dispatch_schedule_executions', { projectId }) */
export function useDispatchScheduleExecutions() {
  return SCHEDULE_EXECUTIONS;
}
