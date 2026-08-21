import { For, Show } from 'solid-js'
import { Button } from '../../ui/Button'
import { Icon } from '../../ui/Icon'
import { Input } from '../../ui/Input'
import { Resizable } from '../../panes/Resizable'
import { COMMIT_PLACEHOLDER, OWNERSHIP_NOTE } from '../../data/develop'
import type { GitFile, GitPanel as GitPanelData } from '../../data/develop'

export interface GitPanelProps {
  git: GitPanelData
  currentFile: string
  /** Stage or unstage one file. The coarse case of the per-hunk operation. */
  onToggleFile: (path: string) => void
  onStageAll: () => void
  onUnstageAll: () => void
}

const statusClass = (status: GitFile['status']) =>
  status === 'M' ? 'dev-status-M' : status === 'A' ? 'dev-status-A' : 'dev-status-unknown'

function FileRow(props: {
  file: GitFile
  current: boolean
  onToggle: () => void
}) {
  return (
    <button
      type="button"
      class={['git-row', props.current ? 'git-row-current' : ''].filter(Boolean).join(' ')}
      data-testid={`git-row-${props.file.path}`}
      data-current={props.current ? 'true' : undefined}
      onClick={props.onToggle}
    >
      <span
        class={`dev-status ${statusClass(props.file.status)}`}
        data-testid={`git-status-${props.file.path}`}
      >
        {props.file.status}
      </span>
      <span class="git-path" data-testid={`git-path-${props.file.path}`}>
        {props.file.path}
      </span>
      <Show when={props.file.added > 0}>
        <span class="git-added">+{props.file.added}</span>
      </Show>
      <Show when={props.file.removed > 0}>
        <span class="git-removed">−{props.file.removed}</span>
      </Show>
    </button>
  )
}

/**
 * Read-write on *your* checkout, while the branch is the agent's. That is the
 * whole point of the local-remote model, and the footer note is the clearest
 * statement of it.
 */
export function GitPanel(props: GitPanelProps) {
  return (
    <Resizable width={252} min={200} max={440} side="left" class="git-panel" testId="git-panel">
      <div class="git-head" data-testid="git-head">
        Git
        <span class="git-ahead" data-testid="git-ahead">
          {props.git.ahead}↑
        </span>
        <span class="git-behind" data-testid="git-behind">
          {props.git.behind}↓
        </span>
        <button type="button" class="git-refresh" aria-label="Refresh">
          <Icon name="arrows-clockwise" size={11} />
        </button>
      </div>

      <div class="git-branch-block" data-testid="git-branch-block">
        <span class="git-branch-line">
          <Icon name="git-branch" size={11} style={{ color: 'var(--ac)' }} />
          {props.git.branch}
        </span>
        <span class="git-branch-from" data-testid="git-branch-from">
          from {props.git.from} · pushed by {props.git.pushedBy} {props.git.pushedAgo}
        </span>
      </div>

      <div class="git-body">
        <div class="git-section" data-testid="git-section-staged">
          Staged
          <span class="mono" style={{ 'letter-spacing': '0' }}>
            {props.git.staged.length}
          </span>
          <button
            type="button"
            class="git-bulk"
            data-testid="git-unstage-all"
            onClick={props.onUnstageAll}
          >
            Unstage all
          </button>
        </div>
        <For each={props.git.staged}>
          {(file) => (
            <FileRow
              file={file}
              current={file.path === props.currentFile}
              onToggle={() => props.onToggleFile(file.path)}
            />
          )}
        </For>

        <div class="git-section" data-testid="git-section-unstaged">
          Unstaged
          <span class="mono" style={{ 'letter-spacing': '0' }}>
            {props.git.unstaged.length}
          </span>
          <button
            type="button"
            class="git-bulk"
            data-testid="git-stage-all"
            onClick={props.onStageAll}
          >
            Stage all
          </button>
        </div>
        <For each={props.git.unstaged}>
          {(file) => (
            <FileRow
              file={file}
              current={file.path === props.currentFile}
              onToggle={() => props.onToggleFile(file.path)}
            />
          )}
        </For>

        <div class="git-section" data-testid="git-section-history">
          History
          <span style={{ 'letter-spacing': '0', 'text-transform': 'none' }}>· this branch</span>
        </div>
        <For each={props.git.history}>
          {(commit, i) => (
            <div class="git-history-row" data-testid={`git-commit-${commit.sha}`}>
              <span
                class={['git-dot', i() === 0 ? 'git-dot-newest' : ''].filter(Boolean).join(' ')}
                data-newest={i() === 0 ? 'true' : undefined}
              />
              <div>
                <div class="git-commit-subject">{commit.subject}</div>
                <div class="git-commit-meta">
                  {commit.sha} · {commit.author} · {commit.age}
                </div>
              </div>
            </div>
          )}
        </For>
      </div>

      <footer class="git-foot" data-testid="git-foot">
        <Input data-testid="git-commit-message" placeholder={COMMIT_PLACEHOLDER} />
        <div class="git-foot-actions">
          <Button variant="primary" data-testid="git-commit">
            <Icon name="check" size={11} />
            Commit
          </Button>
          <Button variant="secondary" data-testid="git-push">
            <Icon name="arrow-up" size={11} />
            Push
          </Button>
        </div>
        <span class="git-foot-note" data-testid="git-foot-note">
          {OWNERSHIP_NOTE}
        </span>
      </footer>
    </Resizable>
  )
}
