import { For, Show, createSignal } from "solid-js";
import { AnalyticsView } from "../analytics/AnalyticsView";

type ProjectTab = "settings" | "persistence" | "analytics";

const projects = [
  {
    name: "tapestry",
    detail: "2 repos · core, desktop",
    activity: "3 running",
  },
  { name: "loom-db", detail: "1 repo · loom", activity: "2 running" },
  {
    name: "weaver",
    detail: "3 repos · keymap, term, ui",
    activity: "2 running",
  },
  { name: "texere", detail: "1 repo · media", activity: "1 waiting" },
  { name: "amq", detail: "1 repo · amq · archived 6d", activity: "idle" },
];

const harnesses = [
  { id: "claude", adapter: "ACP", detail: "Anthropic · opus-4.6" },
  { id: "codex", adapter: "ACP", detail: "OpenAI · gpt-5.2-pro" },
  { id: "gemini", adapter: "ACP", detail: "Google · gemini-3-ultra" },
  { id: "cursor", adapter: "ACP", detail: "Cursor · composer-2" },
];

const extensions = [
  ["Agents", "4 enabled"],
  ["Commands", "6 enabled"],
  ["Hooks", "3 enabled"],
  ["Linters", "8 enabled"],
  ["Output styles", "2 enabled"],
  ["Rules", "12 enabled"],
  ["Skills", "9 enabled"],
  ["Base context", "1 enabled"],
] as const;

