import { createMemo, createSignal } from 'solid-js'
import { WikiArticle } from './WikiArticle'
import { WikiSidebar } from './WikiSidebar'
import { WikiTree } from './WikiTree'
import { useDefaultPageId, useWikiPages } from '../../data/wiki'
import type { NavStore } from '../../nav'

export interface WikiViewProps {
  nav: NavStore
}

/**
 * Curated prose a human reads. The wiki is not memory: they share pgvector and
 * nothing else, and the sidebar footer says so where it cannot be missed.
 */
export function WikiView(props: WikiViewProps) {
  const [selectedId, setSelectedId] = createSignal(useDefaultPageId())
  const pages = useWikiPages()
  const selected = createMemo(() => pages.find((p) => p.id === selectedId()) ?? pages[0])

  /** A wikilink is an address, so following one goes through the resolver. */
  const follow = (slug: string) => {
    props.nav.open(`locus://tapestry/page/${slug.toLowerCase().replace(/[^a-z0-9]+/g, '-')}`)
  }

  return (
    <div class="wiki" data-testid="wiki">
      <WikiTree selectedId={selectedId()} onSelect={setSelectedId} onIngest={() => {}} />
      <WikiArticle page={selected()} onFollow={follow} />
      <WikiSidebar />
    </div>
  )
}

/** Default export so the view can be code-split at the route boundary. */
export default WikiView
