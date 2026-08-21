import { AGENT_DEFS, FRONTMATTER, PROSE, SELECTED_DEF } from '../fixtures/agent-defs'
import type { AgentDefSummary, FrontmatterLine } from '../fixtures/agent-defs'
import { EXTENSION_COUNTS, HARNESS_COUNT } from '../fixtures/generated/harnesses'

export {
  DIFF_LABEL,
  MATERIALIZE_TARGET,
  NEXT_VERSION,
  PROVENANCE,
  SAVE_LABEL,
  SIDEBAR_NOTE,
} from '../fixtures/agent-defs'
export type { AgentDefSummary, FrontmatterLine } from '../fixtures/agent-defs'

/** Becomes: invoke("agent_defs_list") */
export function useAgentDefs(): AgentDefSummary[] {
  return AGENT_DEFS
}

/** Becomes: pane state, once the pane manager owns it. */
export function useDefaultAgentDef(): string {
  return SELECTED_DEF
}

/** Becomes: invoke("agent_def", { name }) */
export function useFrontmatter(): FrontmatterLine[] {
  return FRONTMATTER
}

/** Becomes: invoke("agent_def", { name }) */
export function useProse(): string[] {
  return PROSE
}

/**
 * Becomes: invoke("materialization_report", { extension })
 *
 * How many harnesses this definition reaches, and how many of them take it
 * weaker than native. Computed from the registry, never written down.
 */
export function useAgentMaterialization(): { harnesses: number; downgraded: number } {
  const agents = EXTENSION_COUNTS.find((c) => c.type === 'agents')!
  return { harnesses: HARNESS_COUNT, downgraded: agents.downgraded }
}
