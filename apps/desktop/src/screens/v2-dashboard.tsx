import { For } from 'solid-js'
import {
  V2_DASHBOARD_COUNTERS,
  V2_INBOX_ITEMS,
  V2_MODEL_SCORECARD,
  V2_RUNNING_PROJECTS,
  V2_TOKEN_DAYS,
} from '../fixtures/v2-dashboard'

const TOKEN_DAY_LABELS = ['08', '09', '10', '11', '12', '13', '14', '15', '16', '17', '18', '19', '20', '21']
const MODEL_COLORS = ['data-1', 'data-2', 'data-3', 'data-hi']

/** Global interruption fixture: responses live here; work opens at its own surface. */
export function V2InboxView() {
  return (
    <div class="v2-inbox" data-testid="v2-inbox" data-v2-route="inbox">
      <aside class="v2-inbox-list">
        <div class="v2-inbox-tabs" data-testid="v2-inbox-tabs">
          <button aria-current="page" type="button" data-inbox-group="action-required">To do <span>3</span></button>
          <button type="button" data-inbox-group="completed">Completed <span>3</span></button>
        </div>
        <div class="v2-inbox-budget" data-testid="v2-inbox-budget">
          <span class="v2-budget-meter" aria-hidden="true"><i /><i /><i /><i /><i /><i /></span>
          <span>3 / 6 per hour</span>
          <span>under budget</span>
          <button type="button"># all projects</button>
        </div>
        <div class="v2-inbox-items" data-testid="v2-inbox-items">
          <For each={V2_INBOX_ITEMS}>
            {(item: (typeof V2_INBOX_ITEMS)[number], index: () => number) => (
              <button
                class={`v2-inbox-item v2-inbox-item-${item.kind}`}
                data-inbox-item
                data-selected={index() === 0 ? '' : undefined}
                type="button"
              >
                <span class="v2-inbox-item-head"><strong>{item.title}</strong><span>{item.age}</span></span>
                <span>{item.detail}</span>
              </button>
            )}
          </For>
        </div>
        <p class="v2-inbox-note">Every item type documents the response it wants. Notifications go to Activity.</p>
      </aside>

      <section class="v2-inbox-detail">
        <header>
          <div><span class="v2-inbox-kind">plan</span><h1>Keyframe extraction for recordings</h1></div>
          <p data-inbox-detail="evidence">locus://texere/artifact/a-7741 · builder@3 · role impl · Gate: human</p>
        </header>
        <div class="v2-inbox-detail-body">
          <section>
            <h2>Plan</h2>
            <ol>
              <li>Store the WebM under <code>/var/lib/locus/artifacts/&lt;project&gt;/&lt;run&gt;/</code>; the row carries its path, type, and sha256.</li>
              <li>Derive keyframes on demand with <code>ffmpeg</code>; never hand the clip to a model.</li>
              <li>Cache derivations beside the original; the stored copy is never overwritten.</li>
              <li>Prune with the run at 30 days unless it is linked to a PR or a task in Done.</li>
            </ol>
          </section>
          <p class="v2-inbox-callout">Retention differs from the text artifacts this task also produces — it is the only irreversible step in the plan.</p>
          <label class="v2-inbox-comment">Comment steers the agent that made it<textarea placeholder="Cap frames at 8, and keep the clip when the run is linked to a PR." /></label>
          <div class="v2-inbox-actions"><button type="button">Approve &amp; release the loop</button><button type="button">Send back with comment</button><span>Resolves here — the work opens where the work lives.</span></div>
          <div class="v2-inbox-explanation">
            <p data-inbox-detail="why"><strong>Why this is here</strong>The Gate node in workflow <code>wf-12</code> is human for irreversible steps. The agent has written nothing and is blocked, not idle.</p>
            <p data-inbox-detail="cost"><strong>Cost of waiting</strong>One loop held for 4m.<br />No tokens burn while blocked.</p>
          </div>
        </div>
      </section>
    </div>
  )
}

