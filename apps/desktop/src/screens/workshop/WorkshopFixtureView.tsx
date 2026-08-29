import { createSignal, For, Show } from "solid-js";
import { Button } from "../../ui/Button";
import { FixtureNotice } from "../../ui/FixtureNotice";
import { Input } from "../../ui/Input";
import { Segmented } from "../../ui/Segmented";
import { Tag } from "../../ui/Tag";
import "./workshop-fixtures.css";
import { ExtensionEditor, type ExtensionEditorType } from "./ExtensionEditor";
import { WorkflowView } from "./WorkflowView";

export const WORKSHOP_FIXTURES = [
    "agents",
    "cli",
    "commands",
    "context",
    "harnesses",
    "hooks",
    "linters",
    "output-styles",
    "providers",
    "rules",
    "skills",
    "styles",
    "workflows-list",
    "workflows-visual",
    "workflows-governance",
] as const;

export type WorkshopFixture = (typeof WORKSHOP_FIXTURES)[number];

export interface WorkshopFixtureViewProps {
    fixture: WorkshopFixture;
}

const CLI_GROUPS = [
    {
        id: "source-control",
        label: "Source control",
        tools: ["git", "gh", "delta"],
    },
    { id: "search-files", label: "Search & files", tools: ["rg", "fd", "jq"] },
    { id: "rust", label: "Rust", tools: ["cargo", "rustfmt", "clippy"] },
    { id: "database", label: "Database", tools: ["psql", "sqlx"] },
    { id: "network", label: "Web & network", tools: ["curl", "httpie"] },
] as const;

function Toggle(props: {
    on: boolean;
    onClick: () => void;
    label: string;
    testId?: string;
}) {
    return (
        <button
            type="button"
            class="ws-toggle"
            aria-label={props.label}
            aria-pressed={props.on}
            data-testid={props.testId}
            data-on={props.on ? "true" : "false"}
            onClick={props.onClick}
        >
            <span />
        </button>
    );
}

function ExtensionFixture(props: {
    fixture: Extract<
        WorkshopFixture,
        | "commands"
        | "context"
        | "hooks"
        | "linters"
        | "output-styles"
        | "rules"
        | "skills"
        | "styles"
    >;
}) {
    const type: ExtensionEditorType =
        props.fixture === "output-styles" ? "styles" : props.fixture;
    return (
        <div data-testid={`workshop-${props.fixture}`} class="ws-fixture">
            <FixtureNotice
                surface={`Workshop · ${props.fixture}`}
                command='invoke("extension_inventory")'
            />
            <ExtensionEditor type={type} />
        </div>
    );
}

function AgentsFixture() {
    return (
        <div data-testid="workshop-agents" class="ws-fixture agents-screen">
            <FixtureNotice
                surface="Workshop · Agents"
                command='invoke("agent_defs_list")'
            />
            <ExtensionEditor type="agents" />
            <footer
                class="agents-handoff-footer"
                data-testid="agents-handoff-summary"
            >
                <strong>Stuck run · handoff ready</strong>
                <span>
                    3 iterations without progress · 2 attempted · 1 artifact
                    reference
                </span>
                <Button variant="secondary">Open handoff</Button>
            </footer>
        </div>
    );
}

