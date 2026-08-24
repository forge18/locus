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
const STATES = ["loading", "empty", "error"];

/** Every viewer declares the same bounded state family before live data replaces fixtures. */
export function ViewerStateFamilies() {
  return (
    <section
      data-testid="viewer-state-families"
      aria-live="polite"
      data-visual-themes="light,dark"
    >
      {VIEWERS.flatMap((viewer) =>
        STATES.map((state) => <div data-viewer-state={`${viewer}:${state}`} />),
      )}
    </section>
  );
}