/** Global aggregate fixture; it deliberately ignores the selected-project scope. */
export function V2DashboardView() {
  return (
    <div class="v2-dashboard" data-testid="v2-dashboard">
      <header class="v2-dashboard-head"><h1>All projects</h1><p>5 projects · the one surface that ignores the project selector</p><div data-testid="v2-dashboard-range"><button type="button">7d</button><button aria-current="page" type="button">14d</button><button type="button">30d</button></div></header>
      <div class="v2-dashboard-summary">
        <section class="v2-panel"><h2>Spend today</h2><p class="v2-summary-value">$68.40 <span>of $240 across all projects</span></p><div class="v2-spend-bar"><i class="v2-magnitude-fill v2-data-hi" /><i class="v2-magnitude-fill v2-data-3" /><i class="v2-magnitude-fill v2-data-2" /><i class="v2-magnitude-fill v2-data-1" /></div><p class="v2-legend">opus-4.6 $40.30 · gpt-5.2-pro $12.80 · gemini-3-ultra $7.40 · composer-2 $7.90</p></section>
        <section class="v2-panel" data-testid="v2-dashboard-running"><h2>Running now</h2><p class="v2-summary-value v2-running-value">8 <span>runs across 4 projects · 1 waiting on you</span></p><div class="v2-running-projects"><For each={V2_RUNNING_PROJECTS}>{(project: (typeof V2_RUNNING_PROJECTS)[number]) => <p><i data-state={project.state} /><code>#{project.project}</code><span>{project.detail}</span></p>}</For></div></section>
      </div>
      <section class="v2-panel v2-token-panel"><h2>Tokens per day, by model <span>14 days · 41.7M total · weekends dip because the schedules do</span></h2><div class="v2-token-chart" data-testid="v2-token-chart"><div class="v2-token-axis"><span>5M</span><span>2.5M</span><span>0</span></div><div class="v2-token-days"><For each={V2_TOKEN_DAYS}>{(day: (typeof V2_TOKEN_DAYS)[number], index: () => number) => <div data-token-day><div class="v2-token-stack"><For each={day}>{(height: number, segment: () => number) => <i class={`v2-magnitude-fill v2-data-${MODEL_COLORS[segment()]}`} style={{ height: `${height}px` }} />}</For></div><span>{TOKEN_DAY_LABELS[index()]}</span></div>}</For></div></div></section>
      <div class="v2-dashboard-details">
        <section class="v2-panel"><h2>Average tokens per prompt</h2><p class="v2-panel-note">Context sent per turn, not per run. Check a model's context against cache read.</p><div class="v2-bar-list"><ModelBar model="opus-4.6" value="8.4k" width="75%" color="hi" note="41% of it cached prefix" /><ModelBar model="gpt-5.2-pro" value="11.2k" width="100%" color="3" note="largest context per turn, and the most expensive" /><ModelBar model="gemini-3-ultra" value="6.1k" width="54%" color="2" note="smallest turns — runs more of them" /><ModelBar model="composer-2" value="4.3k" width="38%" color="1" note="edit-scoped by design" /></div></section>
        <section class="v2-panel"><h2>Spend by project</h2><p class="v2-panel-note">14 days. This screen stays global when a project tag is selected.</p><div class="v2-bar-list"><ModelBar model="#tapestry" value="$412" width="100%" color="3" note="287 tasks landed · $1.44 each" /><ModelBar model="#loom-db" value="$232" width="56%" color="3" note="in elicitation — no tasks landed yet" /><ModelBar model="#weaver" value="$141" width="34%" color="3" note="verify pass fell to 44%" /><ModelBar model="#texere" value="$77" width="19%" color="3" note="1 run, waiting on a gate" /></div></section>
      </div>
      <section class="v2-panel v2-scorecard" data-testid="v2-model-scorecard"><h2>Model scorecard <span>14 days, all projects · cost per landed task is the outcome</span></h2><table><thead><tr><th>Model</th><th>Runs</th><th>Cache read</th><th>Verify pass</th><th>Iters / task</th><th>$ / landed task</th></tr></thead><tbody><For each={V2_MODEL_SCORECARD}>{(row: (typeof V2_MODEL_SCORECARD)[number]) => <tr><th>{row.model}</th><td>{row.runs}</td><td>{row.cache}</td><td class={row.good ? 'v2-good' : 'v2-bad'}>{row.verify}</td><td>{row.iterations}</td><td>{row.cost}</td></tr>}</For></tbody></table><p class="v2-panel-note">composer-2 is the cheapest per landed task and the worst per attempt. Cheap and wasteful are not the same axis.</p></section>
      <div class="v2-dashboard-counters" data-testid="v2-dashboard-counters"><For each={V2_DASHBOARD_COUNTERS}>{(counter: (typeof V2_DASHBOARD_COUNTERS)[number]) => <section class="v2-panel" data-dashboard-counter><h2>{counter.label}</h2><strong>{counter.value}</strong><p>{counter.note}</p></section>}</For></div>
    </div>
  )
}

function ModelBar(props: { model: string; value: string; width: string; color: '1' | '2' | '3' | 'hi'; note: string }) {
  return <div class="v2-model-bar"><code>{props.model}</code><span><i class={`v2-magnitude-fill v2-data-${props.color}`} style={{ width: props.width }} /></span><strong>{props.value}</strong><p>{props.note}</p></div>
}
