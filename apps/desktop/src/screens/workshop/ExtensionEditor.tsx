import { createMemo, createSignal, For, onMount, Show } from "solid-js";
import { Button } from "../../ui/Button";
import { InlineError } from "../../ui/InlineError";
import { Input } from "../../ui/Input";
import { dataProvider } from "../../data/provider";
import { Tag } from "../../ui/Tag";
import {
    fetchHarnesses,
    useExtensionCounts,
    useHarnesses,
    useHarnessSummary,
} from "../../data/harnesses";
import type { ExtensionType, HarnessEntry } from "../../data/harnesses";
import {
    fetchExtensionHistory,
    fetchExtensions,
    saveExtension,
    type ExtensionRevision,
    type PersistedExtension,
} from "../../data/extensions";
import type { Envelope } from "../../data/envelope";
import "./workshop-fixtures.css";
import "./ExtensionEditor.css";

/**
 * Shared editor for all eight authored extension types. The explicit demo/test
 * host retains fixture-local behavior; the live host reads and saves Postgres rows.
 */
export type ExtensionEditorType =
    | Exclude<ExtensionType, "output-styles">
    | "styles"
    | "harnesses";
export const EXTENSION_EDITOR_TYPES: readonly ExtensionEditorType[] = [
    "skills",
    "rules",
    "context",
    "commands",
    "hooks",
    "styles",
    "linters",
    "agents",
    "harnesses",
];
export type FrontmatterFieldKind =
    | "text"
    | "select"
    | "number"
    | "toggle"
    | "chips";

const SELECT_NAMES = new Set([
    "harness",
    "provider",
    "model",
    "adapter",
    "workflow",
]);
const CHIP_NAMES = new Set([
    "tools",
    "roles",
    "skills",
    "rules",
    "providers",
    "events",
    "tags",
]);
const NUMBER_NAMES = new Set([
    "budget",
    "timeout",
    "threshold",
    "priority",
    "iterations",
    "tokens",
    "count",
]);
const TOGGLE_NAMES = new Set([
    "enabled",
    "active",
    "lazy",
    "required",
    "gated",
    "autorouting",
    "optional",
]);

/** One inference table is shared by every extension type. */
export function inferFrontmatterFieldKind(name: string): FrontmatterFieldKind {
    const key = name.trim().toLowerCase();
    if (
        SELECT_NAMES.has(key) ||
        key.endsWith("_harness") ||
        key.endsWith("_provider") ||
        key.endsWith("_model") ||
        key.endsWith("_effort")
    )
        return "select";
    if (CHIP_NAMES.has(key) || key.endsWith("_list") || key.endsWith("_ids"))
        return "chips";
    if (
        TOGGLE_NAMES.has(key) ||
        key.startsWith("is_") ||
        key.startsWith("has_") ||
        key.startsWith("use_")
    )
        return "toggle";
    if (
        key.endsWith("_tokens") ||
        key.endsWith("_ms") ||
        key.endsWith("_seconds") ||
        key.endsWith("_count") ||
        NUMBER_NAMES.has(key)
    )
        return "number";
    return "text";
}

/** Short alias for consumers that call this the field-kind inference table. */
export const inferFieldKind = inferFrontmatterFieldKind;
export const fieldKindFor = inferFrontmatterFieldKind;

