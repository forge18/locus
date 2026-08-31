import { dataProvider } from "./provider";
import type { Envelope } from "./envelope";
export {
  DISPATCH_PROJECTS,
  NEVER_AUTORUN_EXCLUSIONS,
  STOP_ALL_AGENT_COUNT,
  STOP_ALL_RESTORE_MINUTES,
  VERIFY_VOCABULARY,
  autorunMasterState,
} from "./demo/fixtures/dispatch";
export type { AutorunState, PermissionPosture } from "./demo/fixtures/dispatch";

/** Wire type: one dispatch schedule from `workflows.schedules`. */
export interface DispatchSchedule {
  id: string;
  projectId: string;
  project: string;
  name: string;
  cron: string;
  enabled: boolean;
}

/** Wire type: one execution of a dispatch schedule. */
export interface DispatchScheduleExecution {
  id: string;
  scheduleName: string;
  project: string;
  status: string;
  scheduledFor: string | null;
  startedAt: string | null;
  endedAt: string | null;
}

/** Wire type: one project's tri-state autorun posture. */
export interface AutorunStateRow {
  projectId: string;
  project: string;
  state: "on" | "off" | "suspended";
}

/** Every project's autorun posture, for the switchboard. A project with no
 * row defaults to off. */
export function fetchAutorunStates(): Promise<Envelope<AutorunStateRow[]>> {
  return dataProvider().query<AutorunStateRow>("autorun_states");
}

/** Set one project's tri-state autorun posture. */
export function setAutorunState(
  projectId: string,
  state: "on" | "off" | "suspended",
): Promise<Envelope<void>> {
  return dataProvider().queryOne<void>("set_project_autorun_state", {
    projectId,
    state,
  });
}

export function fetchDispatchSchedules(): Promise<
  Envelope<DispatchSchedule[]>
> {
  return dataProvider().query<DispatchSchedule>("dispatch_schedules");
}

export function fetchScheduleExecutions(
  projectId?: string,
  limit = 50,
): Promise<Envelope<DispatchScheduleExecution[]>> {
  return dataProvider().query<DispatchScheduleExecution>(
    "dispatch_schedule_executions",
    { projectId, limit },
  );
}

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
