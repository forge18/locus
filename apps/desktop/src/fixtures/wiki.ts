// schema: wiki.pages + wiki.links + wiki.contradictions + wiki.ingest_log
// replaced by: invoke("wiki_pages") + invoke("wiki_graph") + invoke("wiki_lint")

import type { Contradiction, IngestLogEntry, PageKind, WikiLink, WikiPage } from '../types/wiki'

/** The six kinds, in tree order, with the glyph each group carries. */
export const PAGE_KINDS: Array<{ kind: PageKind; label: string; icon: string }> = [
  { kind: 'overview', label: 'Overview', icon: 'book-open-text' },
  { kind: 'decision', label: 'Decision', icon: 'gavel' },
  { kind: 'concept', label: 'Concept', icon: 'lightbulb' },
  { kind: 'entity', label: 'Entity', icon: 'cube' },
  { kind: 'synthesis', label: 'Synthesis', icon: 'sparkle' },
  { kind: 'source', label: 'Source', icon: 'file-text' },
]

/**
 * The counts the tree shows. Real pages are listed below; the totals are what
 * ingest has produced, which is more than any tree draws at once.
 */
export const KIND_COUNTS: Record<PageKind, number> = {
  overview: 1,
  decision: 14,
  concept: 31,
  entity: 42,
  synthesis: 8,
  source: 57,
}

const page = (
  id: string,
  title: string,
  kind: PageKind,
  extra: Partial<WikiPage> = {},
): WikiPage => ({
  id,
  projectId: 'p-tapestry',
  title,
  kind,
  body: [],
  revision: 1,
  assertions: 0,
  sources: 0,
  ingestedAgo: '4d ago',
  curatedAgo: '2d ago',
  links: [],
  provenance: [],
  orphan: false,
  ...extra,
})

export const PAGES: WikiPage[] = [
  page('w-overview', 'Locus architecture', 'overview'),

  page('w-clone', 'Clone from a local bare remote, never a mount', 'decision', {
    revision: 7,
    assertions: 3,
    sources: 2,
    body: [
      'Every project has a bare local remote on the host at `/var/lib/locus/repos/<project>.git`. An agent container clones from it into its own filesystem, commits, and pushes a branch back. The workspace is never bind-mounted.',
      'Isolation is real because the working copy was never there to escape into — a path bug cannot reach a filesystem that was not mounted. Cleanup is free: a finished container takes its clone with it. Reviewing the work stays ordinary git, so [[locus]] stays out of your editor and your merge tool.',
      'Cost, stated: clones take disk and time, mitigated by `git clone --reference` against a shared object store. Overlap surfaces at merge rather than being prevented — the conflict every team already has.',
    ],
    links: ['bare local remote', 'locus-agent credential', 'git invariant: never main', 'Sculptor'],
    provenance: [
      { icon: 'file-text', line: 'PLAN.md — "The git model — a local remote, not shared worktrees", ingested 4d ago' },
      { icon: 'git-pull-request', line: 'PR #491 body — repo manager, merge-back path' },
    ],
  }),
  page('w-no-mcp', 'No MCP servers, ever', 'decision'),
  page('w-codemirror', 'CodeMirror without a seam', 'decision'),
  page('w-board', 'Fixed board columns', 'decision'),

  page('w-determinism', 'Byte-deterministic materialization', 'concept'),
  page('w-waiting', 'Waiting ≠ idle', 'concept'),
  page('w-trifecta', 'The lethal trifecta', 'concept'),

  page('w-locusd', 'locusd', 'entity'),
  page('w-bare-remote', 'bare local remote', 'entity'),
  // Nothing links in. The tree flags it and the linter counts it — one condition.
  page('w-credential-broker', 'credential broker', 'entity', { orphan: true }),
  page('w-canary', 'canary token', 'entity', { orphan: true }),

  page('w-injection', 'Injection for the four that cannot', 'synthesis'),

  page('w-plan-md', 'PLAN.md', 'source'),
  page('w-adr-007', 'ADR-007 port allocation', 'source'),
  page('w-acp', 'agentclientprotocol.com', 'source'),
]

export const SELECTED_PAGE_ID = 'w-clone'