const LABELS: Record<ExtensionEditorType, string> = {
    skills: "Skills",
    rules: "Rules",
    context: "Context",
    commands: "Commands",
    hooks: "Hooks",
    styles: "Styles",
    linters: "Linters",
    agents: "Agents",
    harnesses: "Harnesses",
};
const SINGULAR: Record<ExtensionEditorType, string> = {
    skills: "skill",
    rules: "rule",
    context: "context",
    commands: "command",
    hooks: "hook",
    styles: "style",
    linters: "linter",
    agents: "agent",
    harnesses: "harness",
};
const BLURBS: Record<ExtensionEditorType, string> = {
    skills: "Model-invocable procedures, loaded when needed.",
    rules: "Path-scoped instructions with one glob per rule.",
    context: "The one base file every project run pays for.",
    commands: "Argument-taking prompt templates invoked by name.",
    hooks: "Lifecycle actions with a threshold and safe failure mode.",
    styles: "How an agent writes back, selected per harness.",
    linters: "Checks and the reason each check exists.",
    agents: "Markdown plus a tool allowlist: the privilege set.",
    harnesses: "Adapters, providers, defaults, and routing bands.",
};
const ITEMS: Record<ExtensionEditorType, string[]> = {
    skills: ["verify-loop", "incident-response", "spec-decomposition"],
    rules: ["no-secrets", "rust-style", "desktop-patterns"],
    context: ["base.md"],
    commands: ["check-pr", "handoff", "release-notes"],
    hooks: ["session-start", "before-tool", "session-end"],
    styles: ["brief-bright-gone", "technical-review", "release-notes"],
    linters: ["no-secrets", "no-todo-comments", "typed-boundaries"],
    agents: ["builder", "reviewer", "researcher", "auditor"],
    harnesses: ["claude", "codex", "copilot", "gemini", "cursor"],
};
const FIELDS: Record<ExtensionEditorType, Array<[string, string]>> = {
    skills: [
        ["budget_tokens", "12000"],
        ["lazy", "true"],
        ["tags", "verification, release"],
    ],
    rules: [
        ["glob", "src/**"],
        ["priority", "20"],
        ["enabled", "true"],
    ],
    context: [
        ["budget_tokens", "4000"],
        ["project", "tapestry"],
        ["active", "true"],
    ],
    commands: [
        ["arguments", "path, issue"],
        ["harness", "claude"],
        ["enabled", "true"],
    ],
    hooks: [
        ["event", "before_tool"],
        ["timeout_ms", "5000"],
        ["on_error", "log, continue"],
    ],
    styles: [
        ["roles", "builder, reviewer"],
        ["active", "true"],
        ["priority", "10"],
    ],
    linters: [
        ["command", "pnpm lint"],
        ["violation", "warn"],
        ["enabled", "true"],
    ],
    agents: [
        ["harness", "claude"],
        ["model", "sonnet"],
        ["tools", "git, rg, cargo"],
    ],
    harnesses: [
        ["identifier", "claude"],
        ["adapter", "built-in · v3"],
        ["providers", "Anthropic, OpenRouter"],
        ["default_model", "opus"],
        ["default_effort", "high"],
    ],
};
const BANDS = [
    "xtra-low",
    "low",
    "medium",
    "high",
    "xtra-high",
    "max",
] as const;
const EFFORTS = ["low", "medium", "high", "xhigh"] as const;

function registryType(type: ExtensionEditorType): ExtensionType {
    if (type === "styles") return "output-styles";
    if (type === "harnesses") return "agents";
    return type;
}

function storageType(type: ExtensionEditorType): string {
    if (type === "harnesses") return "agents";
    return type;
}

function Materialization(props: { type: ExtensionEditorType }) {
    const summary = useHarnessSummary();
    const counts = useExtensionCounts();
    const harnesses = useHarnesses();
    const count = counts.find(
        (entry) => entry.type === registryType(props.type),
    );
    return (
        <aside class="extension-materialization" data-testid="materialization">
            <header>
                <h2>Materialization</h2>
                <Tag variant="neutral">registry-derived</Tag>
            </header>
            <div class="materialization-figures">
                <div data-testid="materialization-native">
                    <strong>{count?.native ?? 0}</strong>
                    <span>native</span>
                </div>
                <div data-testid="materialization-downgraded">
                    <strong>{count?.downgraded ?? 0}</strong>
                    <span>downgraded</span>
                </div>
            </div>
            <div
                class="materialization-harnesses"
                data-testid="materialization-harnesses"
            >
                <For each={harnesses}>
                    {(harness) => {
                        const entry = harness.extensions.find(
                            (item) => item.type === registryType(props.type),
                        );
                        return (
                            <div class="materialization-harness">
                                <span>{harness.name}</span>
                                <i
                                    data-native={
                                        entry?.weakerThanNative
                                            ? "false"
                                            : "true"
                                    }
                                    title={entry?.weakerThanNative ?? "native"}
                                />
                            </div>
                        );
                    }}
                </For>
            </div>
            <p
                class="materialization-explanation"
                data-testid="materialization-explanation"
            >
                {props.type === "skills"
                    ? "Downgraded skills inline their description into base context and lose lazy loading."
                    : props.type === "rules"
                      ? "Downgraded rules concatenate into base context and lose path scoping."
                      : "Downgrades name the mechanism loss; the registry decides the count."}
            </p>
            <p class="determinism-note" data-testid="determinism-note">
                Sorted order, no timestamps, no run id. The materialized tree is
                the prompt prefix, so an unstable one costs cache on every run
                that follows it.
            </p>
            <div class="version-history">
                <h3>Version history</h3>
                <p>v4 · edited 2h ago</p>
                <p>v3 · edited yesterday</p>
            </div>
            <small>
                {summary.harnesses} registered harnesses · {summary.entries}{" "}
                extension entries
            </small>
        </aside>
    );
}