function CliFixture() {
    const [enabled, setEnabled] = createSignal<Record<string, boolean>>({
        git: true,
        gh: false,
        delta: true,
        rg: true,
        fd: true,
        jq: true,
        cargo: true,
        rustfmt: true,
        clippy: true,
        psql: false,
        sqlx: true,
        curl: true,
        httpie: false,
    });
    const toggleGroup = (group: (typeof CLI_GROUPS)[number]) => {
        const all = group.tools.every((tool) => enabled()[tool]);
        setEnabled((current) => ({
            ...current,
            ...Object.fromEntries(group.tools.map((tool) => [tool, !all])),
        }));
    };
    const groupState = (group: (typeof CLI_GROUPS)[number]) => {
        const count = group.tools.filter((tool) => enabled()[tool]).length;
        return count === 0
            ? "off"
            : count === group.tools.length
              ? "on"
              : "mixed";
    };
    return (
        <div class="ws-fixture ws-cli" data-testid="workshop-cli">
            <FixtureNotice
                surface="Workshop · CLI"
                command='invoke("cli_tools_list")'
            />
            <header class="ws-fixture-head">
                <div>
                    <h1>CLI</h1>
                    <p>
                        Enabled tools are baked into the image once, never
                        installed per run.
                    </p>
                </div>
                <div class="ws-actions">
                    <Input placeholder="Filter tools" />
                    <Button variant="primary">Upload a CLI</Button>
                </div>
            </header>
            <div class="ws-fixture-columns">
                <main>
                    <p class="ws-intro">
                        A tool switched on here can be added to any project. Off
                        means it is not in the image at all.
                    </p>
                    <For each={CLI_GROUPS}>
                        {(group) => (
                            <section
                                class="ws-tool-group"
                                data-testid={`cli-group-${group.id}`}
                                data-state={groupState(group)}
                            >
                                <header>
                                    <span>{group.label}</span>
                                    <code>
                                        {
                                            group.tools.filter(
                                                (tool) => enabled()[tool],
                                            ).length
                                        }{" "}
                                        / {group.tools.length}
                                    </code>
                                    <Toggle
                                        label={`Toggle ${group.label}`}
                                        testId={`cli-group-toggle-${group.id}`}
                                        on={groupState(group) === "on"}
                                        onClick={() => toggleGroup(group)}
                                    />
                                </header>
                                <For each={group.tools}>
                                    {(tool) => (
                                        <div class="ws-tool">
                                            <code>{tool}</code>
                                            <span>
                                                {tool === "git"
                                                    ? "version control"
                                                    : "available in enabled images"}
                                            </span>
                                            <Toggle
                                                label={`Toggle ${tool}`}
                                                on={enabled()[tool]}
                                                onClick={() =>
                                                    setEnabled((current) => ({
                                                        ...current,
                                                        [tool]: !current[tool],
                                                    }))
                                                }
                                            />
                                        </div>
                                    )}
                                </For>
                            </section>
                        )}
                    </For>
                    <section class="ws-tool-group">
                        <header>
                            <span>Your own</span>
                            <span />
                        </header>
                        <div class="ws-tool">
                            <code>repo-audit</code>
                            <span>unsigned · inspect repository policy</span>
                            <Tag variant="outline">unsigned</Tag>
                        </div>
                        <div class="ws-drop-zone">
                            Drop a binary, a script, or a <code>tool.toml</code>
                            <small>
                                Install and verify fields are recorded before an
                                image build.
                            </small>
                        </div>
                        <div class="ws-install-fields">
                            <Input
                                mono
                                placeholder="install"
                                value="cargo install --git …"
                                readOnly
                            />
                            <Input
                                mono
                                placeholder="verify it landed"
                                value="<tool> --version"
                                readOnly
                            />
                        </div>
                        <p
                            id="cli-unsigned-note"
                            data-testid="cli-unsigned-note"
                        >
                            Unsigned tools are rejected for every write-capable
                            role; read-only roles only may inspect them. Sign it
                            before catalog admission.
                        </p>
                        <p data-testid="cli-upload-verification">
                            Rejected — a valid trusted Minisign signature is
                            required before catalog or image admission.
                        </p>
                    </section>
                </main>
                <aside class="ws-side-cards">
                    <section>
                        <h2>Image</h2>
                        <p>
                            Enabled tools are baked into the base image, not
                            installed per run.
                        </p>
                        <strong class="mono">1.9 GB</strong>
                        <small>rebuilt 2h ago</small>
                    </section>
                    <section>
                        <h2>Most reached for</h2>
                        <For
                            each={["git · 2,481", "rg · 1,920", "cargo · 864"]}
                        >
                            {(tool) => (
                                <div class="ws-usage">
                                    <span>{tool}</span>
                                    <i />
                                </div>
                            )}
                        </For>
                    </section>
                </aside>
            </div>
        </div>
    );
}

