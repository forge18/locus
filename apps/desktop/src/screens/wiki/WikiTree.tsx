import { For, Show } from 'solid-js'
import { Button } from '../../ui/Button'
import { Icon } from '../../ui/Icon'
import { INGEST_NOTE, PAGE_KINDS, useWikiKindCounts, useWikiPagesByKind } from '../../data/wiki'

export interface WikiTreeProps {
  selectedId: string
  onSelect: (id: string) => void
  onIngest: () => void
}

/**
 * The primary action is ingest, not "New page". A wiki nobody writes is a wiki
 * nobody reads, so the default path is derived-then-curated. A human can still fix
 * a page — that is just not where you start.
 */
export function WikiTree(props: WikiTreeProps) {
  const counts = useWikiKindCounts()

  return (
    <nav class="wiki-tree" data-testid="wiki-tree" aria-label="Pages">
      <Button variant="primary" block data-testid="wiki-ingest" onClick={props.onIngest}>
        <Icon name="tray-arrow-down" size={11} />
        Ingest a document
      </Button>
      <span class="wiki-ingest-note" data-testid="wiki-ingest-note">
        {INGEST_NOTE}
      </span>

      <For each={PAGE_KINDS}>
        {(group) => (
          <>
            <div class="wiki-group" data-testid={`wiki-group-${group.kind}`}>
              {group.label}
              <span class="wiki-group-count" data-testid={`wiki-count-${group.kind}`}>
                {counts[group.kind]}
              </span>
            </div>
            <For each={useWikiPagesByKind(group.kind)}>
              {(page) => (
                <button
                  type="button"
                  class="wiki-page"
                  data-testid={`wiki-page-${page.id}`}
                  data-kind={page.kind}
                  aria-selected={props.selectedId === page.id ? 'true' : 'false'}
                  onClick={() => props.onSelect(page.id)}
                >
                  <Icon name={group.icon} size={11} />
                  <span class="wiki-page-title">{page.title}</span>
                  <Show when={page.orphan}>
                    <span class="wiki-orphan" data-testid={`wiki-orphan-${page.id}`}>
                      orphan
                    </span>
                  </Show>
                </button>
              )}
            </For>
          </>
        )}
      </For>
    </nav>
  )
}