function FieldControl(props: {
    name: string;
    value: string;
    onChange: (value: string) => void;
}) {
    const kind = inferFrontmatterFieldKind(props.name);
    if (kind === "toggle") {
        const enabled = () => props.value === "true";
        return (
            <button
                type="button"
                role="switch"
                aria-checked={enabled()}
                aria-label={props.name}
                data-testid={`frontmatter-control-${props.name}`}
                class="ws-toggle"
                data-on={enabled() ? "true" : "false"}
                onClick={() => props.onChange(enabled() ? "false" : "true")}
            >
                <span />
            </button>
        );
    }
    if (kind === "select") {
        const options = Array.from(
            new Set([
                props.value,
                ...(props.name === "default_effort"
                    ? EFFORTS
                    : props.name === "harness"
                      ? ["claude", "codex", "pi"]
                      : props.name === "provider"
                        ? ["anthropic", "openai", "openrouter"]
                        : []),
            ]),
        );
        return (
            <select class="input"
                aria-label={props.name}
                data-testid={`frontmatter-control-${props.name}`}
                value={props.value}
                onChange={(event) => props.onChange(event.currentTarget.value)}
            >
                <For each={options}>
                    {(option) => <option value={option}>{option}</option>}
                </For>
            </select>
        );
    }
    return (
        <Input
            type={kind === "number" ? "number" : "text"}
            value={props.value}
            aria-label={props.name}
            data-testid={`frontmatter-control-${props.name}`}
            onInput={(event) => props.onChange(event.currentTarget.value)}
        />
    );
}

function Frontmatter(props: {
    type: ExtensionEditorType;
    values: Record<string, string>;
    onChange: (name: string, value: string) => void;
}) {
    return (
        <section class="extension-frontmatter" data-testid="frontmatter">
            <header>
                <h2>Frontmatter</h2>
                <span>kind inferred from field name</span>
            </header>
            <div class="frontmatter-grid" role="table" aria-label="Frontmatter fields">
                <div class="frontmatter-header-row" role="row">
                    <div class="frontmatter-column-label" role="columnheader">Key</div>
                    <div class="frontmatter-column-label" role="columnheader">Value</div>
                    <div class="frontmatter-column-label" role="columnheader">Kind</div>
                </div>
                <For each={FIELDS[props.type]}>
                    {([name, value]) => (
                        <div
                            class="frontmatter-row"
                            role="row"
                            data-testid={`frontmatter-field-${name}`}
                        >
                            <div role="cell"><code>{name}</code></div>
                            <div role="cell">
                                <FieldControl
                                    name={name}
                                    value={props.values[name] ?? value}
                                    onChange={(next) => props.onChange(name, next)}
                                />
                            </div>
                            <div role="cell">
                                <Tag
                                    variant="neutral"
                                    data-testid={`frontmatter-kind-${name}`}
                                >
                                    {inferFrontmatterFieldKind(name)}
                                </Tag>
                            </div>
                        </div>
                    )}
                </For>
            </div>
        </section>
    );
}

