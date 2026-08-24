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
export type { AutorunState } from "../fixtures/dispatch";

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
