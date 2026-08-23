import { For, type JSX } from 'solid-js'
import { Button } from '../../ui/Button'
import { Tag } from '../../ui/Tag'

const resident = [
  ['base-context', '14%', '1.2k', 'cached'],
  ['rules in scope', '9%', '0.8k', 'cached'],
  ['skills loaded', '21%', '1.8k', 'cached'],
  ['the live plan', '7%', '0.6k', 're-read'],
  ['recalled facts', '11%', '0.9k', '4 from long-term'],
  ['tool results', '76%', '31.4k', 'volatile'],
  ['assistant turns', '11%', '4.5k', 'volatile'],
] as const

const sessions = [
  ['tapestry · builder@4', 'r-9f21 · iteration 3 · 41.2k resident'],
  ['weaver · builder@4', '102.3k · near the ceiling'],
  ['loom-db · builder@4', '88.6k resident'],
  ['loom-db · auditor@2', '9.4k · fresh context by design'],
] as const

const facts = [
  ['NOTIFY payload caps at 8000 bytes', '0.94', 'verified · recalled 31×'],
  ['Partition key must be in the primary key', '0.88', 'verified · recalled 12×'],
  ['AppKit eats the cmd-chord before JS sees it', '0.61', 'asserted · recalled 4×'],
  ['Port range is 43000–43999', '—', 'contradicted'],
  ['sqlx offline mode needs a prepared cache', '0.44', 'decaying · last recall 19d'],
  ['Verify runs in a fresh container, never the agent’s', '0.91', 'verified · recalled 22×'],
] as const

const artifacts = [
  ['diff · notify.rs', 'r-9f21 · 4m ago'],
  ['walkthrough', 'r-9f09 · 1h'],
  ['image · board pane', 'OCR available'],
  ['recording · 42s', '12 keyframes derived'],
  ['diagram · event flow', ''],
] as const

const pages = [
  'Locus architecture',
  'Clone from a local bare remote, never a mount',
  'No MCP servers, ever',
  'Byte-deterministic materialization',
  'credential broker',
  'canary token',
] as const

function Label(props: { children: string }) {
  return <div class="v2-memory-label">{props.children}</div>
}

function FactList() {
  return (
    <div class="v2-memory-list" aria-label="Memory facts">
      <For each={facts}>
        {([title, confidence, status], index) => (
          <div class="v2-memory-list-item" data-selected={index() === 0 ? 'true' : undefined}>
            <div>{title}</div>
            <small class={status === 'contradicted' ? 'v2-memory-bad' : ''}>
              <span>{confidence}</span> {status}
            </small>
          </div>
        )}
      </For>
    </div>
  )
}

function MemoryFrame(props: { testId: string; route: string; children: JSX.Element }) {
  return (
    <main class="v2-memory" data-testid={props.testId} data-v2-route={props.route}>
      {props.children}
    </main>
  )
}

/** The rebuilt, per-iteration context window fixture. */
export function MemoryShortTermFixture() {
  return (
    <MemoryFrame testId="v2-memory-short-term" route="memory-short-term">
      <aside class="v2-memory-left">
        <header class="v2-memory-pane-head">
          <h1>Short-term</h1>
          <p>The context window. Nothing here is stored — it is rebuilt from scratch every iteration.</p>
        </header>
        <div class="v2-memory-list" aria-label="Active contexts">
          <For each={sessions}>
            {([title, detail], index) => (
              <div class="v2-memory-list-item" data-selected={index() === 0 ? 'true' : undefined}>
                <div>{title}</div>
                <small class={index() === 1 ? 'v2-memory-bad' : ''}>{detail}</small>
              </div>
            )}
          </For>
        </div>
        <footer class="v2-memory-pane-foot">
          A fresh context is a feature for the auditor and a cost for the builder. Both are deliberate.
        </footer>
      </aside>

      <section class="v2-memory-main">
        <header class="v2-memory-title-row">
          <span class="v2-memory-project">#tapestry</span>
          <h2>builder@4 · iteration 3</h2>
          <span class="v2-memory-usage">41.2k / 120k</span>
        </header>
        <div class="v2-memory-scroll">
          <section>
            <Label>Resident now</Label>
            <p class="v2-memory-note">in prefix order — the order is the cache, so it never varies</p>
            <div class="v2-memory-rows">
              <For each={resident}>
                {([name, width, amount, state]) => (
                  <div class="v2-memory-row">
                    <span>{name}</span>
                    <span class="v2-memory-bar"><span style={{ width }} /></span>
                    <code>{amount}</code>
                    <small>{state}</small>
                  </div>
                )}
              </For>
            </div>
            <p class="v2-memory-copy">
              Four fifths of the window is tool output. Everything authored stays under 4k, which keeps the prefix cached.
            </p>
          </section>
          <section>
            <Label>Compacted out</Label>
            <p class="v2-memory-note">written to an artifact, replaced by one line naming it</p>
            <ul class="v2-memory-compacted">
              <li><code>web_fetch</code> agentclientprotocol.com/protocol <span>62.4kB → a-7802</span></li>
              <li><code>bash</code> cargo build — full output <span>18.1kB → a-7811</span></li>
              <li><code>read_file</code> store/mod.rs — whole file <span>9.7kB → a-7815</span></li>
            </ul>
            <p class="v2-memory-copy">Nothing is lost, only moved: short-term drops it and an artifact can fetch it again by name.</p>
          </section>
        </div>
      </section>

      <aside class="v2-memory-right">
        <section class="v2-memory-side-card">
          <Label>Prefix cache</Label>
          <strong>84%</strong> <span>read today</span>
          <p>Stable while the materialized tree is stable. A reordered extension invalidates the prefix for every run that follows.</p>
        </section>
        <section class="v2-memory-side-card">
          <Label>What survives the iteration</Label>
          <ul>
            <li>Facts written to long-term</li>
            <li>Anything put as an artifact</li>
            <li>The plan, and its checked steps</li>
            <li class="v2-memory-bad">Everything else, including the reasoning</li>
          </ul>
        </section>
        <section class="v2-memory-side-card">
          <Label>Ceiling</Label>
          <p><code>120k</code> compaction threshold</p>
          <p><code class="v2-memory-bad">102.3k</code> weaver · builder@4</p>
          <p><code>6kB</code> per-result compaction trigger</p>
        </section>
      </aside>
    </MemoryFrame>
  )
}

