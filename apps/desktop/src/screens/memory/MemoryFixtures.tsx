import { For, type JSX } from "solid-js";
import { Button } from "../../ui/Button";
import { Tag } from "../../ui/Tag";
import { ARTIFACT_LOCATOR } from "../../data/artifacts";
import {
  COMPACTED_CONTEXT,
  CURATION_COPY,
  LONG_TERM_FACTS,
  RESIDENT_LAYERS,
  SHORT_TERM_COPY,
  WIKI_CONTRADICTION_COPY,
  WIKI_GRAPH_COPY,
  WIKI_INGEST_COPY,
  WIKI_KIND_CHIPS,
} from "../../data/knowledge";
import { MailView } from "../mail/MailView";

const sessions = [
  ["tapestry · builder@4", "r-9f21 · iteration 3 · 41.2k resident"],
  ["weaver · builder@4", "102.3k · near the ceiling"],
  ["loom-db · builder@4", "88.6k resident"],
  ["loom-db · auditor@2", "9.4k · fresh context by design"],
] as const;

const artifacts = [
  ["diff · notify.rs", "r-9f21 · 4m ago"],
  ["walkthrough", "r-9f09 · 1h"],
  ["image · board pane", "OCR available"],
  ["recording · 42s", "12 keyframes derived"],
  ["diagram · event flow", ""],
] as const;

const pages = [
  "Locus architecture",
  "Clone from a local bare remote, never a mount",
  "No MCP servers, ever",
  "Byte-deterministic materialization",
  "credential broker",
  "canary token",
] as const;

function Label(props: { children: string }) {
  return <div class="desktop-memory-label">{props.children}</div>;
}

function FactList() {
  return (
    <div class="desktop-memory-list" aria-label="Memory facts">
      <For each={LONG_TERM_FACTS}>
        {(fact, index) => (
          <div
            class="desktop-memory-list-item"
            data-selected={index() === 0 ? "true" : undefined}
            data-confidence={fact.confidence}
          >
            <div>{fact.title}</div>
            <small
              class={
                fact.confidence === "contradicted" ? "desktop-memory-bad" : ""
              }
            >
              {fact.score === null
                ? `— ${fact.confidence} · no score`
                : `${fact.score.toFixed(2)} ${fact.confidence} · ${fact.recall}`}
            </small>
          </div>
        )}
      </For>
    </div>
  );
}

function MemoryFrame(props: {
  testId: string;
  route: string;
  contextLayers?: string;
  compaction?: string;
  contextBudget?: string;
  factFixture?: string;
  factState?: string;
  artifactGroups?: string;
  artifactPreview?: string;
  wikiFixture?: string;
  wikiViewer?: string;
  children: JSX.Element;
}) {
  return (
    <main
      class="desktop-memory"
      data-testid={props.testId}
      data-desktop-route={props.route}
      data-context-layers={props.contextLayers}
      data-compaction={props.compaction}
      data-context-budget={props.contextBudget}
      data-fact-fixture={props.factFixture}
      data-fact-state={props.factState}
      data-artifact-groups={props.artifactGroups}
      data-artifact-preview={props.artifactPreview}
      data-wiki-fixture={props.wikiFixture}
      data-wiki-viewer={props.wikiViewer}
    >
      {props.children}
    </main>
  );
}

