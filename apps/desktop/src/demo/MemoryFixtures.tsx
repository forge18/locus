import {
  For,
  Match,
  Show,
  Switch,
  createEffect,
  createMemo,
  createSignal,
  type JSX,
} from "solid-js";
import { isTauri } from "@tauri-apps/api/core";
import { Button } from "../ui/Button";
import { FixtureNotice } from "../ui/FixtureNotice";
import { InlineError } from "../ui/InlineError";
import { Tag } from "../ui/Tag";
import {
  ARTIFACT_LOCATOR,
  artifactMediaUrl,
  fetchArtifactsFromCore,
} from "../data/artifacts";
import {
  COMPACTED_CONTEXT,
  CURATION_COPY,
  fetchLongTermFacts,
  LONG_TERM_FACTS,
  setMemoryFactConfidence,
  RESIDENT_LAYERS,
  SHORT_TERM_COPY,
  WIKI_CONTRADICTION_COPY,
  WIKI_GRAPH_COPY,
  WIKI_INGEST_COPY,
  WIKI_KIND_CHIPS,
} from "../data/knowledge";
import { failed, type Envelope } from "../data/envelope";
import type { KnowledgeFact } from "../fixtures/knowledge";
import { ARTIFACTS } from "../fixtures/artifacts";
import { MailView } from "../screens/mail/MailView";
import type { Artifact } from "../types/agents";
import {
  GraphRenderer,
  type GraphEdgeShape,
  type GraphNodeShape,
} from "../workflow-canvas/GraphRenderer";

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
  { title: "Locus architecture", kind: "concept" },
  {
    title: "Clone from a local bare remote, never a mount",
    kind: "decision",
  },
  { title: "No MCP servers, ever", kind: "decision" },
  { title: "Byte-deterministic materialization", kind: "concept" },
  { title: "credential broker", kind: "entity" },
  { title: "canary token", kind: "entity" },
] as const;

const WIKI_GRAPH_NODES: GraphNodeShape[] = [
  { id: "bare-remote", label: "bare remote", x: 18, y: 54, focal: true },
  { id: "clone", label: "clone-not-mount", x: 116, y: 28 },
  { id: "locusd", label: "locusd", x: 116, y: 80 },
  { id: "determinism", label: "determinism", x: 214, y: 54 },
];
const WIKI_GRAPH_EDGES: GraphEdgeShape[] = [
  { from: "bare-remote", to: "clone" },
  { from: "bare-remote", to: "locusd" },
  { from: "clone", to: "determinism" },
];

function Label(props: { children: string }) {
  return <div class="desktop-memory-label">{props.children}</div>;
}