export const LINKS: WikiLink[] = [
  { fromPageId: 'w-clone', toPageId: 'w-bare-remote' },
  { fromPageId: 'w-clone', toPageId: 'w-locusd' },
  { fromPageId: 'w-determinism', toPageId: 'w-clone' },
  { fromPageId: 'w-no-mcp', toPageId: 'w-clone' },
  { fromPageId: 'w-plan-md', toPageId: 'w-clone' },
]

/** The graph the sidebar draws: the selected page and what it touches. */
export interface GraphNode {
  id: string
  label: string
  x: number
  y: number
  /** The centre node is the page you are on. */
  center: boolean
}

export const GRAPH_NODES: GraphNode[] = [
  { id: 'w-clone', label: 'clone, never a mount', x: 129, y: 66, center: true },
  { id: 'w-bare-remote', label: 'bare local remote', x: 42, y: 32, center: false },
  { id: 'w-locusd', label: 'locusd', x: 38, y: 100, center: false },
  { id: 'w-determinism', label: 'determinism', x: 214, y: 30, center: false },
  { id: 'w-no-mcp', label: 'no MCP', x: 220, y: 96, center: false },
  { id: 'w-plan-md', label: 'PLAN.md', x: 128, y: 118, center: false },
]

export const GRAPH_EDGES: Array<{ from: string; to: string }> = [
  { from: 'w-clone', to: 'w-bare-remote' },
  { from: 'w-clone', to: 'w-locusd' },
  { from: 'w-determinism', to: 'w-clone' },
  { from: 'w-no-mcp', to: 'w-clone' },
  { from: 'w-plan-md', to: 'w-clone' },
]

export const GRAPH_CAPTION =
  'Pages are nodes, wikilinks are edges — the canvas renderer, repointed.'

/**
 * A contradiction carries both statements and both sources. A flag you cannot
 * adjudicate yourself is just an alarm, so neither half is optional.
 */
export const CONTRADICTIONS: Contradiction[] = [
  {
    id: 'x-1',
    pageIds: ['w-adr-007', 'w-plan-md'],
    claim: 'Port range disagrees across two sources.',
    detectedAt: '2026-08-20T10:04:00Z',
    resolved: false,
    values: [
      { value: '43800-43999', source: 'PLAN.md', age: '4d' },
      { value: '44000-44999', source: 'ADR-007', age: '6h' },
    ],
    note: 'flagged at ingest, not at query',
  },
]

export interface LintFinding {
  kind: 'orphan' | 'broken_link' | 'unnamed_entity' | 'unsourced_assertion'
  count: number
  detail: string
}

/**
 * The lint counts come from the pages themselves — the orphan count is derived,
 * not typed in, so the tree flag and this card cannot disagree.
 */
export const LINT_FINDINGS: LintFinding[] = [
  {
    kind: 'orphan',
    count: PAGES.filter((p) => p.orphan).length,
    detail: PAGES.filter((p) => p.orphan)
      .map((p) => p.title)
      .join(', '),
  },
  { kind: 'broken_link', count: 1, detail: '[[egress tiers]]' },
  { kind: 'unnamed_entity', count: 3, detail: 'entities mentioned, never given a page' },
  { kind: 'unsourced_assertion', count: 1, detail: 'assertion with no source' },
]

export const LINT_CLEAN_LINE = '153 pages otherwise clean'

/** The distinction the screen exists to keep legible. */
export const MEMORY_DISTINCTION =
  'The wiki is curated prose a human reads. Memory is what an agent recalls — they share pgvector and nothing else.'

export const INGEST_NOTE = 'Derived, then curated — a path or a URL, not a blank page.'

export const INGEST_LOG: IngestLogEntry[] = [
  { id: 'g-1', source: 'PLAN.md', pagesWritten: 3, at: '2026-08-16T08:00:00Z', status: 'ok', note: null },
  { id: 'g-2', source: 'PR #491', pagesWritten: 1, at: '2026-08-18T08:00:00Z', status: 'ok', note: null },
  { id: 'g-3', source: 'weaver run 5a71', pagesWritten: 0, at: '2026-08-19T22:20:00Z', status: 'failed', note: 'run aborted before any artifact was written' },
]
