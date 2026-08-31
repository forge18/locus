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
import { Button } from "../../ui/Button";
import { InlineError } from "../../ui/InlineError";
import { Tag } from "../../ui/Tag";
import { fetchArtifactsFromCore } from "../../data/artifacts";
import type { Artifact } from "../../types/agents";
import {
  fetchLongTermFacts,
  setMemoryFactConfidence,
} from "../../data/knowledge";
import { failed, type Envelope } from "../../data/envelope";
import type { KnowledgeFact } from "../../data/knowledge";

function MemoryFrame(props: {
  testId: string;
  route: string;
  children: JSX.Element;
}) {
  return (
    <main
      class="desktop-memory"
      data-testid={props.testId}
      data-desktop-route={props.route}
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

export function MemoryShortTermView() {
  return (
    <UnavailableMemoryView
      testId="desktop-memory-short-term"
      route="short"
      surface="Short-term memory"
      command="memory_short_term"
    />
  );
}

export function MemoryWikiView() {
  return (
    <UnavailableMemoryView
      testId="desktop-memory-wiki"
      route="wiki"
      surface="Wiki"
      command="wiki_pages"
    />
  );
}

export function MemoryLongTermView(props: { projectId?: string } = {}) {
  const [facts, setFacts] = createSignal<Envelope<KnowledgeFact[]>>({
    status: "loading",
  });
  const rows = createMemo(() => {
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

  createEffect(() => loadFacts(props.projectId));

  const adjudicate = () => {
    const fact = rows()[0];
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

  return (
    <MemoryFrame testId="desktop-memory-long-term" route="memory">
      <aside class="desktop-memory-left">
        <header class="desktop-memory-pane-head">
          <h1>
            Long-term <code>{rows().length}</code>
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
              <FactList facts={rows()} />
            </Match>
          </Switch>
        </div>
      </aside>
      <section class="desktop-memory-main">
        <header class="desktop-memory-title-row desktop-memory-stacked">
          <div>
            <Tag>memory</Tag>
            <h2>{rows()[0]?.title ?? "Project memory"}</h2>
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
                {rows()[0].title} is persisted for this project. The list shows
                its current confidence and recall summary.
              </p>
              <section>
                <Label>Current state</Label>
                <p class="desktop-memory-note">
                  {rows()[0].confidence} · {rows()[0].recall}
                  {rows()[0].score === null
                    ? " · no score"
                    : ` · score ${rows()[0]!.score!.toFixed(2)}`}
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

export function MemoryArtifactsView(props: { projectId?: string } = {}) {
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
    <MemoryFrame testId="desktop-memory-artifacts" route="artifact">
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
