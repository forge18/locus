import { Show } from 'solid-js'

export interface MergeModalProps { open: boolean; branch?: string; onClose?: () => void }

export function MergeModal(props: MergeModalProps) {
  return <Show when={props.open}>
    <div class="merge-scrim" role="presentation">
      <section class="merge-modal" role="dialog" aria-labelledby="merge-title" data-testid="merge-modal">
        <header><h2 id="merge-title">The work becomes yours.</h2><button type="button" aria-label="Close" onClick={props.onClose}>×</button></header>
        <p class="mono">{props.branch ?? 'agent branch'} · the only irreversible step in the run.</p>
        <div class="merge-columns">
          <section><h3>Evidence travelling with it</h3><ul><li>Verify command and exit code</li><li>Plan clauses satisfied</li><li>Analyzer result</li><li>Changed files opened</li></ul></section>
          <section><h3>Size of the change</h3><ul><li>Files, added, removed</li><li>Inside the guardrail</li></ul></section>
        </div>
        <p class="merge-warning">An approval granted without opening the artifact is the measured failure mode, not a hypothetical one.</p>
        <footer><button type="button" onClick={props.onClose}>Open the two files first</button><button type="button">Merge and close the task</button></footer>
      </section>
    </div>
  </Show>
}
