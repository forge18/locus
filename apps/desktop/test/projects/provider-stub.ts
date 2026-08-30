import {
  configureDataProvider,
  type DataProvider,
} from "../../src/data/provider";
import { ready, readyOne, type Envelope } from "../../src/data/envelope";

/**
 * A provider stub for the Setup tracer-bullet tests. Data is store-shaped (the
 * same rows the locus-tauri `setup_live_data` tests seed) so the screen is
 * exercised exactly as a live Tauri window would drive it — through the
 * provider seam, never through fixtures.
 */

export const TAPESTRY = "00000000-0000-0000-0000-000000000301";
export const LOOM = "00000000-0000-0000-0000-000000000302";

export const SEED_PROJECTS = [
  { id: LOOM, name: "loom-db" },
  { id: TAPESTRY, name: "tapestry" },
];

export const SEED_REPOS = [
  {
    id: "00000000-0000-0000-0000-000000000311",
    projectId: TAPESTRY,
    name: "core",
    workingCopyPath: "/checkouts/tapestry-core",
  },
  {
    id: "00000000-0000-0000-0000-000000000312",
    projectId: TAPESTRY,
    name: "desktop",
    workingCopyPath: "/checkouts/tapestry-desktop",
  },
  {
    id: "00000000-0000-0000-0000-000000000321",
    projectId: LOOM,
    name: "loom",
    workingCopyPath: "/checkouts/loom",
  },
];

export const SEED_SETUP = {
  harnessAllowList: ["claude", "codex"],
  baseContext: "# Working in tapestry\n\nYour branch is never main.",
  baseContextTokenBudget: 1500,
};

export interface ProjectsStub {
  projects?: { id: string; name: string }[];
  repos?: {
    id: string;
    projectId: string;
    name: string;
    workingCopyPath: string;
  }[];
  setup?: {
    harnessAllowList: string[];
    baseContext: string | null;
    baseContextTokenBudget: number | null;
  } | null;
  /** Every command fails, simulating a dead IPC boundary. A list of command
   * names fails only those — for testing one panel's error state. */
  fail?: boolean | string[];
  /** Queries never settle: pins the loading state. */
  hang?: boolean;
}

export function configureProjectsStub(stub: ProjectsStub = {}): void {
  // Keys left absent default to the store-shaped seed; an explicit empty array
  // is a deliberate empty-state test. Projects are sorted exactly as the host's
  // `ORDER BY name`, so the default selection matches a live window.
  const seededProjects =
    "projects" in stub ? (stub.projects ?? []) : SEED_PROJECTS;
  const projects = [...seededProjects].sort((a, b) =>
    a.name.localeCompare(b.name),
  );
  const repos = "repos" in stub ? (stub.repos ?? []) : SEED_REPOS;
  const setup = "setup" in stub ? (stub.setup ?? null) : SEED_SETUP;
  const failing = (command: string) =>
    Array.isArray(stub.fail) ? stub.fail.includes(command) : stub.fail === true;
  const provider: DataProvider = {
    kind: "demo",
    async query<T>(
      command: string,
      args?: Record<string, unknown>,
    ): Promise<Envelope<T[]>> {
      if (stub.hang) return new Promise(() => undefined);
      if (failing(command)) {
        return {
          status: "failed",
          error: { command, message: `IPC failure for ${command}` },
        };
      }
      if (command === "projects_list") {
        return ready(projects as T[]);
      }
      if (command === "repos_list") {
        const projectId = args?.projectId;
        const rows = repos.filter((repo) => repo.projectId === projectId);
        return ready(rows as T[]);
      }
      return {
        status: "failed",
        error: { command, message: `unexpected command ${command}` },
      };
    },
    async queryOne<T>(command: string): Promise<Envelope<T>> {
      if (stub.hang) return new Promise(() => undefined);
      if (failing(command)) {
        return {
          status: "failed",
          error: { command, message: `IPC failure for ${command}` },
        };
      }
      if (command === "project_setup") {
        return readyOne(setup) as Envelope<T>;
      }
      return {
        status: "failed",
        error: { command, message: `unexpected command ${command}` },
      };
    },
  };
  configureDataProvider(provider);
}
