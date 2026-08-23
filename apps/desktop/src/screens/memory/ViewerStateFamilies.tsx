const VIEWERS = ['inbox', 'dashboard', 'develop', 'review', 'memory-short-term', 'memory-long-term', 'memory-artifacts', 'memory-wiki']
const STATES = ['loading', 'empty', 'error']

/** Every viewer declares the same bounded state family before live data replaces fixtures. */
export function ViewerStateFamilies() {
  return <section data-testid="viewer-state-families">{VIEWERS.flatMap((viewer) => STATES.map((state) => <div data-viewer-state={`${viewer}:${state}`} />))}</section>
}