function HarnessDetails(props: { onAddConfigKey: () => void }) {
    const [autorouting, setAutorouting] = createSignal(true);
    const [routingEfforts, setRoutingEfforts] = createSignal<string[]>(
        BANDS.map((_, index) => EFFORTS[Math.min(index, EFFORTS.length - 1)]),
    );
    return (
        <>
            <section
                class="extension-record"
                data-testid="harness-adapter-gate"
            >
                <div>
                    <strong>adapter</strong>
                    <span>built-in · v3</span>
                    <small>no adapter, no selection — anywhere</small>
                </div>
                <div>
                    <strong>identifier</strong>
                    <code>claude</code>
                </div>
            </section>
            <section class="extension-routing" data-testid="autorouting">
                <header>
                    <h2>Autorouting</h2>
                    <span>
                        six complexity bands · effort is one of four values
                    </span>
                    <button
                        type="button"
                        class="ws-toggle"
                        aria-label="Toggle autorouting"
                        aria-pressed={autorouting()}
                        data-testid="autorouting-toggle"
                        data-on={autorouting() ? "true" : "false"}
                        onClick={() => setAutorouting((value) => !value)}
                    >
                        <span />
                    </button>
                </header>
                <Show
                    when={autorouting()}
                    fallback={
                        <p data-testid="autoroute-disabled">
                            Off — every task uses the harness default model and
                            effort.
                        </p>
                    }
                >
                    <div class="routing-table">
                        <div>
                            <span>Complexity</span>
                            <span>Model</span>
                            <span>Effort</span>
                            <span>Approval</span>
                        </div>
                        <For each={BANDS}>
                            {(band, index) => (
                                <div data-testid={`autoroute-band-${band}`}>
                                    <strong>{band}</strong>
                                    <span>
                                        {index() === 4 ? "—" : "sonnet"}
                                    </span>
                                    <select class="input"
                                        aria-label={`${band} effort`}
                                        value={routingEfforts()[index()]}
                                        onChange={(event) =>
                                            setRoutingEfforts((current) =>
                                                current.map(
                                                    (effort, effortIndex) =>
                                                        effortIndex === index()
                                                            ? event
                                                                  .currentTarget
                                                                  .value
                                                            : effort,
                                                ),
                                            )
                                        }
                                    >
                                        <For each={EFFORTS}>
                                            {(effort) => (
                                                <option value={effort}>
                                                    {effort}
                                                </option>
                                            )}
                                        </For>
                                    </select>
                                    <span>{index() > 3 ? "✓" : "—"}</span>
                                </div>
                            )}
                        </For>
                    </div>
                    <p id="autoroute-fallback" data-testid="autoroute-fallback">
                        A band with no model set never receives work: the task
                        falls upward to the next band up.
                    </p>
                </Show>
            </section>
            <section class="extension-config" data-testid="adapter-config">
                <header>
                    <h2>Adapter config</h2>
                    <span>free-form keys the adapter reads</span>
                </header>
                <div>
                    <code>permission-mode</code>
                    <span>bypass</span>
                    <Tag variant="neutral">string</Tag>
                </div>
                <Button variant="ghost" onClick={props.onAddConfigKey}>
                    Add config key
                </Button>
            </section>
        </>
    );
}

function valueToString(value: unknown, fallback: string): string {
    if (value === undefined || value === null) return fallback;
    if (Array.isArray(value)) return value.map(String).join(", ");
    return String(value);
}

function draftValues(
    type: ExtensionEditorType,
    extension?: PersistedExtension,
): Record<string, string> {
    return Object.fromEntries(
        FIELDS[type].map(([name, fallback]) => [
            name,
            valueToString(extension?.frontmatter[name], fallback),
        ]),
    );
}