function ProvidersFixture() {
    const [selected, setSelected] = createSignal("Anthropic");
    const providers = ["Anthropic", "OpenRouter", "OpenAI", "Gemini", "Local"];
    return (
        <div class="ws-providers" data-testid="workshop-providers">
            <aside class="ws-provider-list">
                <header>
                    <h1>Providers</h1>
                    <p>
                        Credentials and the short list of models you actually
                        use.
                    </p>
                    <Button variant="primary">Add provider</Button>
                </header>
                <For each={providers}>
                    {(provider, index) => (
                        <button
                            type="button"
                            class="ws-provider-row"
                            aria-selected={selected() === provider}
                            onClick={() => setSelected(provider)}
                        >
                            <i
                                data-state={
                                    index() < 3
                                        ? "ok"
                                        : index() === 3
                                          ? "warn"
                                          : "off"
                                }
                            />
                            {provider}
                            <small>
                                {index() === 0
                                    ? "3 preferred · ok"
                                    : index() === 3
                                      ? "warn · verify soon"
                                      : "off · not configured"}
                            </small>
                        </button>
                    )}
                </For>
                <footer
                    id="provider-keychain-note"
                    data-testid="provider-keychain-note"
                >
                    Secrets live in the OS keychain. Locus stores the reference
                    and the model list, never the key.
                </footer>
            </aside>
            <main class="ws-provider-main">
                <FixtureNotice
                    surface="Workshop · Providers"
                    command='invoke("providers_list")'
                />
                <header class="ws-fixture-head">
                    <div>
                        <h1>{selected()}</h1>
                        <p class="mono">provider/{selected().toLowerCase()}</p>
                    </div>
                    <div class="ws-actions">
                        <Button variant="secondary">Test connection</Button>
                        <Button variant="primary">Save</Button>
                    </div>
                </header>
                <section>
                    <h2>Authentication</h2>
                    <div class="ws-settings-card">
                        <div>
                            <span>method</span>
                            <Segmented
                                label="Authentication method"
                                value="api-key"
                                onChange={() => undefined}
                                options={[
                                    { value: "oauth", label: "OAuth" },
                                    { value: "api-key", label: "API key" },
                                    { value: "none", label: "None" },
                                ]}
                            />
                        </div>
                        <div>
                            <span>API key</span>
                            <code
                                id="provider-secret"
                                data-testid="provider-secret"
                            >
                                ••••••••••••••••
                            </code>
                            <Button variant="ghost">Reveal</Button>
                            <Button variant="ghost">Replace</Button>
                            <small>keychain</small>
                        </div>
                        <div>
                            <span>base_url</span>
                            <Input
                                mono
                                value="https://api.anthropic.com"
                                readOnly
                            />
                            <small>optional override</small>
                        </div>
                        <p
                            id="provider-verification"
                            data-testid="provider-verification"
                        >
                            verified 11m ago · 327 models listed
                        </p>
                    </div>
                </section>
                <section>
                    <h2>Preferred models</h2>
                    <p>
                        Aliases are what every model selector shows from then
                        on, for every harness using this provider.
                    </p>
                    <div class="ws-model-table">
                        <div class="ws-model-head">
                            <span>Model</span>
                            <span>Alias</span>
                            <span>Context</span>
                            <span>In / out per M</span>
                            <span>In selector</span>
                        </div>
                        <For
                            each={[
                                [
                                    "claude-opus-4-6",
                                    "opus",
                                    "200k",
                                    "$15 / $75",
                                ],
                                [
                                    "claude-sonnet-4-5",
                                    "sonnet",
                                    "200k",
                                    "$3 / $15",
                                ],
                                [
                                    "claude-haiku-4-5",
                                    "haiku",
                                    "200k",
                                    "$1 / $5",
                                ],
                            ]}
                        >
                            {([id, alias, context, price]) => (
                                <div class="ws-model-row">
                                    <code>{id}</code>
                                    <strong
                                        id={`provider-model-alias-${alias}`}
                                        data-testid={`provider-model-alias-${alias}`}
                                    >
                                        {alias}
                                    </strong>
                                    <code>{context}</code>
                                    <code>{price}</code>
                                    <Toggle
                                        label={`Include ${alias}`}
                                        on={true}
                                        onClick={() => undefined}
                                    />
                                </div>
                            )}
                        </For>
                        <Input
                            class="ws-catalog-search"
                            placeholder="Search catalogue — 4 of 327 match"
                        />
                    </div>
                </section>
            </main>
            <aside class="ws-provider-preview">
                <h2>Selector preview</h2>
                <div
                    id="provider-selector-preview"
                    data-testid="provider-selector-preview"
                >
                    <strong>opus</strong>
                    <span>Anthropic · 200k context</span>
                </div>
                <h2>Used by</h2>
                <Tag variant="neutral">claude</Tag>
                <Tag variant="neutral">cursor</Tag>
                <h2>30-day spend</h2>
                <strong class="mono">$418.20</strong>
            </aside>
        </div>
    );
}