export function ProjectsView() {
  const [tab, setTab] = createSignal<ProjectTab>("settings");
  const [defaultHarness, setDefaultHarness] = createSignal("claude");
  const [enabledHarnesses, setEnabledHarnesses] = createSignal(
    new Set(harnesses.map(({ id }) => id)),
  );
  const [enabledExtensions, setEnabledExtensions] = createSignal(
    new Set<string>(extensions.map(([name]) => name)),
  );

  const toggleHarness = (id: string) => {
    const next = new Set(enabledHarnesses());
    if (next.has(id) && next.size > 1) next.delete(id);
    else next.add(id);
    if (!next.has(defaultHarness())) setDefaultHarness([...next][0]);
    setEnabledHarnesses(next);
  };
  const toggleExtension = (name: string) => {
    const next = new Set(enabledExtensions());
    if (next.has(name)) next.delete(name);
    else next.add(name);
    setEnabledExtensions(next);
  };

  return (
    <div class="projects-view" data-testid="projects-view">
      <aside class="projects-list">
        <div class="projects-list-head">
          <div class="section-title">
            Projects <span>5</span>
          </div>
          <p>
            A project is a set of repos, a memory scope, and a tag. Nothing is
            filtered to one unless you ask.
          </p>
          <button class="btn btn-primary btn-block">New project</button>
        </div>
        <div class="projects-list-items" data-testid="project-state-list">
          <For each={projects}>
            {(project, index) => (
              <button
                class="project-list-item"
                classList={{ "project-list-current": index() === 0 }}
              >
                <span class="mono">#{project.name}</span>
                <span>{project.activity}</span>
                <small>{project.detail}</small>
              </button>
            )}
          </For>
        </div>
      </aside>
      <main class="project-detail">
        <header class="project-detail-head">
          <h1>#tapestry</h1>
          <span class="mono project-locator">locus://tapestry</span>
          <div class="project-tabs" role="tablist" aria-label="Project view">
            <button
              data-testid="project-tab-settings"
              role="tab"
              aria-selected={tab() === "settings"}
              onClick={() => setTab("settings")}
            >
              Settings
            </button>
            <button
              data-testid="project-tab-persistence"
              role="tab"
              aria-selected={tab() === "persistence"}
              onClick={() => setTab("persistence")}
            >
              Persistence
            </button>
            <button
              data-testid="project-tab-analytics"
              role="tab"
              aria-selected={tab() === "analytics"}
              onClick={() => setTab("analytics")}
            >
              Analytics
            </button>
          </div>
          <div class="project-detail-actions">
            <button class="btn btn-ghost">Archive</button>
            <button class="btn btn-secondary">Rename</button>
          </div>
        </header>
        <Show
          when={tab() === "settings"}
          fallback={
            tab() === "persistence" ? (
              <Persistence />
            ) : (
              <AnalyticsView scope="tapestry" />
            )
          }
        >
          <div class="project-settings" data-testid="project-settings">
            <section class="project-panel" data-testid="project-harnesses">
              <PanelTitle
                title="Harnesses"
                note="which harnesses may run here, and which one an unattended agent gets by default"
              />
              <div class="project-harness-head">
                <span>Enabled</span>
                <span>Harness</span>
                <span>Adapter</span>
                <span>Provider · model</span>
                <span>Agent default</span>
              </div>
              <For each={harnesses}>
                {(harness) => (
                  <div class="project-harness-row">
                    <button
                      class="project-check"
                      aria-label={`Enable ${harness.id}`}
                      aria-pressed={enabledHarnesses().has(harness.id)}
                      onClick={() => toggleHarness(harness.id)}
                    >
                      {enabledHarnesses().has(harness.id) ? "✓" : ""}
                    </button>
                    <span class="mono">{harness.id}</span>
                    <span class="project-adapter">{harness.adapter}</span>
                    <span>{harness.detail}</span>
                    <button
                      data-testid={`harness-default-${harness.id}`}
                      class="project-default"
                      disabled={!enabledHarnesses().has(harness.id)}
                      aria-pressed={defaultHarness() === harness.id}
                      onClick={() => setDefaultHarness(harness.id)}
                    >
                      <i />
                      {defaultHarness() === harness.id
                        ? "default"
                        : "make default"}
                    </button>
                  </div>
                )}
              </For>
              <p
                class="project-panel-note"
                data-testid="project-router-summary"
              >
                Enabled harnesses are offered to the router in the order listed;
                anything the router does not claim runs on the agent default.
              </p>
            </section>

            <section class="project-panel" data-testid="project-repos">
              <PanelTitle
                title="Repos"
                note="a repo belongs to exactly one project — this is where that is decided"
                action="Add repo"
              />
              <Repo
                name="core"
                url="git@github.com:forge18/tapestry-core.git"
                status="main + 3 agent branches"
                activity="3 running"
              />
              <Repo
                name="desktop"
                url="git@github.com:forge18/tapestry-desktop.git"
                status="main + 1 agent branch"
                activity="clean"
              />
              <p class="project-panel-note">
                Moving a repo re-tags every run, artifact and memory fact that
                came from it. The old tag stays on the record so history does
                not silently change project.
              </p>
            </section>

            <section class="project-panel" data-testid="project-base-context">
              <div class="project-panel-title">
                <div>
                  <span>Base context</span>
                  <small>
                    always loaded, every run in this project — exactly one, and
                    there is no second
                  </small>
                </div>
                <div
                  class="project-budget"
                  data-testid="project-base-context-budget"
                >
                  <b>1,240 / 1,500 tokens</b>
                  <i>
                    <em />
                  </i>
                </div>
              </div>
              <div
                class="base-context"
                data-testid="project-base-context-editor"
              >
                <header>
                  <strong class="mono">base.md</strong>
                  <small>v9 · edited 5h ago · loaded by 1,204 runs</small>
                  <button class="btn btn-secondary">History</button>
                  <button class="btn btn-primary">Save</button>
                </header>
                <div class="base-prose">
                  <strong># Working in tapestry</strong>
                  <p>
                    You are working in a clone of a bare local remote. Your
                    branch is never main and you cannot reach the host
                    filesystem. Push the branch; a human decides what lands.
                  </p>
                  <p>
                    Record what you learn with <b>locus memory write</b> at
                    project scope, and recall before you explore — the answer is
                    usually already a fact.
                  </p>
                  <p>
                    Verify with <b>cargo nextest run</b>. A claim without the
                    command and its exit code is not a claim.
                    <i />
                  </p>
                </div>
              </div>
              <p class="project-panel-note">
                Kept short on purpose: it is the one file every run pays for.
                Over budget usually means something belongs in a skill or a rule
                instead.
              </p>
            </section>

            <section
              class="project-panel"
              data-testid="project-extension-groups"
            >
              <PanelTitle
                title="Extensions"
                note="pulled from the defaults in Workshop — switch one off and this project materializes without it"
              />
              <div class="extension-grid">
                <For each={extensions}>
                  {([name, count]) => (
                    <div class="extension-card">
                      <div>
                        <strong>{name}</strong>
                        <small>{count}</small>
                        <button
                          class="toggle"
                          aria-label={`Enable ${name}`}
                          data-on={enabledExtensions().has(name)}
                          onClick={() => toggleExtension(name)}
                        >
                          <i />
                        </button>
                      </div>
                      <p>
                        {enabledExtensions().has(name)
                          ? "Included on the next run"
                          : "Excluded from the materialized tree"}
                      </p>
                    </div>
                  )}
                </For>
              </div>
              <p class="project-panel-note">
                Definitions are global; what is per-project is which of them
                this project gets. Switching one off here removes it from the
                materialized tree on the next run — it does not delete it.
              </p>
            </section>

            <section class="project-panel" data-testid="project-cli-tools">
              <PanelTitle
                title="CLI tools"
                note="installed in this project’s container image — agents get exactly these, nothing else"
              />
              <div class="project-tools">
                <div>
                  <div class="project-search">
                    <span>⌕</span>
                    <b class="mono">sqlx</b>
                    <small>3 of 214 match</small>
                  </div>
                  <Tool
                    name="sqlx-cli"
                    version="0.8.2"
                    meta="cargo · on PATH here"
                    action="Add"
                  />
                  <Tool
                    name="sqlx-prepare-check"
                    version="0.3.1"
                    meta="store · unsigned"
                    action="Add"
                  />
                  <Tool
                    name="sqlfluff"
                    version="3.2.0"
                    meta="pipx"
                    action="Add"
                  />
                </div>
                <div>
                  <div class="section-title">In this project · 5</div>
                  <Tool
                    name="ripgrep"
                    version="14.1.1"
                    meta="all agents"
                    action="×"
                  />
                  <Tool
                    name="cargo-nextest"
                    version="0.9.88"
                    meta="impl, maintain"
                    action="×"
                  />
                  <Tool
                    name="psql"
                    version="17.2"
                    meta="all agents"
                    action="×"
                  />
                  <Tool
                    name="jq"
                    version="1.7.1"
                    meta="all agents"
                    action="×"
                  />
                  <Tool
                    name="gh"
                    version="2.63.0"
                    meta="needs a token"
                    action="×"
                  />
                </div>
              </div>
              <p class="project-panel-note">
                Adding a tool rebuilds the image once, not per run. A tool an
                agent cannot find is the second most common reason a run stalls,
                so the plan’s tool list lands here on approval.
              </p>
            </section>
          </div>
        </Show>
      </main>
    </div>
  );
}