function LiveMaterialization(props: { type: ExtensionEditorType }) {
    const [registry, setRegistry] = createSignal<Envelope<HarnessEntry[]>>({
        status: "loading",
    });
    const registryTypeName = registryType(props.type);
    onMount(() => void fetchHarnesses().then(setRegistry));
    const harnesses = () => {
        const state = registry();
        return state.status === "ready" ? state.data : [];
    };
    const registryError = () => {
        const state = registry();
        return state.status === "failed" ? state.error.message : "";
    };
    const entries = () =>
        harnesses().flatMap((harness) =>
            harness.extensions.filter(
                (extension) => extension.type === registryTypeName,
            ),
        );
    return (
        <aside class="extension-materialization" data-testid="materialization">
            <header>
                <h2>Materialization</h2>
                <Tag variant="neutral">registry-derived</Tag>
            </header>
            <Show when={registry().status === "loading"}>
                <p data-testid="materialization-loading">Loading registry…</p>
            </Show>
            <Show when={registry().status === "failed"}>
                <p data-testid="materialization-error">{registryError()}</p>
            </Show>
            <Show when={registry().status === "ready"}>
                <div class="materialization-figures">
                    <div data-testid="materialization-native">
                        <strong>
                            {
                                entries().filter(
                                    (entry) => !entry.weakerThanNative,
                                ).length
                            }
                        </strong>
                        <span>native</span>
                    </div>
                    <div data-testid="materialization-downgraded">
                        <strong>
                            {
                                entries().filter(
                                    (entry) => entry.weakerThanNative,
                                ).length
                            }
                        </strong>
                        <span>downgraded</span>
                    </div>
                </div>
                <div
                    class="materialization-harnesses"
                    data-testid="materialization-harnesses"
                >
                    <For each={harnesses()}>
                        {(harness) => {
                            const entry = harness.extensions.find(
                                (item) => item.type === registryTypeName,
                            );
                            return (
                                <div class="materialization-harness">
                                    <span>{harness.name}</span>
                                    <i
                                        data-native={
                                            entry?.weakerThanNative
                                                ? "false"
                                                : "true"
                                        }
                                        title={
                                            entry?.weakerThanNative ?? "native"
                                        }
                                    />
                                </div>
                            );
                        }}
                    </For>
                </div>
            </Show>
            <p
                class="materialization-explanation"
                data-testid="materialization-explanation"
            >
                {props.type === "skills"
                    ? "Downgraded skills inline their description into base context and lose lazy loading."
                    : props.type === "rules"
                      ? "Downgraded rules concatenate into base context and lose path scoping."
                      : "Downgrades name the mechanism loss; the registry decides the count."}
            </p>
            <p class="determinism-note" data-testid="determinism-note">
                Sorted order, no timestamps, no run id. The materialized tree is
                the prompt prefix, so an unstable one costs cache on every run
                that follows it.
            </p>
            <small>{harnesses().length} registered harnesses</small>
        </aside>
    );
}