/** The rebuilt, per-iteration context window fixture. */
export function MemoryShortTermFixture() {
  return (
    <MemoryFrame
      testId="desktop-memory-short-term"
      route="short"
      contextLayers="resident"
      compaction="artifact-handles"
      contextBudget="120k"
    >
      <aside class="desktop-memory-left">
        <header class="desktop-memory-pane-head">
          <h1>Short-term</h1>
          <p>{SHORT_TERM_COPY.intro}</p>
        </header>
        <div class="desktop-memory-list" aria-label="Active contexts">
          <For each={sessions}>
            {([title, detail], index) => (
              <div
                class="desktop-memory-list-item"
                data-selected={index() === 0 ? "true" : undefined}
              >
                <div>{title}</div>
                <small class={index() === 1 ? "desktop-memory-bad" : ""}>
                  {detail}
                </small>
              </div>
            )}
          </For>
        </div>
        <footer class="desktop-memory-pane-foot">
          A fresh context is a feature for the auditor and a cost for the
          builder. Both are deliberate.
        </footer>
      </aside>

      <section class="desktop-memory-main">
        <header class="desktop-memory-title-row">
          <span class="desktop-memory-project">#tapestry</span>
          <h2>builder@4 · iteration 3</h2>
          <span class="desktop-memory-usage">41.2k / 120k</span>
        </header>
        <div class="desktop-memory-scroll">
          <section>
            <Label>Resident now</Label>
            <p class="desktop-memory-note">{SHORT_TERM_COPY.residentNote}</p>
            <div class="desktop-memory-rows">
              <For each={RESIDENT_LAYERS}>
                {(layer) => (
                  <div class="desktop-memory-row" data-resident-tag={layer.tag}>
                    <span>{layer.name}</span>
                    <span class="desktop-memory-bar">
                      <span style={{ width: layer.percent }} />
                    </span>
                    <code>{layer.size}</code>
                    <small>{layer.tag}</small>
                  </div>
                )}
              </For>
            </div>
            <p class="desktop-memory-copy">{SHORT_TERM_COPY.residentReading}</p>
          </section>
          <section>
            <Label>Compacted out</Label>
            <p class="desktop-memory-note">{SHORT_TERM_COPY.compactedNote}</p>
            <ul class="desktop-memory-compacted">
              <For each={COMPACTED_CONTEXT}>
                {(item) => (
                  <li data-artifact-id={item.artifactId}>
                    <code>{item.tool}</code> {item.description}{" "}
                    <span>
                      {item.size} → {item.artifactId}
                    </span>
                  </li>
                )}
              </For>
            </ul>
            <p class="desktop-memory-copy">
              {SHORT_TERM_COPY.compactedReading}
            </p>
          </section>
        </div>
      </section>

      <aside class="desktop-memory-right">
        <section class="desktop-memory-side-card">
          <Label>Prefix cache</Label>
          <strong>84%</strong> <span>read today</span>
          <p>{SHORT_TERM_COPY.cacheNote}</p>
        </section>
        <section class="desktop-memory-side-card">
          <Label>What survives the iteration</Label>
          <ul>
            <li>Facts written to long-term</li>
            <li>Anything put as an artifact</li>
            <li>The plan, and its checked steps</li>
            <li class="desktop-memory-bad">
              {SHORT_TERM_COPY.survivesReasoning}
            </li>
          </ul>
        </section>
        <section class="desktop-memory-side-card">
          <Label>Ceiling</Label>
          <p>
            <code>120k</code> compaction threshold
          </p>
          <p>
            <code class="desktop-memory-bad">102.3k</code> weaver · builder@4
          </p>
          <p>
            <code>6kB</code> per-result compaction trigger
          </p>
        </section>
      </aside>
    </MemoryFrame>
  );
}

/** Project-scoped facts, their provenance, decay, and contradiction state. */
export function MemoryLongTermFixture() {
  return (
    <MemoryFrame
      testId="desktop-memory-long-term"
      route="memory"
      factFixture="long-term"
      factState="provenance-confidence-decay-contradiction"
    >
      <aside class="desktop-memory-left">
        <header class="desktop-memory-pane-head">
          <h1>
            Long-term <code>318</code>
          </h1>
          <p>
            Facts an agent recalls across runs. Promoted on evidence, ranked by
            provenance, and forgotten by decay.
          </p>
        </header>
        <section class="desktop-memory-scope">
          <Label>Scope</Label>
          <span>#tapestry</span>
          <small>never cross-project</small>
        </section>
        <FactList />
      </aside>

      <section class="desktop-memory-main">
        <header class="desktop-memory-title-row desktop-memory-stacked">
          <div>
            <Tag>memory</Tag>
            <h2>NOTIFY payload caps at 8000 bytes</h2>
          </div>
          <p>
            locus://tapestry/page/mem-1184 · written by builder@4 · promoted 6h
            ago · recalled 31 times
          </p>
        </header>
        <div class="desktop-memory-scroll">
          <p class="desktop-memory-prose">
            Postgres truncates a NOTIFY payload above 8000 bytes without
            raising. The payload carries the row id and the listener re-reads
            what it names.
          </p>
          <section>
            <Label>Why this is trusted</Label>
            <ul class="desktop-memory-evidence">
              <li>
                Confirmed by a passing verify — <code>payload_is_id_only</code>,
                run r-9f21.
              </li>
              <li>
                Cited by <a href="#memory-wiki">[[clone-not-mount]]</a> and the
                store spec.
              </li>
              <li>
                Recalled 31 times in 14 days, so decay has not touched it.
              </li>
            </ul>
          </section>
          <section>
            <Label>Confidence over time</Label>
            <p class="desktop-memory-note">
              decay is the forgetting half; curation is the reconciling half
            </p>
            <div
              class="desktop-memory-confidence"
              aria-label="Confidence increased from 0.38 to 0.94"
            >
              <For each={[38, 46, 44, 58, 55, 72, 70, 88, 94, 94]}>
                {(height) => <span style={{ height: `${height}%` }} />}
              </For>
            </div>
            <p class="desktop-memory-note">
              asserted 0.38 → verified 0.94 · the jump is the verify, not
              repetition
            </p>
          </section>
          <section class="desktop-memory-callout" data-testid="memory-curation">
            <p>{CURATION_COPY}</p>
            <div class="desktop-memory-revisions" aria-label="Fact revisions">
              <span data-revision="1">
                Written fact · revision 1 · unchanged
              </span>
              <span data-revision="2">
                Your correction · revision 2 · recalled now
              </span>
            </div>
            <Button data-testid="edit-recalled-fact">Edit recalled fact</Button>
          </section>
        </div>
      </section>

      <aside class="desktop-memory-right">
        <section class="desktop-memory-side-card">
          <Label>This project’s memory</Label>
          <strong>318</strong> facts ·{" "}
          <strong class="desktop-memory-ok">146</strong> verified ·{" "}
          <strong>1</strong> contradicted
        </section>
        <section class="desktop-memory-side-card">
          <Label>Contradiction</Label>
          <p>Port range disagrees with the wiki.</p>
          <p>
            <code>43000–43999</code> — memory, verified
            <br />
            <code>44000–44999</code> — ADR-007, 6h
          </p>
          <Button variant="primary">Adjudicate</Button>
        </section>
        <section class="desktop-memory-side-card">
          <Label>Decay</Label>
          <p>
            <code>41</code> fell below recall threshold last night
          </p>
          <p>
            <code>6</code> promoted on a passing verify
          </p>
        </section>
        <section class="desktop-memory-side-card">
          <Label>locus memory explain</Label>
          <code>
            $ locus memory explain mem-1184
            <br />
            recalled 31× · 0.94
            <br />
            source: run r-9f21 verify
          </code>
        </section>
      </aside>
    </MemoryFrame>
  );
}

