import { For, Show, createMemo, createSignal } from 'solid-js'
import { FileTree } from './FileTree'
import { GitPanel } from './GitPanel'
import { SideBySideDiff } from './SideBySideDiff'
import { Button } from '../../ui/Button'
import { Icon } from '../../ui/Icon'
import {
  PRIMARY_SURFACE_NOTE,
  useEditorTabs,
  useGitPanel,
  useHunks,
  useSelectedFile,
} from '../../data/develop'
import type { GitFile } from '../../data/develop'

/**
 * Reviewing what an agent changed is the primary editor job, so the diff is the
 * default surface rather than a mode you switch into. The real CodeMirror instance
 * and real LSP arrive with `editor` at M2; this is the frame they land in.
 */
export function DevelopView() {
  const [selectedPath, setSelectedPath] = createSignal(useSelectedFile())
  const [activeTab, setActiveTab] = createSignal(useSelectedFile())
  const [collapseUnchanged, setCollapseUnchanged] = createSignal(true)
  const [diffMode, setDiffMode] = createSignal<'split' | 'unified'>('split')

  const [hunks, setHunks] = createSignal(useHunks())
  const git = useGitPanel()
  const [staged, setStaged] = createSignal<GitFile[]>(git.staged)
  const [unstaged, setUnstaged] = createSignal<GitFile[]>(git.unstaged)

  const chunks = createMemo(() => hunks().length)

  /** Per hunk: only that hunk moves. */
  const toggleHunk = (id: string) =>
    setHunks(hunks().map((h) => (h.id === id ? { ...h, staged: !h.staged } : h)))

  /** Per file: the coarse case of the same operation. */
  const toggleFile = (path: string) => {
    const inStaged = staged().find((f) => f.path === path)
    if (inStaged) {
      setStaged(staged().filter((f) => f.path !== path))
      setUnstaged([...unstaged(), inStaged])
      return
    }
    const inUnstaged = unstaged().find((f) => f.path === path)
    if (inUnstaged) {
      setUnstaged(unstaged().filter((f) => f.path !== path))
      setStaged([...staged(), inUnstaged])
    }
  }

  const tabs = useEditorTabs()

  return (
    <div class="develop" data-testid="develop" data-v2-route="develop">
      <FileTree
        selectedPath={selectedPath()}
        onSelect={(path) => {
          setSelectedPath(path)
          setActiveTab(path)
        }}
      />

      <section class="dev-editor" data-testid="dev-editor">
        <div class="dev-tabs" data-testid="dev-tabs">
          <For each={tabs}>
            {(tab) => (
              <div
                class="dev-tab"
                data-testid={`dev-tab-${tab.name}`}
                aria-selected={activeTab() === tab.path ? 'true' : 'false'}
                onClick={() => setActiveTab(tab.path)}
              >
                <Icon name="file-code" size={10} />
                {tab.name}
                <Show when={activeTab() === tab.path}>
                  <span class="mono" style={{ 'font-size': '8.5px', color: 'var(--text-muted)' }}>
                    {tab.mode}
                  </span>
                  <button
                    type="button"
                    class="dev-tab-close"
                    aria-label={`Close ${tab.name}`}
                    data-testid={`dev-tab-close-${tab.name}`}
                  >
                    ×
                  </button>
                </Show>
              </div>
            )}
          </For>
          <div class="dev-tabs-right">
            <button type="button" data-testid="diff-mode-split" aria-pressed={diffMode() === 'split'} onClick={() => setDiffMode('split')}>Split</button>
            <button type="button" data-testid="diff-mode-unified" aria-pressed={diffMode() === 'unified'} onClick={() => setDiffMode('unified')}>Unified</button>
            <button
              type="button"
              class="git-bulk"
              style={{ color: 'var(--text-muted)' }}
              data-testid="collapse-unchanged"
              aria-pressed={collapseUnchanged() ? 'true' : 'false'}
              onClick={() => setCollapseUnchanged(!collapseUnchanged())}
            >
              collapseUnchanged
            </button>
            <span class="dev-chunks" data-testid="dev-chunks">
              {chunks()} chunks
            </span>
          </div>
        </div>

        <SideBySideDiff hunks={hunks()} onToggleHunk={toggleHunk} />

        <section class="dev-terminal" data-testid="develop-terminal"><strong>Terminal · agent/8f21-notify</strong><code>$ cargo test -p locus-core notify::</code><span>This linked repo is your working copy; the agent works in its own clone.</span></section>

        <footer class="dev-footer" data-testid="dev-footer">
          <Button variant="secondary" data-testid="dev-revert">
            <Icon name="arrow-counter-clockwise" size={11} />
            Revert chunk
          </Button>
          <Button variant="primary" data-testid="dev-open-pr">
            <Icon name="git-pull-request" size={11} />
            Open PR from this branch
          </Button>
          <span class="dev-lsp" data-testid="dev-lsp">
            {git.lsp}
          </span>
          <span class="dev-footer-note" data-testid="dev-footer-note">
            {PRIMARY_SURFACE_NOTE}
          </span>
        </footer>
      </section>

      <GitPanel
        git={{ ...git, staged: staged(), unstaged: unstaged() }}
        currentFile={selectedPath()}
        onToggleFile={toggleFile}
        onStageAll={() => {
          setStaged([...staged(), ...unstaged()])
          setUnstaged([])
        }}
        onUnstageAll={() => {
          setUnstaged([...unstaged(), ...staged()])
          setStaged([])
        }}
      />
    </div>
  )
}

/** Default export so the view can be code-split at the route boundary. */
export default DevelopView
