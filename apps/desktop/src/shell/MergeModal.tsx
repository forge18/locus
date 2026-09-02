import { createSignal, Show } from "solid-js";

export interface MergeModalProps {
  open: boolean;
  branch?: string;
  repo?: string;
  onClose?: () => void;
  /** Runs the merge; the modal closes only after it resolves successfully. */
  onMerge?: () => void | Promise<void>;
}

export function MergeModal(props: MergeModalProps) {
  const [mergeError, setMergeError] = createSignal<string | null>(null);
  const [merging, setMerging] = createSignal(false);

  const confirmMerge = async () => {
    if (merging()) return;

    setMergeError(null);
    if (!props.onMerge) {
      setMergeError("Merge operation is unavailable.");
      return;
    }

    setMerging(true);
    try {
      await props.onMerge();
      props.onClose?.();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setMergeError(message || "Merge failed.");
    } finally {
      setMerging(false);
    }
  };

  return (
    <Show when={props.open}>
      <div class="merge-scrim" role="presentation">
        <section
          class="merge-modal"
          role="dialog"
          aria-labelledby="merge-title"
          data-testid="merge-modal"
        >
          <header>
            <h2 id="merge-title">The work becomes yours.</h2>
            <button type="button" aria-label="Close" onClick={props.onClose}>
              ×
            </button>
          </header>
          <p class="mono">
            {props.branch ?? "agent branch"}
            {props.repo ? ` · ${props.repo}` : ""} · the only irreversible step
            in the run.
          </p>
          <div class="merge-columns">
            <section>
              <h3>Evidence travelling with it</h3>
              <ul>
                <li>Verify command and exit code</li>
                <li>Plan clauses satisfied</li>
                <li>Analyzer result</li>
                <li>Changed files opened</li>
              </ul>
            </section>
            <section>
              <h3>Size of the change</h3>
              <ul>
                <li>Files, added, removed</li>
                <li>Inside the guardrail</li>
              </ul>
            </section>
          </div>
          <p class="merge-warning">
            An approval granted without opening the artifact is the measured
            failure mode, not a hypothetical one.
          </p>
          <Show when={mergeError()}>
            <p class="merge-error" role="alert" data-testid="merge-error">
              {mergeError()}
            </p>
          </Show>
          <footer>
            <button type="button" onClick={props.onClose}>
              Open the two files first
            </button>
            <button
              type="button"
              data-testid="merge-confirm"
              disabled={merging()}
              onClick={() => void confirmMerge()}
            >
              Merge and close the task
            </button>
          </footer>
        </section>
      </div>
    </Show>
  );
}
