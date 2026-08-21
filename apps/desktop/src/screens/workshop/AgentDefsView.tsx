import { For, createSignal } from 'solid-js'
import { Button } from '../../ui/Button'
import { Icon } from '../../ui/Icon'
import { Resizable } from '../../panes/Resizable'
import {
  DIFF_LABEL,
  MATERIALIZE_TARGET,
  PROVENANCE,
  SAVE_LABEL,
  SIDEBAR_NOTE,
  useAgentDefs,
  useAgentMaterialization,
  useDefaultAgentDef,
  useFrontmatter,
  useProse,
} from '../../data/agent-defs'
import type { View } from '../../nav'

export interface AgentDefsViewProps {
  onNavigate: (view: View) => void
}

/**
 * A drill-down of Extensions, not a tab. Markdown plus a tool list — there is no
 * canvas and no compile step, because an agent definition is prose the model
 * reads rather than a program something runs.
 */
export function AgentDefsView(props: AgentDefsViewProps) {
  const [selected, setSelected] = createSignal(useDefaultAgentDef())
  const materialization = useAgentMaterialization()

  return (
    <div class="agentdefs" data-testid="agentdefs">
      <Resizable width={196} min={160} max={320} side="right" class="agentdefs-side" testId="agentdefs-side">
        <div class="agentdefs-side-body">
          <button
            type="button"
            class="agentdefs-back"
            data-testid="agentdefs-back"
            onClick={() => props.onNavigate('extensions')}
          >
            <Icon name="arrow-left" size={11} />
            Extensions
          </button>

          <div class="wf-section" data-testid="agentdefs-list-title">
            Agent definitions
          </div>
          <For each={useAgentDefs()}>
            {(def) => (
              <button
                type="button"
                class="agentdefs-row"
                data-testid={`agentdef-${def.name}`}
                aria-selected={selected() === def.name ? 'true' : 'false'}
                onClick={() => setSelected(def.name)}
              >
                {def.name}
                <span class="agentdefs-version" data-testid={`agentdef-version-${def.name}`}>
                  v{def.version}
                </span>
              </button>
            )}
          </For>
        </div>
        <footer class="agentdefs-side-foot" data-testid="agentdefs-side-foot">
          {SIDEBAR_NOTE}
        </footer>
      </Resizable>

      <section class="agentdefs-main" data-testid="agentdefs-main">
        <header class="agentdefs-head" data-testid="agentdefs-head">
          <span class="agentdefs-file" data-testid="agentdefs-file">
            {selected()}.md
          </span>
          <span class="agentdefs-provenance" data-testid="agentdefs-provenance">
            {PROVENANCE}
          </span>
          <div class="ws-actions">
            <Button variant="secondary" data-testid="agentdefs-diff">
              {DIFF_LABEL}
            </Button>
            <Button variant="primary" data-testid="agentdefs-save">
              {SAVE_LABEL}
            </Button>
          </div>
        </header>

        <div class="agentdefs-body">
          <div class="frontmatter" data-testid="agentdefs-frontmatter">
            <div>---</div>
            <For each={useFrontmatter()}>
              {(line) => (
                <div data-testid={`frontmatter-${line.key}`}>
                  <span class="frontmatter-key">{line.key}</span>: {line.value}
                </div>
              )}
            </For>
            <div>---</div>
          </div>

          <div class="agentdefs-prose" data-testid="agentdefs-prose">
            <For each={useProse()}>{(para) => <p style={{ margin: 0 }}>{para}</p>}</For>
          </div>
        </div>

        <footer class="agentdefs-foot" data-testid="agentdefs-foot">
          Materialized to <span class="mono">{MATERIALIZE_TARGET}</span> for{' '}
          {materialization.harnesses} harnesses, {materialization.downgraded} downgraded.
        </footer>
      </section>
    </div>
  )
}

/** Default export so the view can be code-split at the route boundary. */
export default AgentDefsView
