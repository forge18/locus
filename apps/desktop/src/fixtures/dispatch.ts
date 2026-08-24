// schema: core.settings + workflows.schedules + workflows.executions + agents.runs
// replaced by: invoke("dispatch_snapshot")

export type AutorunState = "on" | "off" | "suspended" | "archived";
export type PermissionPosture = "bypass" | "gated";
export const AUTORUN_STATES = Object.freeze([
  "on",
  "off",
  "suspended",
  "archived",
] as const);
export const VERIFY_VOCABULARY = Object.freeze([
  "running",
  "passed",
  "failed",
  "failed ×n",
  "waiting: gate",
  "n/a",
  "aborted",
] as const);
export const NEVER_AUTORUN_EXCLUSIONS = Object.freeze([
  {
    id: "migrations",
    label: "Anything touching migrations/**",
    reason: "A migration is append-only and irreversible in practice.",
  },
  {
    id: "gate",
    label: "Any workflow containing a Gate node",
    reason: "The gate is the point. Skipping it would be deleting it.",
  },
  {
    id: "change-ceiling",
    label: "Anything over the change ceiling",
    reason:
      "Past a reviewer capacity, review degrades from semantic to syntactic.",
  },
  {
    id: "verify-floor",
    label: "A project under 60% verify pass",
    reason: "Trust is measured, not assumed. It resumes on its own.",
  },
  {
    id: "first-plan-task",
    label: "The first task of any plan",
    reason: "You see what a plan produces once before it produces unattended.",
  },
] as const);

export interface DispatchProject {
  id: string;
  name: string;
  repos: string;
  state: AutorunState;
  detail: string;
  activity: string;
}

/** Autorun is project policy. Archived and suspended projects cannot be armed here. */
export const DISPATCH_PROJECTS: readonly DispatchProject[] = Object.freeze([
  {
    id: "tapestry",
    name: "tapestry",
    repos: "2 repos · core, desktop",
    state: "on",
    detail:
      "Verify pass 91% over 7 days. Three agents are working now; you started none of them.",
    activity: "3 running",
  },
  {
    id: "loom-db",
    name: "loom-db",
    repos: "1 repo · loom",
    state: "on",
    detail:
      "Verify pass 84%. The migration task is held — it touches migrations/**, which never autoruns.",
    activity: "2 running",
  },
  {
    id: "weaver",
    name: "weaver",
    repos: "3 repos · keymap, term, ui",
    state: "suspended",
    detail:
      "Turned itself off. Verify pass fell to 44%, under the 60% floor — it returns when the number recovers.",
    activity: "suspended",
  },
  {
    id: "texere",
    name: "texere",
    repos: "1 repo · media",
    state: "off",
    detail:
      "Never been on. Its one run is blocked on a gate you have not answered.",
    activity: "1 waiting",
  },
  {
    id: "amq",
    name: "amq",
    repos: "1 repo · amq",
    state: "archived",
    detail:
      "Archived 6 days ago. Autorun cannot be turned on for an archived project.",
    activity: "archived",
  },
]);

export const SCHEDULE_RUN_MODES = Object.freeze(["project", "custom"] as const);
export const SCHEDULE_WHEN = Object.freeze([
  "run-once",
  "on-schedule",
  "hold",
] as const);
export const CRON_PRESETS = Object.freeze([
  { id: "hourly", label: "Hourly", cron: "0 * * * *" },
  { id: "nightly", label: "Nightly", cron: "0 2 * * *" },
  { id: "weekdays", label: "Weekdays 09:00", cron: "0 9 * * 1-5" },
  { id: "pick", label: "Once at a time I pick", cron: "" },
] as const);

export function cronForPreset(id: string) {
  return CRON_PRESETS.find((preset) => preset.id === id)?.cron ?? "";
}