/** Project-scoped facts, their provenance, decay, and contradiction state. */
export function MemoryLongTermFixture() {
  return (
    <MemoryFrame testId="v2-memory-long-term" route="memory-long-term">
      <aside class="v2-memory-left">
        <header class="v2-memory-pane-head">
          <h1>Long-term <code>318</code></h1>
          <p>Facts an agent recalls across runs. Promoted on evidence, ranked by provenance, and forgotten by decay.</p>
        </header>
        <section class="v2-memory-scope"><Label>Scope</Label><span>#tapestry</span><small>never cross-project</small></section>
        <FactList />
      </aside>

      <section class="v2-memory-main">
        <header class="v2-memory-title-row v2-memory-stacked">
          <div><Tag>memory</Tag><h2>NOTIFY payload caps at 8000 bytes</h2></div>
          <p>locus://tapestry/page/mem-1184 · written by builder@4 · promoted 6h ago · recalled 31 times</p>
        </header>
        <div class="v2-memory-scroll">
          <p class="v2-memory-prose">Postgres truncates a NOTIFY payload above 8000 bytes without raising. The payload carries the row id and the listener re-reads what it names.</p>
          <section>
            <Label>Why this is trusted</Label>
            <ul class="v2-memory-evidence">
              <li>Confirmed by a passing verify — <code>payload_is_id_only</code>, run r-9f21.</li>
              <li>Cited by <a href="#memory-wiki">[[clone-not-mount]]</a> and the store spec.</li>
              <li>Recalled 31 times in 14 days, so decay has not touched it.</li>
            </ul>
          </section>
          <section>
            <Label>Confidence over time</Label>
            <p class="v2-memory-note">decay is the forgetting half; curation is the reconciling half</p>
            <div class="v2-memory-confidence" aria-label="Confidence increased from 0.38 to 0.94">
              <For each={[38, 46, 44, 58, 55, 72, 70, 88, 94, 94]}>{(height) => <span style={{ height: `${height}%` }} />}</For>
            </div>
            <p class="v2-memory-note">asserted 0.38 → verified 0.94 · the jump is the verify, not repetition</p>
          </section>
          <p class="v2-memory-callout">Editing this makes it yours, not the agent’s. The page keeps both the fact and the correction.</p>
        </div>
      </section>

      <aside class="v2-memory-right">
        <section class="v2-memory-side-card"><Label>This project’s memory</Label><strong>318</strong> facts · <strong class="v2-memory-ok">146</strong> verified · <strong>1</strong> contradicted</section>
        <section class="v2-memory-side-card">
          <Label>Contradiction</Label>
          <p>Port range disagrees with the wiki.</p>
          <p><code>43000–43999</code> — memory, verified<br /><code>44000–44999</code> — ADR-007, 6h</p>
          <Button variant="primary">Adjudicate</Button>
        </section>
        <section class="v2-memory-side-card"><Label>Decay</Label><p><code>41</code> fell below recall threshold last night</p><p><code>6</code> promoted on a passing verify</p></section>
        <section class="v2-memory-side-card"><Label>locus memory explain</Label><code>$ locus memory explain mem-1184<br />recalled 31× · 0.94<br />source: run r-9f21 verify</code></section>
      </aside>
    </MemoryFrame>
  )
}