function HarnessesFixture() {
    return (
        <div data-testid="workshop-harnesses" class="ws-fixture">
            <FixtureNotice
                surface="Workshop · Harnesses"
                command='invoke("harness_registry_list")'
            />
            <ExtensionEditor type="harnesses" />
        </div>
    );
}

function WorkflowsListFixture() {
    return (
        <div class="ws-fixture" data-testid="workshop-workflows-list">
            <FixtureNotice
                surface="Workshop · Workflows"
                command='invoke("workflow_defs_list")'
            />
            <header class="ws-fixture-head">
                <div>
                    <h1>Workflows</h1>
                    <p>
                        Authored definitions are versioned before a run
                        evaluates them.
                    </p>
                </div>
                <Button variant="primary">New workflow</Button>
            </header>
            <section class="ws-settings-card">
                <article data-testid="workflow-list-published">
                    <strong>Release verification</strong>
                    <span>published · revision 4</span>
                    <small>author: Avery · edited 2m ago</small>
                </article>
                <article data-testid="workflow-list-draft">
                    <strong>Migration readiness</strong>
                    <span>draft · revision 1</span>
                    <small>author: Rowan · no runs yet</small>
                </article>
            </section>
        </div>
    );
}

function WorkflowsFixture(props: { governance: boolean }) {
    const [tab, setTab] = createSignal(
        props.governance ? "governance" : "visual",
    );
    return (
        <div
            class="ws-fixture ws-workflows"
            data-testid={`workshop-workflows-${props.governance ? "governance" : "visual"}`}
        >
            <Show when={props.governance}>
                <FixtureNotice
                    surface="Workshop · Workflow governance"
                    command='invoke("workflow_def")'
                />
            </Show>
            <header class="ws-fixture-head">
                <div>
                    <h1>Release verification</h1>
                    <p>saved 2s ago · authoring only</p>
                </div>
                <Segmented
                    label="Workflow authoring view"
                    value={tab()}
                    onChange={setTab}
                    options={[
                        { value: "visual", label: "Visual" },
                        { value: "governance", label: "Governance" },
                    ]}
                />
            </header>
            <Show when={tab() === "visual"} fallback={<Governance />}>
                <WorkflowView />
            </Show>
        </div>
    );
}

function Governance() {
    return (
        <div class="ws-governance">
            <section>
                <h2>Goal</h2>
                <p
                    id="workflow-governance-goal"
                    data-testid="workflow-governance-goal"
                >
                    Ship a release only when every required verification command
                    passes.
                </p>
            </section>
            <section>
                <h2>Guardrails</h2>
                <article>
                    <strong>Preserve branches</strong>
                    <p>Never work in main or master; stop at a PR boundary.</p>
                </article>
                <Button variant="ghost">Add a guardrail</Button>
            </section>
            <section
                id="workflow-success-criteria"
                data-testid="workflow-success-criteria"
            >
                <h2>Success criteria</h2>
                <div>
                    <Tag variant="neutral">command</Tag>
                    <code>pnpm test</code>
                    <span>checked by core</span>
                </div>
                <div>
                    <Tag variant="neutral">human</Tag>
                    <span>Release notes approved</span>
                    <span>checked by a gate</span>
                </div>
            </section>
        </div>
    );
}

export function WorkshopFixtureView(props: WorkshopFixtureViewProps) {
    if (props.fixture === "agents") return <AgentsFixture />;
    if (props.fixture === "cli") return <CliFixture />;
    if (props.fixture === "providers") return <ProvidersFixture />;
    if (props.fixture === "harnesses") return <HarnessesFixture />;
    if (props.fixture === "workflows-list") return <WorkflowsListFixture />;
    if (props.fixture === "workflows-visual")
        return <WorkflowsFixture governance={false} />;
    if (props.fixture === "workflows-governance")
        return <WorkflowsFixture governance />;
    return <ExtensionFixture fixture={props.fixture} />;
}

export default WorkshopFixtureView;
