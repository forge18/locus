import { For, Show } from 'solid-js'
import { Icon } from '../../ui/Icon'
import type { WikiPage } from '../../types/wiki'

export interface WikiArticleProps {
  page: WikiPage
  /** A wikilink goes to a page by locator. */
  onFollow: (slug: string) => void
}

/** Backticked spans become mono. The source is curated prose, not user input. */
function mono(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/`([^`]+)`/g, '<code class="mono">$1</code>')
    .replace(/\[\[([^\]]+)\]\]/g, '<span class="mono">[[$1]]</span>')
}

export function WikiArticle(props: WikiArticleProps) {
  return (
    <article class="wiki-article" data-testid="wiki-article">
      <header class="wiki-article-head">
        <span class="wiki-kind" data-testid="wiki-article-kind">
          {props.page.kind}
        </span>
        <h1 class="wiki-article-title" data-testid="wiki-article-title">
          {props.page.title}
        </h1>
      </header>

      <div class="wiki-article-meta" data-testid="wiki-article-meta">
        <span class="mono" data-testid="wiki-article-locator">
          locus://tapestry/page/{props.page.id.replace(/^w-/, '')}
        </span>
        <span class="mono" data-testid="wiki-article-rev">
          rev {props.page.revision}
        </span>
        <span data-testid="wiki-article-counts">
          {props.page.assertions} assertions · {props.page.sources} sources
        </span>
        <span data-testid="wiki-article-ages">
          Ingest {props.page.ingestedAgo}, curated {props.page.curatedAgo}
        </span>
      </div>

      <div class="wiki-prose" data-testid="wiki-prose">
        <For each={props.page.body}>{(para) => <p style={{ margin: 0 }} innerHTML={mono(para)} />}</For>
      </div>

      <Show when={props.page.links.length > 0}>
        <div class="wiki-section">Links out</div>
        <div class="wiki-links" data-testid="wiki-links">
          <For each={props.page.links}>
            {(link) => (
              <button
                type="button"
                class="wikilink"
                data-testid={`wikilink-${link.replace(/\W+/g, '-')}`}
                data-slug={link}
                onClick={() => props.onFollow(link)}
              >
                [[{link}]]
              </button>
            )}
          </For>
        </div>
      </Show>

      <Show when={props.page.provenance.length > 0}>
        <div class="wiki-section">Provenance</div>
        <div class="wiki-provenance" data-testid="wiki-provenance">
          <For each={props.page.provenance}>
            {(row) => (
              <div class="wiki-provenance-row">
                <Icon name={row.icon} size={11} style={{ 'flex-shrink': 0, 'margin-top': '2px' }} />
                <span>{row.line}</span>
              </div>
            )}
          </For>
        </div>
      </Show>
    </article>
  )
}