/** Reviewable artifacts retain their source while comments steer their session. */
export function MemoryArtifactsFixture() {
  return (
    <MemoryFrame testId="v2-memory-artifacts" route="memory-artifacts">
      <aside class="v2-memory-left">
        <Label>Review artifacts</Label>
        <div class="v2-memory-list">
          <For each={artifacts}>{([title, detail], index) => <div class="v2-memory-list-item" data-selected={index() === 0 ? 'true' : undefined}><div>{title}</div><small>{detail}</small></div>}</For>
        </div>
        <Label>Reference · never in the inbox</Label>
        <div class="v2-memory-list"><div class="v2-memory-list-item">finding · ACP prior art</div><div class="v2-memory-list-item">payload · 62.4kB fetch</div></div>
      </aside>
      <section class="v2-memory-main">
        <header class="v2-memory-title-row"><Tag>diff</Tag><h2>store/notify.rs</h2><code>locus://tapestry/artifact/a-7830</code><small>one viewer per kind · three entry points</small></header>
        <pre class="v2-memory-diff">@@ -19,6 +19,7 @@ impl Store{`\n`}  pub async fn notify(&self, ch: &str, id: Uuid){`\n`}<ins>+ // NOTIFY carries an id only — cap is 8000 bytes</ins>{`\n`}<del>- .bind(serde_json::to_string(&row)?)</del>{`\n`}<ins>+ .bind(id.to_string())</ins>{`\n`}  .execute(&self.pool).await?;</pre>
      </section>
      <aside class="v2-memory-right">
        <Label>Comments steer the agent</Label>
        <div class="v2-memory-comment"><small>you · line 22</small>The listener needs to handle a row deleted between NOTIFY and re-read.</div>
        <div class="v2-memory-comment" data-live="true"><small>builder@4 · replying</small>Added <code>Ok(None)</code> on a missing row and a test that deletes before the listener wakes.</div>
        <footer class="v2-memory-comment-form"><textarea placeholder="Comment on line 62…" /><Button variant="primary">Send to session</Button><Button>Resolve</Button></footer>
      </aside>
    </MemoryFrame>
  )
}

/** Curated wiki prose remains separate from agent recall. */
export function MemoryWikiFixture() {
  return (
    <MemoryFrame testId="v2-memory-wiki" route="memory-wiki">
      <aside class="v2-memory-left">
        <header class="v2-memory-pane-head"><h1>All <code>153</code></h1><p>Curated project knowledge derived from sources, then reviewed by people.</p><Button variant="primary" block>Ingest a document</Button></header>
        <div class="v2-memory-kinds"><span>All 153</span><span>Decisions 14</span><span>Concepts 31</span><span>Entities 42</span></div>
        <div class="v2-memory-list"><For each={pages}>{(page, index) => <div class="v2-memory-list-item" data-selected={index() === 1 ? 'true' : undefined}>{page}</div>}</For></div>
        <footer class="v2-memory-pane-foot">Derived, then curated — a path or a URL, never a blank page.</footer>
      </aside>
      <section class="v2-memory-main">
        <header class="v2-memory-title-row v2-memory-stacked"><div><Tag>decision</Tag><h2>Clone from a local bare remote, never a mount</h2></div><p>locus://tapestry/page/clone-not-mount · rev 7 · 3 assertions · 2 sources</p></header>
        <article class="v2-memory-scroll v2-memory-article">
          <p>Every project has a bare local remote on the host at <code>/var/lib/locus/repos/&lt;project&gt;.git</code>. An agent container clones from it into its own filesystem, commits, and pushes a branch back. The workspace is never bind-mounted.</p>
          <p>Isolation is real because the working copy was never there to escape into. Reviewing the work stays ordinary git, so Locus stays out of your editor and merge tool.</p>
          <p>Cost, stated: clones take disk and time, mitigated by <code>git clone --reference</code> against a shared object store.</p>
          <Label>Links out</Label><p class="v2-memory-links">[[bare local remote]] [[locus-agent container]] [[git invariant: never main]]</p>
          <Label>Provenance</Label><p>PLAN.md — “The git model — a local remote, not shared worktrees”, ingested 4d ago</p><p>PR #491 body — repo manager, merge-back path</p>
        </article>
      </section>
      <aside class="v2-memory-right">
        <section class="v2-memory-side-card"><Label>Graph</Label><div class="v2-memory-graph" aria-label="Wiki links graph"><span>bare remote</span><strong>clone-not-mount</strong><span>locusd</span><span>determinism</span></div><p>Pages are nodes, wikilinks are edges — the canvas renderer, repointed.</p></section>
        <section class="v2-memory-side-card"><Label>Contradictions</Label><p>Port range disagrees across two sources.</p><p><code>43000–43999</code> — PLAN.md<br /><code>44000–44999</code> — ADR-007</p><Button variant="primary">Adjudicate</Button> <Button>Board card</Button></section>
        <section class="v2-memory-side-card"><Label>locus wiki lint</Label><p>2 orphan pages — credential broker, canary token</p><p>1 broken link — <code>[[egress tiers]]</code></p><p class="v2-memory-ok">153 pages otherwise clean</p><p>The wiki is curated prose a human reads. Memory is what an agent recalls — they share pgvector and nothing else.</p></section>
      </aside>
    </MemoryFrame>
  )
}