export interface ScheduleFixture {
  id: string;
  name: string;
  cron: string;
  cadence: string;
  workflow: string;
  last: string;
  skipped: number;
  enabled: boolean;
  /** Per-job permission choice; bypass is the safe unattended default. */
  permissionPosture: PermissionPosture;
}

export const SCHEDULES: readonly ScheduleFixture[] = Object.freeze([
  {
    id: "wiki",
    name: "Nightly wiki reconcile",
    cron: "0 2 * * *",
    cadence: "every day at 02:00",
    workflow: "wf-04 · keeper → contradiction sweep",
    last: "skipped · 6h ago",
    skipped: 11,
    enabled: true,
    permissionPosture: "bypass",
  },
  {
    id: "audit",
    name: "Dependency audit",
    cron: "0 6 * * 1",
    cadence: "Mondays at 06:00",
    workflow: "wf-09 · auditor → spec gap report",
    last: "passed · 2d ago",
    skipped: 0,
    enabled: true,
    permissionPosture: "bypass",
  },
  {
    id: "bisect",
    name: "Flaky-test bisect",
    cron: "*/30 * * * *",
    cadence: "every 30 minutes",
    workflow: "wf-11 · builder → bisect-verify",
    last: "passed · 12m ago",
    skipped: 3,
    enabled: true,
    permissionPosture: "bypass",
  },
  {
    id: "decay",
    name: "Memory decay sweep",
    cron: "0 4 * * *",
    cadence: "every day at 04:00",
    workflow: "wf-02 · keeper → decay + promote",
    last: "passed · 8h ago",
    skipped: 1,
    enabled: true,
    permissionPosture: "bypass",
  },
  {
    id: "probe",
    name: "Harness capability probe",
    cron: "0 3 * * 0",
    cadence: "Sundays at 03:00",
    workflow: "wf-14 · probe all registered",
    last: "paused 5d ago",
    skipped: 0,
    enabled: false,
    permissionPosture: "bypass",
  },
  {
    id: "babysitter",
    name: "PR babysitter",
    cron: "*/15 * * * *",
    cadence: "every 15 minutes",
    workflow: "wf-07 · ci-babysitter → retry green",
    last: "passed · 3m ago",
    skipped: 2,
    enabled: true,
    permissionPosture: "bypass",
  },
]);

export interface ScheduleExecution {
  firedAt: string;
  schedule: string;
  result: "passed" | "failed" | "skipped";
  duration: string;
  evidence: string;
}

/** Executions record a verify result; an overlap is recorded as skipped, never queued. */
export const SCHEDULE_EXECUTIONS: readonly ScheduleExecution[] = Object.freeze([
  {
    firedAt: "2026-08-21 05:45",
    schedule: "PR babysitter",
    result: "passed",
    duration: "41s",
    evidence: "2 checks re-run green · r-a114",
  },
  {
    firedAt: "2026-08-21 05:30",
    schedule: "Flaky-test bisect",
    result: "passed",
    duration: "4m 02s",
    evidence: "keymap.test.ts isolated · r-a112",
  },
  {
    firedAt: "2026-08-21 05:00",
    schedule: "Flaky-test bisect",
    result: "skipped",
    duration: "—",
    evidence: "previous execution still running",
  },
  {
    firedAt: "2026-08-21 04:00",
    schedule: "Memory decay sweep",
    result: "failed",
    duration: "2m 14s",
    evidence: "verify: 2 assertions failed · r-a111",
  },
]);

export const STOP_ALL_AGENT_COUNT = 8;
export const STOP_ALL_RESTORE_MINUTES = 10;

export function autorunMasterState(projects: readonly DispatchProject[]) {
  const eligible = projects.filter(
    (project) => project.state !== "archived" && project.state !== "suspended",
  );
  const on = eligible.filter((project) => project.state === "on").length;
  return {
    eligible: eligible.length,
    on,
    label:
      eligible.length > 0 && on === eligible.length
        ? "All on"
        : on === 0
          ? "All off"
          : "Mixed",
  };
}
