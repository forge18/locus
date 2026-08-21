// schema: core.repos + agents.runs (the branch a run pushed) + agents.artifacts (kind = 'diff')
// replaced by: invoke("git_status") + invoke("file_tree") + invoke("diff_for_file")

export interface TreeNode {
  path: string
  name: string
  depth: number
  kind: 'dir' | 'file'
  /** Git status letter, or null for unchanged. */
  status: 'M' | 'A' | '?' | null
}

export const BRANCH = 'agent/8f21-notify'

export const FILE_TREE: TreeNode[] = [
  { path: 'crates/locus-core/src', name: 'crates/locus-core/src', depth: 0, kind: 'dir', status: null },
  { path: 'crates/locus-core/src/store', name: 'store', depth: 1, kind: 'dir', status: null },
  { path: 'crates/locus-core/src/store/notify.rs', name: 'notify.rs', depth: 2, kind: 'file', status: 'M' },
  { path: 'crates/locus-core/src/store/pool.rs', name: 'pool.rs', depth: 2, kind: 'file', status: null },
  { path: 'crates/locus-core/src/store/mod.rs', name: 'mod.rs', depth: 2, kind: 'file', status: 'M' },
  { path: 'crates/locus-core/src/materialize', name: 'materialize', depth: 1, kind: 'dir', status: null },
  { path: 'crates/locus-core/src/telemetry', name: 'telemetry', depth: 1, kind: 'dir', status: null },
  { path: 'crates/locus-core/src/memory', name: 'memory', depth: 1, kind: 'dir', status: null },
  { path: 'apps/desktop/src', name: 'apps/desktop/src', depth: 0, kind: 'dir', status: null },
  { path: 'migrations', name: 'migrations', depth: 0, kind: 'dir', status: null },
]

export const SELECTED_FILE = 'crates/locus-core/src/store/notify.rs'

export interface EditorTab {
  path: string
  name: string
  /** The tab's own diff mode, shown under the name on the active tab. */
  mode: string
}

export const TABS: EditorTab[] = [
  { path: 'crates/locus-core/src/store/notify.rs', name: 'notify.rs', mode: 'diff' },
  { path: 'crates/locus-core/src/store/mod.rs', name: 'mod.rs', mode: 'diff' },
  { path: 'crates/locus-core/src/board.rs', name: 'board.rs', mode: 'diff' },
]

export type DiffRowKind = 'context' | 'added' | 'removed' | 'fold'

export interface DiffCell {
  no: number
  text: string
}

export interface DiffRow {
  kind: DiffRowKind
  /** Null on an added row: the left side has nothing there. */
  left: DiffCell | null
  /** Null on a removed row. */
  right: DiffCell | null
  /** Set on a fold row: how many unchanged lines it stands in for. */
  foldCount?: number
}

/**
 * A hunk is the unit of staging. Per-file staging is the coarse case of it, and
 * the granularity is the reason the git panel exists rather than a status readout.
 */
export interface Hunk {
  id: string
  /** The `@@` header, as git writes it. */
  header: string
  rows: DiffRow[]
  staged: boolean
}

const fold = (n: number): DiffRow => ({ kind: 'fold', left: null, right: null, foldCount: n })
const ctx = (leftNo: number, rightNo: number, text: string): DiffRow => ({
  kind: 'context',
  left: { no: leftNo, text },
  right: { no: rightNo, text },
})
const removed = (no: number, text: string): DiffRow => ({
  kind: 'removed',
  left: { no, text },
  right: null,
})
const added = (no: number, text: string): DiffRow => ({
  kind: 'added',
  left: null,
  right: { no, text },
})

export const HUNKS: Hunk[] = [
  {
    id: 'h-1',
    header: '@@ -18,7 +18,9 @@ impl Notifier',
    staged: true,
    rows: [
      fold(18),
      ctx(19, 19, '    pub async fn notify(&self, ch: &str, id: Uuid) -> Result<()> {'),
      removed(20, '        sqlx::query("SELECT pg_notify($1, $2)")'),
      added(20, '        // NOTIFY carries an id only — payload cap is 8000 bytes'),
      added(21, '        sqlx::query("SELECT pg_notify($1, $2)")'),
      removed(21, '            .bind(ch).bind(serde_json::to_string(&row)?)'),
      added(22, '            .bind(ch).bind(id.to_string())'),
      ctx(22, 23, '            .execute(&self.pool).await?;'),
      ctx(23, 24, '        Ok(())'),
      ctx(24, 25, '    }'),
      fold(41),
    ],
  },
  {
    id: 'h-2',
    header: '@@ -71,4 +73,6 @@ impl Notifier',
    staged: false,
    rows: [
      fold(12),
      ctx(72, 74, '    /// Listener reconnects with backoff.'),
      removed(73, '        let mut backoff = Duration::from_millis(50);'),
      added(75, '        let mut backoff = Duration::from_millis(100);'),
      ctx(74, 76, '        loop {'),
      fold(9),
    ],
  },
]

export const DIFF_LEFT_HEADER = 'HEAD · main'
export const DIFF_RIGHT_HEADER = `${BRANCH} · builder@4`

export interface GitFile {
  path: string
  status: 'M' | 'A' | '?'
  added: number
  removed: number
}

export interface Commit {
  sha: string
  subject: string
  author: string
  age: string
}

export interface GitPanel {
  branch: string
  from: string
  pushedBy: string
  pushedAgo: string
  ahead: number
  behind: number
  staged: GitFile[]
  unstaged: GitFile[]
  history: Commit[]
  lsp: string
}

export const GIT: GitPanel = {
  branch: BRANCH,
  from: 'main',
  pushedBy: 'builder@4',
  pushedAgo: '6m ago',
  ahead: 2,
  behind: 0,
  staged: [
    { path: 'crates/locus-core/src/store/notify.rs', status: 'M', added: 9, removed: 2 },
    { path: 'crates/locus-core/src/store/mod.rs', status: 'M', added: 1, removed: 1 },
  ],
  unstaged: [
    { path: 'crates/locus-core/src/store/notify_test.rs', status: 'A', added: 61, removed: 0 },
    { path: 'migrations/0042_notify.sql', status: '?', added: 0, removed: 0 },
  ],
  // Newest first. Only the newest carries the halo.
  history: [
    { sha: '8f21a4c', subject: 'cap NOTIFY payload at the row id', author: 'builder@4', age: '6m' },
    { sha: '11c9e0f', subject: 'add listener reconnect backoff', author: 'builder@4', age: '22m' },
    { sha: 'a0bc3a4', subject: 'branch from main', author: 'you', age: '1h' },
  ],
  lsp: 'rust-analyzer · 0 errors · 2 hints',
}

/** Your own checkout, which the agent cannot reach — it clones, never mounts. */
export const LINKED_CHECKOUT = '~/Repos/tapestry'
export const LINKED_REPO_NOTE = `Linked repo · your own checkout at ${LINKED_CHECKOUT}`

export const COMMIT_PLACEHOLDER =
  'Commit message — agent commits are authored, not squashed'

/**
 * The clearest statement of the git model, and the easiest thing for a person to
 * get wrong. It renders verbatim.
 */
export const OWNERSHIP_NOTE =
  'Working tree is your own checkout — the agent pushed to the branch, you decide what lands.'

export const PRIMARY_SURFACE_NOTE =
  'Reviewing what an agent changed is the primary editor surface'
