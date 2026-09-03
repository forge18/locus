import {
  For,
  Match,
  Show,
  Switch,
  createEffect,
  createMemo,
  createSignal,
  onMount,
  type Accessor,
} from "solid-js";
import { EditorPane } from "../../editor/EditorPane";
import { FullWindowEditor } from "../../editor/FullWindowEditor";
import { plainTextDescriptor } from "../../editor/types";
import { FixtureNotice } from "../../ui/FixtureNotice";
import {
  fetchProjectSetup,
  fetchProjects,
  fetchRepos,
  renameProject,
  saveBaseContext,
  setProjectArchived,
} from "../../data/core";
import type { Envelope } from "../../data/envelope";
import type {
  ProjectRepo,
  ProjectSetup,
  ProjectSummary,
} from "../../types/core";
import { notify } from "../../ui/Toast";
import { AnalyticsView } from "../analytics/AnalyticsView";

type ProjectTab = "settings" | "persistence" | "analytics";

/**
 * Narrows one `Envelope` signal into per-state accessors, so the markup never
 * re-reads the signal inside a ternary (a second call returns a fresh union and
 * TypeScript cannot narrow it).
 */
function envelopeParts<T>(envelope: Accessor<Envelope<T>>) {
  const data = createMemo<T | null>(() => {
    const value = envelope();
    return value.status === "ready" ? value.data : null;
  });
  const error = createMemo(() => {
    const value = envelope();
    return value.status === "failed" ? value.error : null;
  });
  const loading = createMemo(() => envelope().status === "loading");
  const empty = createMemo(() => envelope().status === "empty");
  return { data, error, loading, empty };
}

/** Fixture until epic slice 8 (`extension_inventory`); flagged in place below. */
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

export interface ProjectsViewProps {
  /** A real ordinary checkout path enables host LSP for the editor surface. */
  editorProjectRoot?: string;
  editorProjectId?: string;
  editorPaneId?: string;
}