function PanelTitle(props: { title: string; note: string; action?: string }) {
  return (
    <div class="project-panel-title">
      <div>
        <span>{props.title}</span>
        <small>{props.note}</small>
      </div>
      <Show when={props.action}>
        <button class="btn btn-secondary">{props.action}</button>
      </Show>
    </div>
  );
}

function Repo(props: {
  name: string;
  url: string;
  status: string;
  activity: string;
}) {
  return (
    <div class="project-repo">
      <div>
        <strong class="mono">{props.name}</strong>
        <small class="mono">{props.url}</small>
      </div>
      <span class="mono" data-testid="project-repo-branch-state">
        {props.status} · {props.activity}
      </span>
      <b>#tapestry</b>
    </div>
  );
}

function Tool(props: {
  name: string;
  version: string;
  meta: string;
  action: string;
}) {
  return (
    <div class="project-tool">
      <strong class="mono">{props.name}</strong>
      <small class="mono">{props.version}</small>
      <span>{props.meta}</span>
      <button>{props.action}</button>
    </div>
  );
}

function Persistence() {
  return (
    <div class="project-persistence" data-testid="project-persistence">
      <p>
        Everything this project has kept, in one place. Memory tiers decay on
        their own schedule; specs and research stay until you delete them.
      </p>
      <section>
        <h2>Memory</h2>
        <p>Short-Term · clears at session end unless promoted</p>
        <p>Long-Term · promoted facts — survive the session</p>
        <p>Artifacts · what runs left behind</p>
      </section>
      <section>
        <h2>Specs &amp; Tasks</h2>
        <p>Plans and their nested board tasks stay until you delete them.</p>
      </section>
      <section>
        <h2>Research</h2>
        <p>Source and synthesis entries · no delete control</p>
      </section>
    </div>
  );
}
