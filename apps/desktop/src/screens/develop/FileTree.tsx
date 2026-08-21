import { For, Show } from 'solid-js'
import { Icon } from '../../ui/Icon'
import { Resizable } from '../../panes/Resizable'
import { LINKED_REPO_NOTE, useBranch, useFileTree } from '../../data/develop'

export interface FileTreeProps {
  selectedPath: string
  onSelect: (path: string) => void
}

/** 20px for the first indent, 34px for the second, as the design draws it. */
const INDENT = [0, 20, 34]

const statusClass = (status: 'M' | 'A' | '?') =>
  status === 'M' ? 'dev-status-M' : status === 'A' ? 'dev-status-A' : 'dev-status-unknown'

export function FileTree(props: FileTreeProps) {
  return (
    <Resizable width={206} min={160} max={400} side="right" class="dev-tree" testId="dev-tree">
      <div class="dev-tree-head" data-testid="dev-tree-head">
        <Icon name="git-branch" size={11} />
        <span data-testid="dev-tree-branch">{useBranch()}</span>
        <Icon name="caret-down" size={9} style={{ 'margin-left': 'auto' }} />
      </div>

      <div class="dev-tree-body">
        <For each={useFileTree()}>
          {(node) => (
            <button
              type="button"
              class="dev-tree-row"
              data-testid={`dev-tree-row-${node.path}`}
              data-depth={node.depth}
              aria-selected={props.selectedPath === node.path ? 'true' : 'false'}
              style={{ 'padding-left': `${INDENT[node.depth] ?? node.depth * 14}px` }}
              onClick={() => node.kind === 'file' && props.onSelect(node.path)}
            >
              <Icon
                name={node.kind === 'dir' ? 'caret-down' : 'file-code'}
                size={10}
                style={{ 'flex-shrink': 0 }}
              />
              <span class="dev-tree-name">{node.name}</span>
              <Show when={node.status}>
                <span
                  class={`dev-status ${statusClass(node.status!)}`}
                  data-testid={`dev-tree-status-${node.path}`}
                >
                  {node.status}
                </span>
              </Show>
            </button>
          )}
        </For>
      </div>

      <footer class="dev-tree-foot" data-testid="dev-tree-foot">
        {LINKED_REPO_NOTE}
      </footer>
    </Resizable>
  )
}
