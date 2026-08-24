import { For, Show } from 'solid-js'
import { Icon } from '../../ui/Icon'
import {
  ARROW_MARKERS,
  BUDGET_NOTE,
  COMPILED,
  COMPILED_NOTE,
  NO_MODEL_NOTE,
  OPERAND_NOTE,
  PRESET_NOTE,
  WAITING_NOTE,
  ZOOM,
  useCanvas,
  useExpression,
  useGuardrails,
  useOperands,
  usePalette,
  usePresets,
} from '../../data/workflow'

const NODE_W = 168
const NODE_H = 46

/**
 * Layout and inspector on fixture data. Real graph editing, compile and the live
 * overlay are `workflow-canvas` at M4, gated by Spike 3.
 */
export function WorkflowView() {
  const canvas = useCanvas()
  const byId = new Map(canvas.nodes.map((n) => [n.id, n]))
  const centre = (id: string) => {
    const n = byId.get(id)!
    return { x: n.x + NODE_W / 2, y: n.y + NODE_H / 2 }
  }

  return (
    <div class="wf" data-testid="workflow">
      <aside class="wf-palette" data-testid="wf-palette">
        <span class="wf-section">Nodes</span>
        <For each={usePalette()}>
          {(node) => (
            <div
              class={['wf-chip', node.tone !== 'default' ? `wf-chip-${node.tone}` : '']
                .filter(Boolean)
                .join(' ')}
              data-testid={`wf-chip-${node.kind}`}
              data-tone={node.tone}
              draggable="true"
            >
              <Icon name="dots-six-vertical" size={10} style={{ color: 'var(--text-muted)' }} />
              <Icon name={node.icon} size={11} />
              {node.label}
              <Show when={node.required}>
                <span class="wf-chip-req" data-testid={`wf-chip-req-${node.kind}`}>
                  req
                </span>
              </Show>
            </div>
          )}
        </For>

        <span class="wf-section" data-testid="wf-presets-title">
          Presets
        </span>
        <For each={usePresets()}>
          {(preset) => (
            <div class="wf-preset" data-testid={`wf-preset-${preset.name.replace(/\W+/g, '-')}`}>
              {preset.name}
              <div class="wf-preset-note">{preset.note}</div>
            </div>
          )}
        </For>
        <span class="wf-preset-note" data-testid="wf-preset-note">
          {PRESET_NOTE}
        </span>
      </aside>

      <div class="wf-canvas" data-testid="wf-canvas">
        <svg class="wf-edges" data-testid="wf-edges">
          <defs>
            <For each={ARROW_MARKERS}>
              {(marker) => (
                <marker
                  id={marker}
                  data-testid={`wf-marker-${marker}`}
                  viewBox="0 0 8 8"
                  refX="7"
                  refY="4"
                  markerWidth="6"
                  markerHeight="6"
                  orient="auto"
                >
                  <path d="M0,0 L8,4 L0,8 z" fill="currentColor" />
                </marker>
              )}
            </For>
          </defs>
          <For each={canvas.edges}>
            {(edge) => {
              const a = centre(edge.from)
              const b = centre(edge.to)
              return (
                <line
                  class="graph-edge"
                  data-testid={`wf-edge-${edge.from}-${edge.to}`}
                  data-dashed={edge.dashed ? 'true' : undefined}
                  x1={a.x}
                  y1={a.y}
                  x2={b.x}
                  y2={b.y}
                  stroke-dasharray={edge.dashed ? '4 3' : undefined}
                  marker-end={`url(#${edge.dashed ? 'arrow-loop' : 'arrow-default'})`}
                />
              )
            }}
          </For>
        </svg>

        <div
          class="wf-loop-group"
          data-testid="wf-loop-group"
          style={{
            left: `${canvas.loop.x}px`,
            top: `${canvas.loop.y}px`,
            width: `${canvas.loop.width}px`,
            height: `${canvas.loop.height}px`,
          }}
        >
          <span class="wf-loop-label">{canvas.loop.label}</span>
        </div>

        <For each={canvas.nodes}>
          {(node) => (
            <div
              class={['wf-node', node.tone !== 'default' ? `wf-node-${node.tone}` : '']
                .filter(Boolean)
                .join(' ')}
              data-testid={`wf-node-${node.id}`}
              data-tone={node.tone}
              style={{ left: `${node.x}px`, top: `${node.y}px` }}
            >
              <div class="wf-node-strip" data-testid={`wf-node-strip-${node.id}`}>
                <Icon name="cube" size={7} style={{ color: 'var(--text-muted)' }} />
                <span class="wf-node-kind">{node.kind}</span>
                <span class="wf-node-state" data-testid={`wf-node-state-${node.id}`}>
                  {node.state}
                </span>
              </div>
              <div class="wf-node-label">{node.label}</div>
            </div>
          )}
        </For>

        <For each={canvas.edges.filter((e) => e.label)}>
          {(edge) => {
            const a = centre(edge.from)
            const b = centre(edge.to)
            return (
              <span
                class="wf-edge-label"
                data-testid={`wf-edge-label-${edge.from}-${edge.to}`}
                style={{ left: `${(a.x + b.x) / 2 - 20}px`, top: `${(a.y + b.y) / 2 - 8}px` }}
              >
                {edge.label}
              </span>
            )
          }}
        </For>

        <div class="wf-canvas-foot">
          <span class="wf-zoom" data-testid="wf-zoom">
            {ZOOM}
          </span>
          <span class="wf-canvas-note" data-testid="wf-no-model-note">
            {NO_MODEL_NOTE}
          </span>
        </div>
      </div>

      <aside class="wf-inspector" data-testid="wf-inspector">
        <div class="wf-inspector-body">
          <span class="wf-section" data-testid="wf-inspector-title">
            Condition · verify.passed
          </span>

          <For each={useExpression()}>
            {(clause, i) => (
              <>
                <Show when={i() > 0}>
                  <span class="clause-joiner" data-testid="clause-joiner">
                    and
                  </span>
                </Show>
                <div class="clause" data-testid={`clause-${i()}`}>
                  <span class="clause-field">{clause.operand}</span>
                  <span class="clause-field">{clause.operator}</span>
                  <span class="clause-field">{clause.value}</span>
                </div>
              </>
            )}
          </For>
          <button type="button" class="clause-add" data-testid="clause-add">
            + add clause
          </button>

          <div class="compiled" data-testid="wf-compiled">
            <div class="compiled-expr" data-testid="wf-compiled-expr">
              {COMPILED}
            </div>
            <div class="compiled-note" data-testid="wf-compiled-note">
              {COMPILED_NOTE}
            </div>
          </div>

          <span class="wf-section" data-testid="wf-operands-title">
            Operands — every one is a column
          </span>
          <div class="operands" data-testid="wf-operands">
            <For each={useOperands()}>
              {(operand) => (
                <span class="operand" data-testid={`operand-${operand.replace(/\W+/g, '-')}`}>
                  {operand}
                </span>
              )}
            </For>
          </div>
          <span class="wf-preset-note" data-testid="wf-operand-note">
            {OPERAND_NOTE}
          </span>

          <span class="wf-section" data-testid="wf-guardrails-title">
            Guardrails
          </span>
          <For each={useGuardrails()}>
            {(guardrail) => (
              <div class="guardrail-row" data-testid={`guardrail-${guardrail.key}`}>
                {guardrail.label}
                <Show
                  when={guardrail.kind === 'stepper'}
                  fallback={
                    <Show
                      when={guardrail.kind === 'toggle'}
                      fallback={
                        <span class="guardrail-value" data-testid={`guardrail-value-${guardrail.key}`}>
                          {guardrail.value}
                        </span>
                      }
                    >
                      <span
                        class="toggle"
                        data-testid={`guardrail-toggle-${guardrail.key}`}
                        data-on={guardrail.value === 'on' ? 'true' : 'false'}
                      />
                    </Show>
                  }
                >
                  <span class="stepper" data-testid={`guardrail-stepper-${guardrail.key}`}>
                    <button type="button" aria-label={`Decrease ${guardrail.label}`}>
                      −
                    </button>
                    <span class="guardrail-value" data-testid={`guardrail-value-${guardrail.key}`}>
                      {guardrail.value}
                    </span>
                    <button type="button" aria-label={`Increase ${guardrail.label}`}>
                      +
                    </button>
                  </span>
                </Show>
              </div>
            )}
          </For>
          <span class="wf-preset-note" data-testid="wf-budget-note">
            {BUDGET_NOTE}
          </span>
          <span class="wf-preset-note" data-testid="wf-waiting-note">
            {WAITING_NOTE}
          </span>
        </div>

        <footer class="wf-inspector-foot" data-testid="wf-inspector-foot">
          <span data-testid="wf-autosave">saved 2s ago</span>
        </footer>
      </aside>
    </div>
  )
}

/** Default export so the view can be code-split at the route boundary. */
export default WorkflowView
