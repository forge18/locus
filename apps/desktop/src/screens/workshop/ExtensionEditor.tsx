import { createSignal, For, Show } from "solid-js";
import { Button } from "../../ui/Button";
import { Input } from "../../ui/Input";
import { Tag } from "../../ui/Tag";
import {
    useExtensionCounts,
    useHarnesses,
    useHarnessSummary,
} from "../../data/harnesses";
import type { ExtensionType } from "../../fixtures/generated/harnesses";
import "./workshop-fixtures.css";
import "./ExtensionEditor.css";

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

function Frontmatter(props: { type: ExtensionEditorType }) {
    return (
        <section class="extension-frontmatter" data-testid="frontmatter">
            <header>
                <h2>Frontmatter</h2>
                <span>kind inferred from field name</span>
            </header>
            <div class="frontmatter-grid">
                <div class="frontmatter-column-label">Key</div>
                <div class="frontmatter-column-label">Value</div>
                <div class="frontmatter-column-label">Kind</div>
                <For each={FIELDS[props.type]}>
                    {([name, value]) => (
                        <div
                            class="frontmatter-row"
                            data-testid={`frontmatter-field-${name}`}
                        >
                            <code>{name}</code>
                            <Show
                                when={name === "default_effort"}
                                fallback={<Input value={value} readOnly />}
                            >
                                <select
                                    aria-label="Default effort"
                                    value={value}
                                >
                                    {EFFORTS.map((effort) => (
                                        <option value={effort}>{effort}</option>
                                    ))}
                                </select>
                            </Show>
                            <Tag
                                variant="neutral"
                                data-testid={`frontmatter-kind-${name}`}
                            >
                                {inferFrontmatterFieldKind(name)}
                            </Tag>
                        </div>
                    )}
                </For>
            </div>
        </section>
    );
}

function HarnessDetails() {
    const [autorouting, setAutorouting] = createSignal(true);
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
                                    <select
                                        aria-label={`${band} effort`}
                                        value={EFFORTS[Math.min(index(), 3)]}
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
                <Button variant="ghost">Add config key</Button>
            </section>
        </>
    );
}

export function ExtensionEditor(props: { type: ExtensionEditorType }) {
    const label = LABELS[props.type];
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
                    <span data-testid="extension-total">
                        {ITEMS[props.type].length}
                    </span>
                    <p>{BLURBS[props.type]}</p>
                    <Button variant="primary" data-testid="extension-new">
                        New {SINGULAR[props.type]}
                    </Button>
                </header>
                <div class="extension-sort">
                    <span>Items</span>
                    <button type="button">Sort: recently edited</button>
                </div>
                <For each={ITEMS[props.type]}>
                    {(item, index) => (
                        <button
                            type="button"
                            class="extension-item"
                            data-testid={`extension-item-${item}`}
                            aria-selected={index() === 0 ? "true" : "false"}
                        >
                            <span>{item}</span>
                            <small>v{4 - index()}</small>
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
                        <h1>{ITEMS[props.type][0]}</h1>
                        <p>{label} · version 4 · last edited 2h ago</p>
                    </div>
                    <div class="ws-actions">
                        <Button
                            variant="secondary"
                            data-testid="extension-history"
                        >
                            History
                        </Button>
                        <Button variant="primary" data-testid="extension-save">
                            Save
                        </Button>
                    </div>
                </header>
                <Frontmatter type={props.type} />
                <Show when={props.type === "harnesses"}>
                    <HarnessDetails />
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
