import { invoke, isTauri } from "@tauri-apps/api/core";
import {
 AGENT_DEFS,
 FRONTMATTER,
 PROSE,
 SELECTED_DEF,
} from "../fixtures/agent-defs";
import type { AgentDefSummary, FrontmatterLine } from "../fixtures/agent-defs";
import {
 EXTENSION_COUNTS,
 HARNESS_COUNT,
} from "../fixtures/generated/harnesses";

export {
 DIFF_LABEL,
 MATERIALIZE_TARGET,
 NEXT_VERSION,
 PROVENANCE,
 SAVE_LABEL,
 SIDEBAR_NOTE,
} from "../fixtures/agent-defs";
export type { AgentDefSummary, FrontmatterLine } from "../fixtures/agent-defs";

export interface CoreAgentDefinition {
 name: string;
 version: number;
 frontmatter: Record<string, unknown>;
 body: string;
 warnings: string[];
}

/** Rebuild a CoreAgentDefinition from the fixture so the no-host path renders the same content. */
function fixtureDefinition(name: string): CoreAgentDefinition {
 const def = AGENT_DEFS.find((d) => d.name === name) ?? AGENT_DEFS[0];
 const frontmatter = Object.fromEntries(
  FRONTMATTER.map((line) => [line.key, line.value]),
 );
 return {
  name: def.name,
  version: def.version,
  frontmatter,
  body: PROSE.join("\n\n"),
  warnings: [],
 };
}

/** The Workshop IPC boundary; fixtures remain only as the non-Tauri test fallback. */
export async function fetchAgentDefsFromCore(): Promise<AgentDefSummary[]> {
 // No host (browser preview, tests) → fixture, not an error.
 if (!isTauri()) return AGENT_DEFS;
 return invoke<AgentDefSummary[]>("agent_defs_list");
}

export async function fetchAgentDefFromCore(
 name: string,
): Promise<CoreAgentDefinition> {
 // No host (browser preview, tests) → fixture, not an error.
 if (!isTauri()) return fixtureDefinition(name);
 return invoke<CoreAgentDefinition>("agent_def", { name });
}

/** Becomes: fetchAgentDefsFromCore() after the Tauri runtime connects. */
export function useAgentDefs(): AgentDefSummary[] {
 return AGENT_DEFS;
}

/** Becomes: pane state, once the pane manager owns it. */
export function useDefaultAgentDef(): string {
 return SELECTED_DEF;
}

/** Becomes: fetchAgentDefFromCore(name) after the Tauri runtime connects. */
export function useFrontmatter(): FrontmatterLine[] {
 return FRONTMATTER;
}

/** Becomes: fetchAgentDefFromCore(name) after the Tauri runtime connects. */
export function useProse(): string[] {
 return PROSE;
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
 const agents = EXTENSION_COUNTS.find((c) => c.type === "agents") ?? {
  native: 0,
  downgraded: 0,
 };
 return { harnesses: HARNESS_COUNT, downgraded: agents.downgraded };
}