export function ProjectsView(props: ProjectsViewProps = {}) {
  const [tab, setTab] = createSignal<ProjectTab>("settings");
  const [editorOpen, setEditorOpen] = createSignal(false);
  const [editorFullscreen, setEditorFullscreen] = createSignal(false);
  const [enabledExtensions, setEnabledExtensions] = createSignal(
    new Set<string>(extensions.map(([name]) => name)),
  );
  const [editedContent, setEditedContent] = createSignal<string | null>(null);
  const [savingBaseContext, setSavingBaseContext] = createSignal(false);
  const [renaming, setRenaming] = createSignal(false);
  const [nameDraft, setNameDraft] = createSignal("");

  const toggleExtension = (name: string) => {
    const next = new Set(enabledExtensions());
    if (next.has(name)) next.delete(name);
    else next.add(name);
    setEnabledExtensions(next);
  };

  const [projects, setProjects] = createSignal<Envelope<ProjectSummary[]>>({
    status: "loading",
  });
  const [selectedId, setSelectedId] = createSignal<string | null>(null);
  const [repos, setRepos] = createSignal<Envelope<ProjectRepo[]>>({
    status: "loading",
  });
  const [setup, setSetup] = createSignal<Envelope<ProjectSetup>>({
    status: "loading",
  });
  const projectsView = envelopeParts(projects);
  const reposView = envelopeParts(repos);
  const setupView = envelopeParts(setup);

  const selected = createMemo(() => {
    const list = projectsView.data();
    return list?.find((project) => project.id === selectedId()) ?? null;
  });

  async function refreshProjects() {
    const envelope = await fetchProjects();
    setProjects(envelope);
    if (envelope.status === "ready" && selectedId() === null) {
      setSelectedId(envelope.data[0]?.id ?? null);
    }
  }

  async function refreshProjectDetail(projectId: string) {
    setRepos({ status: "loading" });
    setSetup({ status: "loading" });
    const [reposEnvelope, setupEnvelope] = await Promise.all([
      fetchRepos(projectId),
      fetchProjectSetup(projectId),
    ]);
    // A slower response for an abandoned selection must not win the race.
    if (selectedId() !== projectId) return;
    setRepos(reposEnvelope);
    setSetup(setupEnvelope);
  }

  async function saveContext() {
    const id = selected()?.id;
    if (!id) return;
    const content = editedContent() ?? setupView.data()?.baseContext ?? "";
    const budget =
      content.trim() === "" ? undefined : setupView.data()?.baseContextTokenBudget ?? undefined;
    setSavingBaseContext(true);
    try {
      const envelope = await saveBaseContext(id, content, budget);
      if (envelope.status === "ready") {
        setSetup({ status: "ready", data: envelope.data });
        setEditedContent(null);
        setEditorOpen(false);
        notify({ title: "Base context saved" });
      } else if (envelope.status === "failed") {
        notify({
          title: "Save failed",
          description: envelope.error.message,
          type: "error",
        });
      }
    } finally {
      setSavingBaseContext(false);
    }
  }

  async function archive() {
    const id = selected()?.id;
    if (!id) return;
    const envelope = await setProjectArchived(id, true);
    if (envelope.status === "ready") {
      notify({ title: "Project archived" });
      void refreshProjects();
    } else if (envelope.status === "failed") {
      notify({
        title: "Archive failed",
        description: envelope.error.message,
        type: "error",
      });
    }
  }

  async function confirmRename() {
    const id = selected()?.id;
    const name = nameDraft().trim();
    if (!id) return;
    const envelope = await renameProject(id, name);
    if (envelope.status === "ready") {
      setRenaming(false);
      notify({ title: `Renamed to #${envelope.data.name}` });
      void refreshProjects();
    } else if (envelope.status === "failed") {
      notify({
        title: "Rename failed",
        description: envelope.error.message,
        type: "error",
      });
    }
  }

  onMount(() => {
    void refreshProjects();
  });
  createEffect(() => {
    const projectId = selectedId();
    if (projectId) void refreshProjectDetail(projectId);
  });

  const baseContextFile = () => ({
    uri: `locus://project/${selectedId() ?? ""}/settings/base-context`,
    path: "base.md",
    languageId: "plain",
    content: setupView.data()?.baseContext ?? "",
  });

  return (
    <div class="projects-view" data-testid="projects-view">
      <aside class="projects-list">
        <div class="projects-list-head">
          <div class="section-title">
            Projects
            <Show when={projectsView.data()}>
              <span>{projectsView.data()?.length}</span>
            </Show>
          </div>
          <p>
            A project is a set of repos, a memory scope, and a tag. Nothing is
            filtered to one unless you ask.
          </p>
          <button class="btn btn-primary btn-block">New project</button>
        </div>
        <div class="projects-list-items" data-testid="project-state-list">
          <Show
            when={projectsView.data()}
            fallback={
              <Switch>
                <Match when={projectsView.loading()}>
                  <p class="project-panel-note">Loading projects…</p>
                </Match>
                <Match when={projectsView.empty()}>
                  <p class="project-panel-note">
                    No projects yet. Create one to begin.
                  </p>
                </Match>
                <Match when={projectsView.error()}>
                  <p class="project-panel-note" role="alert">
                    {projectsView.error()?.message}
                  </p>
                  <button
                    class="btn btn-secondary btn-block"
                    onClick={() => void refreshProjects()}
                  >
                    Retry
                  </button>
                </Match>
              </Switch>
            }
          >
            <For each={projectsView.data()}>
              {(project) => (
                <button
                  class="project-list-item"
                  classList={{
                    "project-list-current": project.id === selectedId(),
                  }}
                  onClick={() => setSelectedId(project.id)}
                >
                  <span class="mono">#{project.name}</span>
                </button>
              )}
            </For>
          </Show>
        </div>
      </aside>
      <main class="project-detail">
        <header class="project-detail-head">
          <h1>#{selected()?.name ?? "…"}</h1>
          <span class="mono project-locator">
            {selected() ? `locus://${selected()?.name}` : "locus://…"}
          </span>
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
            <button class="btn btn-ghost" onClick={() => void archive()}>
              Archive
            </button>
            <Show
              when={renaming()}
              fallback={
                <button
                  class="btn btn-secondary"
                  onClick={() => {
                    setNameDraft(selected()?.name ?? "");
                    setRenaming(true);
                  }}
                >
                  Rename
                </button>
              }
            >
              <input
                class="project-rename-input"
                data-testid="project-rename-input"
                value={nameDraft()}
                onInput={(event) => setNameDraft(event.currentTarget.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") void confirmRename();
                  if (event.key === "Escape") setRenaming(false);
                }}
                aria-label="Project name"
              />
              <button class="btn btn-primary" onClick={() => void confirmRename()}>
                Save name
              </button>
              <button
                class="btn btn-secondary"
                onClick={() => setRenaming(false)}
              >
                Cancel
              </button>
            </Show>
          </div>
        </header>
        <Show
          when={tab() === "settings"}
          fallback={
            tab() === "persistence" ? (
              <Persistence />
            ) : (
              <AnalyticsView scope={selected()?.name ?? ""} />
            )
          }
        >
          <div class="project-settings" data-testid="project-settings">
            <section class="project-panel" data-testid="project-harnesses">
              <PanelTitle
                title="Harnesses"
                note="which harnesses may run here, and which one an unattended agent gets by default"
              />
              <Switch>
                <Match when={setupView.loading()}>
                  <p class="project-panel-note">Loading harness policy…</p>
                </Match>
                <Match when={setupView.error()}>
                  <p class="project-panel-note" role="alert">
                    {setupView.error()?.message}
                  </p>
                </Match>
                <Match when={setupView.data()?.harnessAllowList.length === 0}>
                  <p class="project-panel-note">
                    No harness policy is stored for this project yet.
                  </p>
                </Match>
              </Switch>
              <For each={setupView.data()?.harnessAllowList ?? []}>
                {(harness) => (
                  <div class="project-harness-row">
                    <button
                      class="project-check"
                      aria-label={`${harness} is allowed here`}
                      aria-pressed="true"
                    >
                      ✓
                    </button>
                    <span class="mono">{harness}</span>
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
              <Switch>
                <Match when={reposView.loading()}>
                  <p class="project-panel-note">Loading repos…</p>
                </Match>
                <Match when={reposView.empty()}>
                  <p class="project-panel-note">
                    No repos in this project yet. Add one to give agents a
                    workspace.
                  </p>
                </Match>
                <Match when={reposView.error()}>
                  <p class="project-panel-note" role="alert">
                    {reposView.error()?.message}
                  </p>
                  <button
                    class="btn btn-secondary"
                    onClick={() => {
                      const id = selected()?.id;
                      if (id) void refreshProjectDetail(id);
                    }}
                  >
                    Retry
                  </button>
                </Match>
              </Switch>
              <For each={reposView.data() ?? []}>
                {(repo) => (
                  <Repo name={repo.name} path={repo.workingCopyPath} />
                )}
              </For>
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
                <Show when={setupView.data()?.baseContextTokenBudget != null}>
                  <div
                    class="project-budget"
                    data-testid="project-base-context-budget"
                  >
                    <b>
                      budget: {setupView.data()?.baseContextTokenBudget} tokens
                    </b>
                  </div>
                </Show>
              </div>
              <Switch>
                <Match when={setupView.loading()}>
                  <div
                    class="base-context"
                    data-testid="project-base-context-editor"
                  >
                    <p class="project-panel-note">Loading base context…</p>
                  </div>
                </Match>
                <Match when={setupView.error()}>
                  <div
                    class="base-context"
                    data-testid="project-base-context-editor"
                  >
                    <p class="project-panel-note" role="alert">
                      {setupView.error()?.message}
                    </p>
                  </div>
                </Match>
                <Match
                  when={
                    setupView.data() && setupView.data()?.baseContext == null
                  }
                >
                  <div
                    class="base-context"
                    data-testid="project-base-context-editor"
                  >
                    <p class="project-panel-note">
                      No base context yet — this project's runs start with none.
                    </p>
                  </div>
                </Match>
              </Switch>
              <Show when={setupView.data()?.baseContext != null}>
                <div
                  class="base-context"
                  data-testid="project-base-context-editor"
                >
                  <header>
                    <strong class="mono">base.md</strong>
                    <button class="btn btn-secondary">History</button>
                    <button
                      class="btn btn-secondary"
                      data-testid="project-base-context-edit"
                      onClick={() => setEditorOpen(!editorOpen())}
                    >
                      {editorOpen() ? "Preview" : "Edit"}
                    </button>
                    <button
                      class="btn btn-primary"
                      data-testid="project-base-context-save"
                      disabled={savingBaseContext()}
                      onClick={() => void saveContext()}
                    >
                      {savingBaseContext() ? "Saving…" : "Save"}
                    </button>
                  </header>
                  <Show
                    when={editorOpen()}
                    fallback={
                      <div
                        class="base-prose"
                        style={{ "white-space": "pre-wrap" }}
                      >
                        {setupView.data()?.baseContext}
                      </div>
                    }
                  >
                    <EditorPane
                      onChange={setEditedContent}
                      file={baseContextFile()}
                      language={plainTextDescriptor}
                      projectRoot={props.editorProjectRoot}
                      projectId={props.editorProjectId}
                      paneId={props.editorPaneId}
                    />
                  </Show>
                </div>
              </Show>
              <Show when={editorFullscreen()}>
                <div
                  class="project-editor-overlay"
                  role="dialog"
                  aria-label="Base context editor"
                  data-testid="project-editor-overlay"
                >
                  <header>
                    <strong class="mono">base.md</strong>
                    <button
                      class="btn btn-secondary"
                      onClick={() => setEditorFullscreen(false)}
                    >
                      Close
                    </button>
                  </header>
                  <FullWindowEditor
                    file={baseContextFile()}
                    language={plainTextDescriptor}
                    projectRoot={props.editorProjectRoot}
                    projectId={props.editorProjectId}
                    paneId={props.editorPaneId}
                  />
                </div>
              </Show>
              <Show when={editorOpen()}>
                <button
                  class="btn btn-secondary project-editor-fullscreen"
                  data-testid="project-base-context-fullscreen"
                  onClick={() => setEditorFullscreen(true)}
                >
                  Open full window
                </button>
              </Show>
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
              <FixtureNotice
                surface="Extension counts"
                command='invoke("extension_inventory")'
              />
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
                          class="project-toggle"
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
              <FixtureNotice
                surface="CLI tools"
                command='invoke("extension_inventory")'
              />
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

function Repo(props: { name: string; path: string }) {
  return (
    <div class="project-repo" data-testid="project-repo-row">
      <div>
        <strong class="mono">{props.name}</strong>
        <small class="mono">{props.path}</small>
      </div>
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
