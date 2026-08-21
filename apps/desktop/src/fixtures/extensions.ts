// schema: agents.agent_defs + the authored extension tree (no Postgres table yet)
// replaced by: invoke("extension_inventory") + invoke("recently_edited")
//
// The per-type item counts here are *authored* content — how many agents you have
// written. Everything about how those reach a harness is computed from
// harnesses/*.toml instead, and lives in generated/harnesses.ts.

export const HEADER_TITLE = 'The one surface'
export const HEADER_NOTE =
  'Eight extension types, authored once here, materialized fresh into every runtime at run start'

export const DETERMINISM_NOTE =
  'Materialization is byte-deterministic: sorted file order, sorted lists, no timestamps, no run id, no hostname. The tree is the prompt prefix, so a per-run difference is a cache miss for every agent that harness serves.'

/** How many of each type are authored, and what each type is for. */
export interface TypeCard {
  type: string
  icon: string
  count: number
  description: string
}

export const TYPE_CARDS: TypeCard[] = [
  { type: 'agents', icon: 'robot', count: 14, description: 'who does the work, versioned' },
  { type: 'skills', icon: 'sparkle', count: 31, description: 'model-invocable procedures' },
  { type: 'rules', icon: 'scales', count: 22, description: 'always-on, path-scoped' },
  { type: 'context', icon: 'file-text', count: 4, description: 'the base file every harness reads' },
  { type: 'commands', icon: 'terminal-window', count: 18, description: 'what you invoke by name' },
  { type: 'hooks', icon: 'plugs', count: 9, description: 'fired on lifecycle events' },
  { type: 'output-styles', icon: 'pen-nib', count: 5, description: 'how the agent writes back' },
  { type: 'linters', icon: 'broom', count: 11, description: 'a check plus why it exists' },
]

/** The card you enter the agent-definitions drill-down from. */
export const ENTRY_TYPE = 'agents'

export interface EditedEntry {
  type: string
  file: string
  summary: string
  age: string
}

export const RECENTLY_EDITED: EditedEntry[] = [
  { type: 'agents', file: 'builder.md', summary: 'tool list narrowed to read-only for review', age: '2h' },
  { type: 'rules', file: 'no-secrets.md', summary: 'added the rotate-not-delete line', age: '5h' },
  { type: 'skills', file: 'verify-loop.md', summary: 'three iterations, then hand off', age: '1d' },
  { type: 'linters', file: 'no-todo-comments.sh', summary: 'why: a TODO is a decision deferred silently', age: '2d' },
]

/**
 * Measured, not computed from the registry: how often a materialized tree hit the
 * model's prompt cache. It is the number byte-determinism exists to protect.
 */
export const CACHE_READ_RATE = '84%'

export const SEARCH_PLACEHOLDER = 'Search extensions'
export const NEW_LABEL = 'New'
