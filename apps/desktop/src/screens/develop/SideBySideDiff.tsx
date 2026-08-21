import { For, Show } from 'solid-js'
import type { JSX } from 'solid-js'
import { Button } from '../../ui/Button'
import { DIFF_LEFT_HEADER, DIFF_RIGHT_HEADER } from '../../data/develop'
import type { DiffCell, DiffRow, Hunk } from '../../data/develop'

export interface SideBySideDiffProps {
  hunks: Hunk[]
  /** Stage or unstage one hunk. Per-hunk is the granularity the panel exists for. */
  onToggleHunk: (id: string) => void
}

/** Enough of Rust to colour a diff. The real grammar arrives with CodeMirror at M2. */
const KEYWORDS = new Set([
  'pub', 'async', 'fn', 'let', 'mut', 'impl', 'use', 'match', 'if', 'else',
  'return', 'loop', 'await', 'self', 'Ok', 'Err', 'struct', 'enum', 'trait',
])

/** Comments are one span; everything else is split on word boundaries. */
export function tokenize(text: string): JSX.Element {
  const comment = text.indexOf('//')
  if (comment >= 0) {
    return (
      <>
        {tokenize(text.slice(0, comment))}
        <span class="tok-comment">{text.slice(comment)}</span>
      </>
    )
  }
  return (
    <For each={text.split(/(\b)/)}>
      {(part) =>
        KEYWORDS.has(part) ? <span class="tok-keyword">{part}</span> : <>{part}</>
      }
    </For>
  )
}

const rowClass = (row: DiffRow, side: 'left' | 'right') => {
  if (row.kind === 'fold') return 'diff-row diff-row-fold'
  if (row.kind === 'added') return side === 'right' ? 'diff-row diff-row-added' : 'diff-row'
  if (row.kind === 'removed') return side === 'left' ? 'diff-row diff-row-removed' : 'diff-row'
  return 'diff-row'
}

const sign = (row: DiffRow, side: 'left' | 'right') => {
  if (row.kind === 'added' && side === 'right') return '+'
  if (row.kind === 'removed' && side === 'left') return '−'
  return ''
}

function Side(props: { hunks: Hunk[]; side: 'left' | 'right' }) {
  return (
    <div
      class={['diff-side', props.side === 'right' ? 'diff-side-right' : ''].filter(Boolean).join(' ')}
      data-testid={`diff-side-${props.side}`}
    >
      <For each={props.hunks}>
        {(hunk) => (
          <For each={hunk.rows}>
            {(row) => {
              const cell = (): DiffCell | null => (props.side === 'left' ? row.left : row.right)
              return (
                <Show
                  when={row.kind !== 'fold'}
                  fallback={
                    <div
                      class="diff-row diff-row-fold"
                      data-testid={`diff-fold-${props.side}-${row.foldCount}`}
                    >
                      <span class="diff-fold-text">⋯ {row.foldCount} unchanged lines</span>
                    </div>
                  }
                >
                  <div
                    class={rowClass(row, props.side)}
                    data-kind={row.kind}
                    data-hunk={hunk.id}
                  >
                    <span class="diff-gutter">{cell()?.no ?? ''}</span>
                    <span class="diff-sign">{sign(row, props.side)}</span>
                    <span class="diff-text">{cell() ? tokenize(cell()!.text) : ''}</span>
                  </div>
                </Show>
              )
            }}
          </For>
        )}
      </For>
    </div>
  )
}

export function SideBySideDiff(props: SideBySideDiffProps) {
  return (
    <div class="diff" data-testid="diff">
      <div class="diff-headers">
        <span class="diff-header diff-header-left" data-testid="diff-header-left">
          {DIFF_LEFT_HEADER}
        </span>
        <span class="diff-header diff-header-right" data-testid="diff-header-right">
          {DIFF_RIGHT_HEADER}
        </span>
      </div>

      <div class="diff-body">
        <Side hunks={props.hunks} side="left" />
        <Side hunks={props.hunks} side="right" />
      </div>

      <div class="diff-hunk-actions">
        <For each={props.hunks}>
          {(hunk) => (
            <Button
              variant="ghost"
              data-testid={`hunk-toggle-${hunk.id}`}
              onClick={() => props.onToggleHunk(hunk.id)}
            >
              {hunk.staged ? 'Unstage hunk' : 'Stage hunk'} {hunk.header.split(' ')[1]}
            </Button>
          )}
        </For>
      </div>
    </div>
  )
}
