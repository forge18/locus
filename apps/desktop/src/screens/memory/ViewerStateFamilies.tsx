const VIEWERS = [
  "inbox",
  "status",
  "sessions",
  "qa",
  "short",
  "memory",
  "artifact",
  "wiki",
];
export type ViewerStateKind = "loading" | "empty" | "error" | "loaded";

const STATES: readonly ViewerStateKind[] = [
  "loading",
  "empty",
  "error",
  "loaded",
];

/**
 * The shared state marker keeps viewer seams honest: an empty result is not a
 * request in flight, and either is distinct from a failed request. This seam
 * carries no fixture content; each routed viewer supplies its own real data.
 */
export function ViewerState(props: {
  viewer: string;
  state: ViewerStateKind;
}) {
  return (
    <div
      data-viewer-state={`${props.viewer}:${props.state}`}
      data-state={props.state}
      aria-label={`${props.viewer} ${props.state}`}
    />
  );
}

/** Every viewer declares the same bounded state family before live data replaces fixtures. */
export function ViewerStateFamilies() {
  return (
    <section
      data-testid="viewer-state-families"
      aria-live="polite"
      data-visual-themes="light,dark"
    >
      {VIEWERS.flatMap((viewer) =>
        STATES.map((state) => <ViewerState viewer={viewer} state={state} />),
      )}
    </section>
  );
}
