import type { Envelope } from "./envelope";
import { dataProvider } from "./provider";
import type {
 AgentDefSummary,
 FrontmatterLine,
} from "./demo/fixtures/agent-defs";

export {
 DIFF_LABEL,
 MATERIALIZE_TARGET,
 NEXT_VERSION,
 PROVENANCE,
 SAVE_LABEL,
 SIDEBAR_NOTE,
} from "./demo/fixtures/agent-defs";
export type {
 AgentDefSummary,
 FrontmatterLine,
} from "./demo/fixtures/agent-defs";

export interface CoreAgentDefinition {
 name: string;
 version: number;
 frontmatter: Record<string, unknown>;
 body: string;
 warnings: string[];
}

/** The Workshop IPC boundary; fixtures remain only as the non-Tauri test fallback. */
export function fetchAgentDefsFromCore(): Promise<Envelope<AgentDefSummary[]>> {
 return dataProvider().query<AgentDefSummary>("agent_defs_list");
}

export function fetchAgentDefFromCore(
 name: string,
): Promise<Envelope<CoreAgentDefinition>> {
 return dataProvider().queryOne<CoreAgentDefinition>("agent_def", { name });
}

/** Becomes: fetchAgentDefsFromCore() after the Tauri runtime connects. */
export function useAgentDefs(): AgentDefSummary[] {
 return dataProvider().read?.<AgentDefSummary[]>("agent_defs_list") ?? [];
}

/** Becomes: pane state, once the pane manager owns it. */
export function useDefaultAgentDef(): string {
 return dataProvider().read?.<string>("agent_default_id") ?? "";
}

/** Becomes: fetchAgentDefFromCore(name) after the Tauri runtime connects. */
export function useFrontmatter(): FrontmatterLine[] {
 return dataProvider().read?.<FrontmatterLine[]>("agent_frontmatter") ?? [];
}

/** Becomes: fetchAgentDefFromCore(name) after the Tauri runtime connects. */
export function useProse(): string[] {
 return dataProvider().read?.<string[]>("agent_prose") ?? [];
}

/**
 * Becomes: invoke("materialization_report", { extension })
 *
 * How many harnesses this definition reaches, and how many of them take it
 * weaker than native. Computed from the registry, never written down.
 */
export function useAgentMaterialization(): {
 harnesses: number;
 downgraded: number;
} {
 return (
  dataProvider().read?.<{ harnesses: number; downgraded: number }>(
   "agent_materialization",
  ) ?? { harnesses: 0, downgraded: 0 }
 );
}