/** Reviewable artifacts retain their source while comments steer their session. */
export function MemoryArtifactsFixture() {
  return (
    <MemoryFrame
      testId="desktop-memory-artifacts"
      route="artifact"
      artifactGroups="review-reference"
      artifactPreview="comments-review"
    >
      <aside class="desktop-memory-left">
        <Label>Review artifacts</Label>
        <div class="desktop-memory-list">
          <For each={artifacts}>
            {([title, detail], index) => (
              <div
                class="desktop-memory-list-item"
                data-selected={index() === 0 ? "true" : undefined}
              >
                <div>{title}</div>
                <small>{detail}</small>
              </div>
            )}
          </For>
        </div>
        <Label>Reference · never in the inbox</Label>
        <div class="desktop-memory-list">
          <div class="desktop-memory-list-item">finding · ACP prior art</div>
          <div class="desktop-memory-list-item">payload · 62.4kB fetch</div>
        </div>
      </aside>
      <section class="desktop-memory-main">
        <header class="desktop-memory-title-row">
          <Tag>diff</Tag>
          <h2>store/notify.rs</h2>
          <code>{ARTIFACT_LOCATOR}</code>
          <small>one viewer per kind · three entry points</small>
        </header>
        <pre class="desktop-memory-diff">
          @@ -19,6 +19,7 @@ impl Store{`\n`} pub async fn notify(&self, ch:
          &str, id: Uuid){`\n`}
          <ins>+ // NOTIFY carries an id only — cap is 8000 bytes</ins>
          {`\n`}
          <del>- .bind(serde_json::to_string(&row)?)</del>
          {`\n`}
          <ins>+ .bind(id.to_string())</ins>
          {`\n`} .execute(&self.pool).await?;
        </pre>
        <section class="desktop-memory-media-viewers" data-testid="artifacts-media-viewers">
          <figure data-media-kind="image">
            <img alt="Derived screenshot preview" src="data:image/webp;base64,UklGRg==" />
            <figcaption>image · original preserved · derived preview</figcaption>
          </figure>
          <figure data-media-kind="recording">
            <video controls aria-label="Recording keyframes" />
            <figcaption>recording · keyframes for context · clip stays human-only</figcaption>
          </figure>
        </section>
      </section>
      <aside class="desktop-memory-right">
        <Label>Comments steer the agent</Label>
        <div class="desktop-memory-comment">
          <small>you · line 22</small>The listener needs to handle a row deleted
          between NOTIFY and re-read.
        </div>
        <div class="desktop-memory-comment" data-live="true">
          <small>builder@4 · replying</small>Added <code>Ok(None)</code> on a
          missing row and a test that deletes before the listener wakes.
        </div>
        <footer class="desktop-memory-comment-form">
          <textarea placeholder="Comment on line 62…" />
          <Button variant="primary">Send to session</Button>
          <Button>Resolve</Button>
        </footer>
      </aside>
    </MemoryFrame>
  );
}