function FactList(props: { facts: KnowledgeFact[] }) {
  return (
    <div class="desktop-memory-list" aria-label="Memory facts">
      <For each={props.facts}>
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

function UnavailableMemoryView(props: {
  testId: string;
  route: string;
  surface: string;
  command: string;
}) {
  return (
    <MemoryFrame testId={props.testId} route={props.route}>
      <section class="desktop-memory-main">
        <InlineError
          cause={`${props.surface} is unavailable`}
          next={`${props.command} has no persisted desktop contract yet.`}
        />
      </section>
    </MemoryFrame>
  );
}

/** The rebuilt, per-iteration context window fixture. */
export function MemoryShortTermFixture() {
  if (isTauri()) {
    return (
      <UnavailableMemoryView
        testId="desktop-memory-short-term"
        route="short"
        surface="Short-term memory"
        command="memory_short_term"
      />
    );
  }

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
        <FixtureNotice
          surface="Short-term memory"
          command='invoke("knowledge_snapshot")'
        />
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

function MemoryLongTermLive(props: { projectId?: string }) {
  const [facts, setFacts] = createSignal<Envelope<KnowledgeFact[]>>({
    status: "loading",
  });
  const factRows = createMemo(() => {
    const state = facts();
    return state.status === "ready" ? state.data : [];
  });
  const errorMessage = createMemo(() => {
    const state = facts();
    return state.status === "failed"
      ? `${state.error.command}: ${state.error.message}`
      : "";
  });
  const [mutationStatus, setMutationStatus] = createSignal<
    "idle" | "saving" | "saved" | "failed"
  >("idle");
  const [mutationError, setMutationError] = createSignal("");
  let requestId = 0;

  const loadFacts = (projectId?: string) => {
    const request = ++requestId;
    setFacts({ status: "loading" });
    if (!projectId) {
      setFacts(
        failed("memory_facts", "an active project is required to read memory"),
      );
      return;
    }
    void fetchLongTermFacts(projectId)
      .then((result) => {
        if (request === requestId) setFacts(result);
      })
      .catch((cause) => {
        if (request === requestId) setFacts(failed("memory_facts", cause));
      });
  };

  const adjudicate = () => {
    const fact = factRows()[0];
    if (!props.projectId || !fact) return;
    setMutationStatus("saving");
    setMutationError("");
    void setMemoryFactConfidence(props.projectId, fact.id, "verified")
      .then((result) => {
        if (result.status === "failed") {
          setMutationError(`${result.error.command}: ${result.error.message}`);
          setMutationStatus("failed");
        } else {
          setMutationStatus("saved");
          loadFacts(props.projectId);
        }
      })
      .catch((cause) => {
        setMutationError(String(cause));
        setMutationStatus("failed");
      });
  };

  createEffect(() => loadFacts(props.projectId));

  return (
    <MemoryFrame
      testId="desktop-memory-long-term"
      route="memory"
      factFixture="live"
      factState="loading-empty-ready-failed"
    >
      <aside class="desktop-memory-left">
        <header class="desktop-memory-pane-head">
          <h1>
            Long-term <code>{factRows().length}</code>
          </h1>
          <p>Project facts recalled across runs.</p>
        </header>
        <section class="desktop-memory-scope">
          <Label>Scope</Label>
          <span>{props.projectId ?? "No active project"}</span>
          <small>never cross-project</small>
        </section>
        <div data-testid="memory-facts-state" data-state={facts().status}>
          <Switch>
            <Match when={facts().status === "loading"}>
              <p class="desktop-memory-note">Loading project memory…</p>
            </Match>
            <Match when={facts().status === "failed"}>
              <InlineError
                cause={errorMessage()}
                next="Check the project connection and retry this view."
              />
            </Match>
            <Match when={facts().status === "empty"}>
              <p class="desktop-memory-note">
                No durable facts for this project.
              </p>
            </Match>
            <Match when={facts().status === "ready"}>
              <FactList facts={factRows()} />
            </Match>
          </Switch>
        </div>
      </aside>
      <section class="desktop-memory-main">
        <header class="desktop-memory-title-row desktop-memory-stacked">
          <div>
            <Tag>memory</Tag>
            <h2>{factRows()[0]?.title ?? "Project memory"}</h2>
          </div>
          <p>Live data from the project-scoped memory store.</p>
        </header>
        <div class="desktop-memory-scroll">
          <Switch>
            <Match when={facts().status === "loading"}>
              <p class="desktop-memory-note">
                Loading the selected project’s facts…
              </p>
            </Match>
            <Match when={facts().status === "failed"}>
              <InlineError
                cause={errorMessage()}
                next="The memory detail is unavailable until the backend read succeeds."
              />
            </Match>
            <Match when={facts().status === "empty"}>
              <p class="desktop-memory-prose">
                This project has no persisted long-term memory yet.
              </p>
            </Match>
            <Match when={facts().status === "ready"}>
              <p class="desktop-memory-prose">
                {factRows()[0].title} is persisted for this project. The list
                shows its current confidence and recall summary.
              </p>
              <section>
                <Label>Current state</Label>
                <p class="desktop-memory-note">
                  {factRows()[0].confidence} · {factRows()[0].recall}
                  {factRows()[0].score === null
                    ? " · no score"
                    : ` · score ${factRows()[0]!.score!.toFixed(2)}`}
                </p>
                <Button
                  variant="primary"
                  data-testid="memory-adjudicate"
                  disabled={mutationStatus() === "saving"}
                  onClick={adjudicate}
                >
                  {mutationStatus() === "saving" ? "Saving…" : "Adjudicate"}
                </Button>
                <Show when={mutationStatus() === "saved"}>
                  <p role="status">Adjudication saved.</p>
                </Show>
                <Show when={mutationStatus() === "failed"}>
                  <InlineError
                    cause={mutationError()}
                    next="The confidence state was not changed."
                  />
                </Show>
              </section>
            </Match>
          </Switch>
        </div>
      </section>
    </MemoryFrame>
  );
}

/** Project-scoped facts, their provenance, decay, and contradiction state. */
export function MemoryLongTermFixture(props: { projectId?: string } = {}) {
  if (isTauri()) return <MemoryLongTermLive projectId={props.projectId} />;

  const [editing, setEditing] = createSignal(false);
  const [correction, setCorrection] = createSignal(
    "NOTIFY payload caps at 8000 bytes",
  );
  const [actionNotice, setActionNotice] = createSignal("");

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
        <FactList facts={LONG_TERM_FACTS} />
      </aside>

      <section class="desktop-memory-main">
        <FixtureNotice
          surface="Long-term memory"
          command='invoke("knowledge_snapshot")'
        />
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
            <Button
              data-testid="edit-recalled-fact"
              onClick={() => {
                setEditing(true);
                setActionNotice("");
              }}
            >
              Edit recalled fact
            </Button>
            <Show when={editing()}>
              <textarea
                data-testid="memory-recalled-fact-editor"
                aria-label="Recalled fact correction"
                value={correction()}
                onInput={(event) => setCorrection(event.currentTarget.value)}
              />
              <Button
                data-testid="memory-save-revision"
                onClick={() => {
                  if (!correction().trim()) {
                    setActionNotice("A correction is required before saving.");
                    return;
                  }
                  setEditing(false);
                  setActionNotice("Revision 2 staged in the demo provider.");
                }}
              >
                Save revision 2
              </Button>
            </Show>
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
          <Button
            variant="primary"
            onClick={() =>
              setActionNotice("Contradiction adjudicated in the demo provider.")
            }
          >
            Adjudicate
          </Button>
          <Show when={actionNotice()}>
            <p role="status" data-testid="memory-action-status">
              {actionNotice()}
            </p>
          </Show>
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

function MemoryArtifactsLive(props: { projectId?: string }) {
  const [artifacts, setArtifacts] = createSignal<Envelope<Artifact[]>>({
    status: "loading",
  });
  const rows = createMemo(() => {
    const state = artifacts();
    return state.status === "ready" ? state.data : [];
  });
  const errorMessage = createMemo(() => {
    const state = artifacts();
    return state.status === "failed"
      ? `${state.error.command}: ${state.error.message}`
      : "";
  });

  let requestId = 0;
  createEffect(() => {
    const projectId = props.projectId;
    const request = ++requestId;
    setArtifacts({ status: "loading" });
    if (!projectId) {
      setArtifacts(
        failed(
          "artifacts_list",
          "an active project is required to read artifacts",
        ),
      );
      return;
    }
    void fetchArtifactsFromCore(projectId)
      .then((result) => {
        if (request === requestId) setArtifacts(result);
      })
      .catch((cause) => {
        if (request === requestId)
          setArtifacts(failed("artifacts_list", cause));
      });
  });

  return (
    <MemoryFrame
      testId="desktop-memory-artifacts"
      route="artifact"
      artifactGroups="review-reference"
      artifactPreview="live"
    >
      <aside class="desktop-memory-left">
        <header class="desktop-memory-pane-head">
          <h1>
            Review artifacts <code>{rows().length}</code>
          </h1>
          <p>Project-scoped artifacts retained for human review.</p>
        </header>
        <div
          data-testid="memory-artifacts-state"
          data-state={artifacts().status}
        >
          <Switch>
            <Match when={artifacts().status === "loading"}>
              <p class="desktop-memory-note">Loading review artifacts…</p>
            </Match>
            <Match when={artifacts().status === "failed"}>
              <InlineError
                cause={errorMessage()}
                next="Check the project connection and retry this view."
              />
            </Match>
            <Match when={artifacts().status === "empty"}>
              <p class="desktop-memory-note">
                No review artifacts for this project.
              </p>
            </Match>
            <Match when={artifacts().status === "ready"}>
              <div class="desktop-memory-list">
                <For each={rows()}>
                  {(artifact, index) => (
                    <div
                      class="desktop-memory-list-item"
                      data-selected={index() === 0 ? "true" : undefined}
                    >
                      <div>{artifact.title}</div>
                      <small>{artifact.kind}</small>
                    </div>
                  )}
                </For>
              </div>
            </Match>
          </Switch>
        </div>
      </aside>
      <section class="desktop-memory-main">
        <header class="desktop-memory-title-row">
          <Tag>artifact</Tag>
          <h2>{rows()[0]?.title ?? "Review artifacts"}</h2>
        </header>
        <div class="desktop-memory-scroll">
          <Switch>
            <Match when={artifacts().status === "loading"}>
              <p class="desktop-memory-note">Loading artifact details…</p>
            </Match>
            <Match when={artifacts().status === "failed"}>
              <InlineError
                cause={errorMessage()}
                next="The artifact detail is unavailable until the backend read succeeds."
              />
            </Match>
            <Match when={artifacts().status === "empty"}>
              <p class="desktop-memory-prose">
                Nothing has been retained for review yet.
              </p>
            </Match>
            <Match when={artifacts().status === "ready"}>
              <p class="desktop-memory-prose">
                {rows()[0]!.body ??
                  rows()[0]!.derivedText ??
                  "This artifact has no text preview."}
              </p>
              <small>Source retained by the project artifact store.</small>
            </Match>
          </Switch>
        </div>
      </section>
    </MemoryFrame>
  );
}

/** Reviewable artifacts retain their source while comments steer their session. */
export function MemoryArtifactsFixture(props: { projectId?: string } = {}) {
  if (isTauri()) return <MemoryArtifactsLive projectId={props.projectId} />;

  const coreArtifacts = () => [] as Artifact[];
  const mediaArtifact = (kind: "image" | "recording") =>
    coreArtifacts().find((artifact) => artifact.kind === kind) ??
    ARTIFACTS.find((artifact) => artifact.kind === kind)!;
  const image = () => mediaArtifact("image");
  const recording = () => mediaArtifact("recording");
  const imageFallback = "data:image/webp;base64,UklGRg==";
  const recordingFallback = "data:video/webm;base64,GkXfo0AgQoaBAULygQ==";

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
        <FixtureNotice surface="Artifacts" command='invoke("artifacts_list")' />
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
        <section
          class="desktop-memory-media-viewers"
          data-testid="artifacts-media-viewers"
        >
          <figure data-media-kind="image">
            <img
              alt="Derived screenshot preview"
              data-artifact-id={image().id}
              src={artifactMediaUrl(image(), imageFallback)}
            />
            <figcaption>
              image · original preserved · derived preview
            </figcaption>
          </figure>
          <figure data-media-kind="recording">
            <video
              controls
              aria-label="Recording keyframes"
              data-artifact-id={recording().id}
              preload="metadata"
            >
              <source
                src={artifactMediaUrl(recording(), recordingFallback)}
                type={recording().mediaType}
              />
            </video>
            <figcaption>
              recording · keyframes for context · clip stays human-only
            </figcaption>
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
  if (isTauri()) {
    return (
      <UnavailableMemoryView
        testId="desktop-memory-wiki"
        route="wiki"
        surface="Wiki"
        command="wiki_pages"
      />
    );
  }

  const [wikiKind, setWikiKind] = createSignal("all");
  const visiblePages = createMemo(() =>
    wikiKind() === "all"
      ? pages
      : pages.filter((page) => page.kind === wikiKind()),
  );
  const selectedKind = createMemo(
    () =>
      WIKI_KIND_CHIPS.find((chip) => chip.kind === wikiKind()) ??
      WIKI_KIND_CHIPS[0],
  );

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
            {selectedKind().label} <code>{selectedKind().count}</code>
          </h1>
          <p>
            Curated project knowledge derived from sources, then reviewed by
            people.
          </p>
          <Button variant="primary" block>
            Ingest a document
          </Button>
        </header>
        <div
          class="desktop-memory-kinds"
          aria-label="Wiki kinds"
          data-testid="wiki-kind-filter"
        >
          <For each={WIKI_KIND_CHIPS}>
            {(chip) => (
              <button
                type="button"
                data-kind={chip.kind}
                data-active={wikiKind() === chip.kind ? "true" : undefined}
                aria-pressed={wikiKind() === chip.kind}
                onClick={() => setWikiKind(chip.kind)}
              >
                {chip.label} {chip.count}
              </button>
            )}
          </For>
        </div>
        <div class="desktop-memory-list" data-testid="wiki-pages">
          <For each={visiblePages()}>
            {(page, index) => (
              <div
                class="desktop-memory-list-item"
                data-selected={index() === 1 ? "true" : undefined}
                data-page-kind={page.kind}
              >
                {page.title}
              </div>
            )}
          </For>
        </div>
        <footer class="desktop-memory-pane-foot">{WIKI_INGEST_COPY}</footer>
      </aside>
      <section class="desktop-memory-main">
        <FixtureNotice surface="Wiki" command='invoke("wiki_pages")' />
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
            <GraphRenderer
              nodes={WIKI_GRAPH_NODES}
              edges={WIKI_GRAPH_EDGES}
              width={258}
              height={132}
              showLabels
              testId="wiki-graph-renderer"
            />
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