function LiveExtensionEditor(props: { type: ExtensionEditorType }) {
    const type = storageType(props.type);
    const label = LABELS[props.type];
    const [extensions, setExtensions] = createSignal<
        Envelope<PersistedExtension[]>
    >({
        status: "loading",
    });
    const [selectedId, setSelectedId] = createSignal<string>();
    const [draft, setDraft] = createSignal<{
        id?: string;
        name: string;
        frontmatter: Record<string, unknown>;
        body: string;
    }>();
    const [sortByName, setSortByName] = createSignal(false);
    const [historyOpen, setHistoryOpen] = createSignal(false);
    const [history, setHistory] = createSignal<Envelope<ExtensionRevision[]>>({
        status: "empty",
    });
    const [saved, setSaved] = createSignal(true);
    const [saveError, setSaveError] = createSignal<string>();
    const rows = () => {
        const state = extensions();
        return state.status === "ready" ? state.data : [];
    };
    const extensionError = () => {
        const state = extensions();
        return state.status === "failed" ? state.error.message : "";
    };
    const historyRows = () => {
        const state = history();
        return state.status === "ready" ? state.data : [];
    };
    const historyError = () => {
        const state = history();
        return state.status === "failed" ? state.error.message : "";
    };
    const displayedRows = createMemo(() =>
        sortByName()
            ? [...rows()].sort((left, right) =>
                  left.name.localeCompare(right.name),
              )
            : rows(),
    );

    const choose = (extension: PersistedExtension) => {
        setSelectedId(extension.id);
        setDraft({
            id: extension.id,
            name: extension.name,
            frontmatter: { ...extension.frontmatter },
            body: extension.body,
        });
        setHistoryOpen(false);
        setSaved(true);
        setSaveError(undefined);
    };

    onMount(() => {
        void fetchExtensions(type).then((result) => {
            setExtensions(result);
            if (result.status === "ready" && result.data[0])
                choose(result.data[0]);
        });
    });

    const add = () => {
        setSelectedId(undefined);
        setDraft({
            name: `${SINGULAR[props.type]}-${rows().length + 1}`,
            frontmatter: draftValues(props.type),
            body: "",
        });
        setHistoryOpen(false);
        setSaved(false);
        setSaveError(undefined);
    };
    const updateDraft = (
        patch: Partial<NonNullable<ReturnType<typeof draft>>>,
    ) => {
        setDraft((current) => (current ? { ...current, ...patch } : current));
        setSaved(false);
    };
    const updateField = (name: string, value: string) => {
        updateDraft({
            frontmatter: {
                ...(draft()?.frontmatter ?? {}),
                [name]: value,
            },
        });
    };
    const save = async () => {
        const current = draft();
        if (!current?.name.trim()) {
            setSaveError("Extension name is required.");
            return;
        }
        setSaveError(undefined);
        const result = await saveExtension({
            id: current.id,
            extensionType: type,
            name: current.name,
            frontmatter: current.frontmatter,
            body: current.body,
        });
        if (result.status !== "ready") {
            setSaveError(
                result.status === "failed"
                    ? result.error.message
                    : "Extension was not saved.",
            );
            return;
        }
        setExtensions({
            status: "ready",
            data: [
                ...rows().filter(
                    (extension) => extension.id !== result.data.id,
                ),
                result.data,
            ],
        });
        choose(result.data);
    };
    const toggleHistory = async () => {
        const id = selectedId();
        setHistoryOpen((open) => !open);
        if (!id || historyOpen()) return;
        setHistory({ status: "loading" });
        setHistory(await fetchExtensionHistory(id));
    };

    return (
        <div
            class="extension-editor"
            data-testid="extension-editor"
            data-extension-type={props.type}
            data-live-state="ready"
        >
            <aside
                class="extension-list-rail"
                data-testid="extension-list-rail"
            >
                <header>
                    <div class="extension-rail-icon" aria-hidden="true">
                        ✦
                    </div>
                    <h1>{label}</h1>
                    <span data-testid="extension-total">{rows().length}</span>
                    <p>{BLURBS[props.type]}</p>
                    <Button
                        variant="primary"
                        data-testid="extension-new"
                        onClick={add}
                    >
                        New {SINGULAR[props.type]}
                    </Button>
                </header>
                <div class="extension-sort">
                    <span>Items</span>
                    <button
                        type="button"
                        onClick={() => setSortByName((value) => !value)}
                    >
                        Sort: {sortByName() ? "name" : "recently edited"}
                    </button>
                </div>
                <Show when={extensions().status === "loading"}>
                    <p data-testid="extension-loading">Loading extensions…</p>
                </Show>
                <Show when={extensions().status === "failed"}>
                    <InlineError
                        cause={extensionError()}
                        next="Retry the extension store connection."
                    />
                </Show>
                <Show when={extensions().status === "empty"}>
                    <p data-testid="extension-empty">
                        No {label.toLowerCase()} are persisted.
                    </p>
                </Show>
                <For each={displayedRows()}>
                    {(extension) => (
                        <button
                            type="button"
                            class="extension-item"
                            data-testid={`extension-item-${extension.name}`}
                            aria-selected={
                                selectedId() === extension.id ? "true" : "false"
                            }
                            onClick={() => choose(extension)}
                        >
                            <span>{extension.name}</span>
                            <small>v{extension.version}</small>
                        </button>
                    )}
                </For>
                <footer>
                    Persisted in Postgres; revisions are retained for History.
                </footer>
            </aside>
            <Show
                when={draft()}
                fallback={
                    <main class="extension-editor-main">
                        <p>Select an extension or create a new one.</p>
                    </main>
                }
            >
                {(current) => (
                    <main class="extension-editor-main">
                        <header class="extension-editor-head">
                            <div>
                                <Input
                                    value={current().name}
                                    aria-label="Extension name"
                                    data-testid="extension-name"
                                    onInput={(event) =>
                                        updateDraft({
                                            name: event.currentTarget.value,
                                        })
                                    }
                                />
                                <p>
                                    {label} ·{" "}
                                    {selectedId()
                                        ? `version ${rows().find((extension) => extension.id === selectedId())?.version ?? 1}`
                                        : "new"}{" "}
                                    · {saved() ? "saved" : "unsaved changes"}
                                </p>
                            </div>
                            <div class="ws-actions">
                                <Button
                                    variant="secondary"
                                    data-testid="extension-history"
                                    onClick={toggleHistory}
                                >
                                    History
                                </Button>
                                <Button
                                    variant="primary"
                                    data-testid="extension-save"
                                    onClick={save}
                                >
                                    Save
                                </Button>
                            </div>
                        </header>
                        <Show when={saveError()}>
                            <InlineError
                                cause={saveError()!}
                                next="Correct the extension and save again."
                            />
                        </Show>
                        <Frontmatter
                            type={props.type}
                            values={draftValues(props.type, {
                                ...current(),
                                id: current().id ?? "",
                                extensionType: type,
                                version: 1,
                                updatedAt: "",
                            })}
                            onChange={updateField}
                        />
                        <Show when={historyOpen()}>
                            <section
                                class="extension-history"
                                data-testid="extension-history-panel"
                            >
                                <Show when={history().status === "loading"}>
                                    Loading history…
                                </Show>
                                <Show when={history().status === "failed"}>
                                    {historyError()}
                                </Show>
                                <For each={historyRows()}>
                                    {(revision) => (
                                        <p>
                                            v{revision.version} ·{" "}
                                            {revision.createdAt}
                                        </p>
                                    )}
                                </For>
                                <Show when={history().status === "empty"}>
                                    No revisions yet.
                                </Show>
                            </section>
                        </Show>
                        <section
                            class="extension-body"
                            data-testid="extension-body"
                        >
                            <header>
                                <h2>Rendered file body</h2>
                                <Tag variant="neutral">markdown</Tag>
                            </header>
                            <textarea class="input"
                                value={current().body}
                                aria-label="Extension body"
                                data-testid="extension-body-input"
                                onInput={(event) =>
                                    updateDraft({
                                        body: event.currentTarget.value,
                                    })
                                }
                            />
                        </section>
                    </main>
                )}
            </Show>
            <LiveMaterialization type={props.type} />
        </div>
    );
}