/** Curated wiki prose remains separate from agent recall. */
export function MemoryMailFixture() {
  return <MailView />;
}

export function MemoryWikiFixture() {
  return (
    <MemoryFrame
      testId="desktop-memory-wiki"
      route="wiki"
      wikiFixture="typed-page"
      wikiViewer="outline-links-provenance-graph"
    >
      <aside class="desktop-memory-left">
        <header class="desktop-memory-pane-head">
          <h1>
            All <code>153</code>
          </h1>
          <p>
            Curated project knowledge derived from sources, then reviewed by
            people.
          </p>
          <Button variant="primary" block>
            Ingest a document
          </Button>
        </header>
        <div class="desktop-memory-kinds" aria-label="Wiki kinds">
          <For each={WIKI_KIND_CHIPS}>
            {(chip) => (
              <button type="button" data-kind={chip.kind}>
                {chip.label} {chip.count}
              </button>
            )}
          </For>
        </div>
        <div class="desktop-memory-list">
          <For each={pages}>
            {(page, index) => (
              <div
                class="desktop-memory-list-item"
                data-selected={index() === 1 ? "true" : undefined}
              >
                {page}
              </div>
            )}
          </For>
        </div>
        <footer class="desktop-memory-pane-foot">{WIKI_INGEST_COPY}</footer>
      </aside>
      <section class="desktop-memory-main">
        <header class="desktop-memory-title-row desktop-memory-stacked">
          <div>
            <Tag>decision</Tag>
            <h2>Clone from a local bare remote, never a mount</h2>
          </div>
          <p>
            locus://tapestry/page/clone-not-mount · rev 7 · 3 assertions · 2
            sources
          </p>
        </header>
        <article class="desktop-memory-scroll desktop-memory-article">
          <p>
            Every project has a bare local remote on the host at{" "}
            <code>/var/lib/locus/repos/&lt;project&gt;.git</code>. An agent
            container clones from it into its own filesystem, commits, and
            pushes a branch back. The workspace is never bind-mounted.
          </p>
          <p>
            Isolation is real because the working copy was never there to escape
            into. Reviewing the work stays ordinary git, so Locus stays out of
            your editor and merge tool.
          </p>
          <p>
            Cost, stated: clones take disk and time, mitigated by{" "}
            <code>git clone --reference</code> against a shared object store.
          </p>
          <Label>Links out</Label>
          <p class="desktop-memory-links">
            [[bare local remote]] [[locus-agent container]] [[git invariant:
            never main]]
          </p>
          <Label>Kind definitions</Label>
          <dl class="desktop-memory-definitions">
            <For each={WIKI_KIND_CHIPS}>
              {(chip) => (
                <>
                  <dt>{chip.label}</dt>
                  <dd>{chip.definition}</dd>
                </>
              )}
            </For>
          </dl>
          <Label>Provenance</Label>
          <p>
            PLAN.md — “The git model — a local remote, not shared worktrees”,
            ingested 4d ago
          </p>
          <p>PR #491 body — repo manager, merge-back path</p>
        </article>
      </section>
      <aside class="desktop-memory-right">
        <section class="desktop-memory-side-card">
          <Label>Graph</Label>
          <div class="desktop-memory-graph" aria-label="Wiki links graph">
            <span>bare remote</span>
            <strong>clone-not-mount</strong>
            <span>locusd</span>
            <span>determinism</span>
          </div>
          <p>{WIKI_GRAPH_COPY}</p>
        </section>
        <section
          class="desktop-memory-side-card"
          data-contradiction-timing="ingest"
        >
          <Label>Contradictions</Label>
          <p>{WIKI_CONTRADICTION_COPY}</p>
          <p>Port range disagrees across two sources.</p>
          <p>
            <code>43000–43999</code> — PLAN.md
            <br />
            <code>44000–44999</code> — ADR-007
          </p>
          <Button variant="primary">Adjudicate</Button>{" "}
          <Button>Board card</Button>
        </section>
        <section class="desktop-memory-side-card">
          <Label>locus wiki lint</Label>
          <p>2 orphan pages — credential broker, canary token</p>
          <p>
            1 broken link — <code>[[egress tiers]]</code>
          </p>
          <p class="desktop-memory-ok">153 pages otherwise clean</p>
          <p>
            The wiki is curated prose a human reads. Memory is what an agent
            recalls — they share pgvector and nothing else.
          </p>
        </section>
      </aside>
    </MemoryFrame>
  );
}