export function ExtensionEditor(props: { type: ExtensionEditorType }) {
    if (dataProvider().kind === "live") {
        if (props.type === "harnesses")
            return <LiveExtensionEditor type="agents" />;
        return <LiveExtensionEditor type={props.type} />;
    }
    const label = LABELS[props.type];
    const [items, setItems] = createSignal([...ITEMS[props.type]]);
    const [selectedItem, setSelectedItem] = createSignal(items()[0] ?? "");
    const [sortByName, setSortByName] = createSignal(false);
    const [historyOpen, setHistoryOpen] = createSignal(false);
    const [saved, setSaved] = createSignal(true);
    const [values, setValues] = createSignal<Record<string, string>>(
        Object.fromEntries(FIELDS[props.type]),
    );
    const [configKeys, setConfigKeys] = createSignal<string[]>([]);
    const displayedItems = createMemo(() =>
        sortByName() ? [...items()].sort() : items(),
    );
    const updateField = (name: string, value: string) => {
        setValues((current) => ({ ...current, [name]: value }));
        setSaved(false);
    };
    const addItem = () => {
        const name = `${SINGULAR[props.type]}-${items().length + 1}`;
        setItems((current) => [...current, name]);
        setSelectedItem(name);
        setSaved(false);
    };
    const addConfigKey = () => {
        const name = `config-${configKeys().length + 1}`;
        setConfigKeys((current) => [...current, name]);
        setSaved(false);
    };
    return (
        <div
            class="extension-editor"
            data-testid="extension-editor"
            data-extension-type={props.type}
        >
            <aside
                class="extension-list-rail"
                data-testid="extension-list-rail"
            >
                <header>
                    <div class="extension-rail-icon" aria-hidden="true">
                        ✦
                    </div>
                    <h1>{label}</h1>
                    <span data-testid="extension-total">{items().length}</span>
                    <p>{BLURBS[props.type]}</p>
                    <Button
                        variant="primary"
                        data-testid="extension-new"
                        onClick={addItem}
                    >
                        New {SINGULAR[props.type]}
                    </Button>
                </header>
                <div class="extension-sort">
                    <span>Items</span>
                    <button
                        type="button"
                        onClick={() => setSortByName((current) => !current)}
                    >
                        Sort: {sortByName() ? "name" : "recently edited"}
                    </button>
                </div>
                <For each={displayedItems()}>
                    {(item) => (
                        <button
                            type="button"
                            class="extension-item"
                            data-testid={`extension-item-${item}`}
                            aria-selected={
                                selectedItem() === item ? "true" : "false"
                            }
                            onClick={() => setSelectedItem(item)}
                        >
                            <span>{item}</span>
                            <small>
                                v{Math.max(1, 4 - items().indexOf(item))}
                            </small>
                        </button>
                    )}
                </For>
                <footer>
                    One directory per {SINGULAR[props.type]}, entry point{" "}
                    {props.type === "skills"
                        ? "SKILL.md"
                        : `${SINGULAR[props.type]}.md`}
                    .
                </footer>
            </aside>
            <main class="extension-editor-main">
                <header class="extension-editor-head">
                    <div>
                        <h1>{selectedItem()}</h1>
                        <p>
                            {label} · version 4 ·{" "}
                            {saved() ? "saved" : "unsaved changes"}
                        </p>
                    </div>
                    <div class="ws-actions">
                        <Button
                            variant="secondary"
                            data-testid="extension-history"
                            onClick={() =>
                                setHistoryOpen((current) => !current)
                            }
                        >
                            History
                        </Button>
                        <Button
                            variant="primary"
                            data-testid="extension-save"
                            onClick={() => setSaved(true)}
                        >
                            Save
                        </Button>
                    </div>
                </header>
                <Frontmatter
                    type={props.type}
                    values={values()}
                    onChange={updateField}
                />
                <Show when={historyOpen()}>
                    <section
                        class="extension-history"
                        data-testid="extension-history-panel"
                    >
                        v4 · current draft
                        <br />
                        v3 · edited yesterday
                    </section>
                </Show>
                <Show when={props.type === "harnesses"}>
                    <HarnessDetails onAddConfigKey={addConfigKey} />
                </Show>
                <Show
                    when={
                        props.type !== "harnesses" && props.type !== "linters"
                    }
                >
                    <section
                        class="extension-segment"
                        data-testid="extension-segment"
                    >
                        <h2>Loading</h2>
                        <Tag variant="neutral">on demand</Tag>
                    </section>
                </Show>
                <Show when={props.type === "agents"}>
                    <section
                        class="extension-checklist"
                        data-testid="extension-checklist"
                    >
                        <h2>Tool allowlist</h2>
                        <p>The allowlist is the privilege set.</p>
                        <label>
                            <input type="checkbox" checked /> git
                        </label>
                        <label>
                            <input type="checkbox" checked /> rg
                        </label>
                    </section>
                </Show>
                <Show when={configKeys().length > 0}>
                    <section
                        class="extension-config-keys"
                        data-testid="extension-config-keys"
                    >
                        <h2>Added config keys</h2>
                        <For each={configKeys()}>
                            {(key) => <code>{key}</code>}
                        </For>
                    </section>
                </Show>
                <section class="extension-body" data-testid="extension-body">
                    <header>
                        <h2>Rendered file body</h2>
                        <Tag variant="neutral">markdown</Tag>
                    </header>
                    <pre>
                        Instructions authored once in Locus and materialized
                        fresh for each run.
                    </pre>
                </section>
            </main>
            <Show when={props.type !== "harnesses"}>
                <Materialization type={props.type} />
            </Show>
        </div>
    );
}

export default ExtensionEditor;
