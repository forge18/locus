import {
  For,
  Match,
  Show,
  Switch,
  createEffect,
  createMemo,
  createSignal,
  on,
} from "solid-js";
import type {
  AgentGateMode,
  AgentPaneElicitation,
  AgentPaneProps,
  AgentPaneSession,
  AgentPermissionPosture,
  AgentToolDisplay,
} from "./agent-panel-model";
import {
  formatTokens,
  typedElicitationValues,
  validateElicitationField,
} from "./agent-pane-utils";

export function AgentHeader(props: {
  session: AgentPaneSession;
  posture: AgentPermissionPosture;
  costVisible: boolean;
  contextOpen: boolean;
  researchOpen: boolean;
  showResearchControl: boolean;
  menuOpen: boolean;
  gateMode: AgentGateMode;
  toolCallsDisplay: AgentToolDisplay;
  onCostToggle: () => void;
  onGateModeChange: (mode: AgentGateMode) => void;
  onToolCallsDisplay: (display: AgentToolDisplay) => void;
  onContextToggle: () => void;
  onResearchToggle: () => void;
  onMenuToggle: () => void;
  onNewSession?: () => void;
  onCompact?: () => void;
  onClearContext?: () => void;
  onSessionRename?: (name: string) => void;
  onHarnessChange?: (harness: string) => void;
  onModelChange?: (model: string) => void;
  onEffortChange?: (effort: string) => void;
}) {
  const options = (provided: string[] | undefined, selected: string) =>
    provided?.length ? provided : [selected];
  return (
    <header class="agent-pane-header" data-testid="agent-panel-header">
      <div class="agent-pane-identity">
        <span class="agent-avatar" aria-hidden="true">
          {props.session.agent.slice(0, 2).toUpperCase()}
        </span>
        <div>
          <strong>{props.session.agent}</strong>
          <span>
            {props.session.project} · {props.session.harness}
          </span>
          <div class="agent-session-tags">
            <Show when={props.session.task}>
              {(task) => (
                <span data-testid="agent-task-chip">task · {task()}</span>
              )}
            </Show>
            <Show when={props.session.workflow}>
              {(workflow) => (
                <span data-testid="agent-workflow-chip">
                  workflow · {workflow()}
                </span>
              )}
            </Show>
          </div>
        </div>
      </div>
      <input
        class="agent-session-name"
        aria-label="Session name"
        value={props.session.name}
        onInput={(event) => props.onSessionRename?.(event.currentTarget.value)}
      />
      <div class="agent-pane-selectors" aria-label="Session configuration">
        <label>
          <span class="sr-only">Harness</span>
          <select class="input"
            aria-label="Harness"
            value={props.session.harness}
            onChange={(event) =>
              props.onHarnessChange?.(event.currentTarget.value)
            }
          >
            <For
              each={options(
                props.session.harnessOptions,
                props.session.harness,
              )}
            >
              {(option) => <option value={option}>{option}</option>}
            </For>
          </select>
        </label>
        <label>
          <span class="sr-only">Model</span>
          <select class="input"
            aria-label="Model"
            value={props.session.model}
            onChange={(event) =>
              props.onModelChange?.(event.currentTarget.value)
            }
          >
            <For
              each={options(props.session.modelOptions, props.session.model)}
            >
              {(option) => <option value={option}>{option}</option>}
            </For>
          </select>
        </label>
        <label>
          <span class="sr-only">Effort</span>
          <select class="input"
            aria-label="Effort"
            value={props.session.effort}
            onChange={(event) =>
              props.onEffortChange?.(event.currentTarget.value)
            }
          >
            <For
              each={options(props.session.effortOptions, props.session.effort)}
            >
              {(option) => <option value={option}>{option}</option>}
            </For>
          </select>
        </label>
      </div>
      <Show when={props.posture === "gated"}>
        <div
          class="agent-gate-toggle"
          role="group"
          aria-label="Permission gate mode"
        >
          <button
            type="button"
            aria-pressed={props.gateMode === "manual"}
            onClick={() => props.onGateModeChange("manual")}
          >
            Manual
          </button>
          <button
            type="button"
            aria-pressed={props.gateMode === "auto"}
            onClick={() => props.onGateModeChange("auto")}
          >
            Auto
          </button>
        </div>
      </Show>
      <button
        type="button"
        class="agent-context-chip"
        aria-expanded={props.contextOpen}
        data-testid="agent-context-toggle"
        onClick={props.onContextToggle}
      >
        <Show when={props.session.context} fallback="context unknown">
          {(context) =>
            `${formatTokens(context().used)} / ${formatTokens(context().total)}`
          }
        </Show>
      </button>
      <button
        type="button"
        class="agent-header-action"
        aria-pressed={props.costVisible}
        data-testid="agent-cost-toggle"
        onClick={props.onCostToggle}
      >
        {props.costVisible ? (props.session.cost ?? "cost unknown") : "cost"}
      </button>
      <Show when={props.showResearchControl}>
        <button
          type="button"
          class="agent-header-action"
          aria-pressed={props.researchOpen}
          data-testid="agent-research-toggle"
          onClick={props.onResearchToggle}
        >
          {props.researchOpen ? "Close research" : "Research"}
        </button>
      </Show>
      <div class="agent-overflow-wrap">
        <button
          type="button"
          class="agent-header-action agent-overflow"
          aria-expanded={props.menuOpen}
          aria-label="Session actions"
          data-testid="agent-overflow-toggle"
          onClick={props.onMenuToggle}
        >
          ···
        </button>
        <Show when={props.menuOpen}>
          <div class="agent-overflow-menu" role="menu">
            <button type="button" role="menuitem" onClick={props.onNewSession}>
              New session
            </button>
            <button type="button" role="menuitem" onClick={props.onCompact}>
              Compact context
            </button>
            <button
              type="button"
              role="menuitem"
              onClick={props.onClearContext}
            >
              Clear context
            </button>
            <span class="agent-menu-label">Tool calls</span>
            <button
              type="button"
              role="menuitemradio"
              aria-checked={props.toolCallsDisplay === "expanded"}
              onClick={() => props.onToolCallsDisplay("expanded")}
            >
              Expanded
            </button>
            <button
              type="button"
              role="menuitemradio"
              aria-checked={props.toolCallsDisplay === "collapsed"}
              onClick={() => props.onToolCallsDisplay("collapsed")}
            >
              Collapsed
            </button>
            <button
              type="button"
              role="menuitemradio"
              aria-checked={props.toolCallsDisplay === "hidden"}
              onClick={() => props.onToolCallsDisplay("hidden")}
            >
              Hidden
            </button>
          </div>
        </Show>
      </div>
      <span class={`agent-permission-badge agent-permission-${props.posture}`}>
        {props.posture}
      </span>
    </header>
  );
}

export function ElicitationCard(props: {
  elicitation: AgentPaneElicitation;
  minimized: boolean;
  onToggle: () => void;
  onAccept?: AgentPaneProps["onAcceptElicitation"];
  onDecline?: AgentPaneProps["onDeclineElicitation"];
  onCancel?: AgentPaneProps["onCancelElicitation"];
}) {
  const initialValues = () =>
    Object.fromEntries(
      props.elicitation.fields.map((field) => [
        field.id,
        String(field.defaultValue ?? ""),
      ]),
    );
  const [values, setValues] = createSignal<Record<string, string>>(
    initialValues(),
  );
  const [error, setError] = createSignal<string | null>(null);
  createEffect(
    on(
      () => props.elicitation.id,
      () => {
        setValues(initialValues());
        setError(null);
      },
      { defer: false },
    ),
  );
  const suggestionsFor = (field: AgentPaneElicitation["fields"][number]) =>
    [
      ...(field.suggestions ?? []),
      ...(props.elicitation.history ?? [])
        .map((item) => item[field.id])
        .filter(Boolean),
    ].filter((value, index, all) => all.indexOf(value) === index);
  const accept = () => {
    if (props.elicitation.mode === "url")
      return props.onAccept?.(props.elicitation, {});
    const invalid = props.elicitation.fields
      .map((field) => validateElicitationField(field, values()[field.id] ?? ""))
      .find(Boolean);
    if (invalid) return setError(invalid);
    setError(null);
    props.onAccept?.(
      props.elicitation,
      typedElicitationValues(props.elicitation, values()),
    );
  };
  return (
    <article
      class="agent-blocker agent-elicitation"
      data-testid="agent-elicitation"
      data-blocker-minimized={props.minimized}
    >
      <header>
        <span class="agent-card-kind">elicitation</span>
        <h2>{props.elicitation.title}</h2>
        <button
          type="button"
          aria-expanded={!props.minimized}
          onClick={props.onToggle}
        >
          {props.minimized ? "Restore" : "Minimize"}
        </button>
      </header>
      <Show when={!props.minimized}>
        <p>{props.elicitation.detail}</p>
        <Show
          when={props.elicitation.mode !== "url"}
          fallback={
            <p class="agent-url-consent">
              You will open a URL in your browser. Credentials never enter this
              panel.
            </p>
          }
        >
          <p class="agent-elicitation-review-note">
            Review the values before sending.
          </p>
          <div class="agent-elicitation-fields">
            <For each={props.elicitation.fields}>
              {(field) => {
                const suggestions = suggestionsFor(field);
                const listId = `agent-elicitation-${props.elicitation.id}-${field.id}`;
                return (
                  <label>
                    {field.label}
                    <Switch>
                      <Match when={field.type === "enum"}>
                        <select class="input"
                          aria-label={field.label}
                          value={values()[field.id]}
                          onChange={(event) =>
                            setValues((current) => ({
                              ...current,
                              [field.id]: event.currentTarget.value,
                            }))
                          }
                        >
                          <option value="">Choose…</option>
                          <For each={field.options ?? []}>
                            {(option) => (
                              <option value={option}>{option}</option>
                            )}
                          </For>
                        </select>
                      </Match>
                      <Match when={field.type === "boolean"}>
                        <input
                          aria-label={field.label}
                          type="checkbox"
                          checked={values()[field.id] === "true"}
                          onChange={(event) =>
                            setValues((current) => ({
                              ...current,
                              [field.id]: String(event.currentTarget.checked),
                            }))
                          }
                        />
                      </Match>
                      <Match when={true}>
                        <input
                          aria-label={field.label}
                          type={
                            field.type === "number" || field.type === "integer"
                              ? "number"
                              : "text"
                          }
                          value={values()[field.id]}
                          list={suggestions.length ? listId : undefined}
                          min={field.minimum}
                          max={field.maximum}
                          minLength={field.minLength}
                          pattern={field.pattern}
                          onInput={(event) =>
                            setValues((current) => ({
                              ...current,
                              [field.id]: event.currentTarget.value,
                            }))
                          }
                        />
                        <Show when={suggestions.length}>
                          <datalist id={listId}>
                            <For each={suggestions}>
                              {(suggestion) => <option value={suggestion} />}
                            </For>
                          </datalist>
                        </Show>
                      </Match>
                    </Switch>
                  </label>
                );
              }}
            </For>
          </div>
        </Show>
        <Show when={error()}>
          {(message) => (
            <p class="agent-form-error" role="alert">
              {message()}
            </p>
          )}
        </Show>
        <div class="agent-card-actions">
          <button type="button" class="agent-action-approve" onClick={accept}>
            Accept
          </button>
          <button
            type="button"
            onClick={() => props.onDecline?.(props.elicitation)}
          >
            Decline
          </button>
          <button
            type="button"
            onClick={() => props.onCancel?.(props.elicitation)}
          >
            Cancel
          </button>
        </div>
      </Show>
    </article>
  );
}

interface SlashCommand {
  input: string;
  action: string;
  label: string;
}

export function Composer(props: {
  running: boolean;
  value: string;
  onValue: (value: string) => void;
  onSend?: (prompt: string) => void;
  onQueue?: (prompt: string) => void;
  onStop?: () => void;
  onSlashCommand: (command: string) => void;
  mentionSuggestions?: string[];
}) {
  const commands: SlashCommand[] = [
    { input: "/new", action: "new-session", label: "New session" },
    { input: "/new-session", action: "new-session", label: "New session" },
    { input: "/compact", action: "compact", label: "Compact context" },
    { input: "/clear", action: "clear-context", label: "Clear context" },
    {
      input: "/clear-context",
      action: "clear-context",
      label: "Clear context",
    },
    { input: "/context", action: "context", label: "Show context" },
  ];
  const mentionSuggestions = () =>
    props.mentionSuggestions ?? ["@file", "@symbol", "@task"];
  const [selectedIndex, setSelectedIndex] = createSignal(0);
  const [queued, setQueued] = createSignal<string[]>([]);
  const suggestions = createMemo(() => {
    if (props.value.startsWith("/")) {
      return commands
        .filter((command) => command.input.startsWith(props.value.trim()))
        .map((command) => command.input)
        .filter((value, index, all) => all.indexOf(value) === index);
    }
    const mention = props.value.match(/(?:^|\s)(@[^\s]*)$/)?.[1];
    return mention
      ? mentionSuggestions().filter((suggestion) =>
          suggestion.startsWith(mention),
        )
      : [];
  });
  const choose = (suggestion: string) => {
    props.onValue(`${suggestion} `);
    setSelectedIndex(0);
  };
  const send = (event: SubmitEvent) => {
    event.preventDefault();
    const value = props.value.trim();
    if (suggestions().length && value !== suggestions()[selectedIndex()])
      return choose(suggestions()[selectedIndex()]);
    const command = commands.find((item) => item.input === value);
    if (command) {
      props.onSlashCommand(command.action);
      props.onValue("");
      return;
    }
    if (props.running) {
      if (!value) return props.onStop?.();
      setQueued((current) => [...current, value]);
      props.onQueue?.(value);
      if (!props.onQueue) props.onSend?.(value);
      props.onValue("");
      return;
    }
    if (value) {
      props.onSend?.(value);
      props.onValue("");
    }
  };
  const keyDown = (
    event: KeyboardEvent & { currentTarget: HTMLTextAreaElement },
  ) => {
    if (suggestions().length && event.key === "ArrowDown") {
      event.preventDefault();
      setSelectedIndex((current) => (current + 1) % suggestions().length);
      return;
    }
    if (suggestions().length && event.key === "ArrowUp") {
      event.preventDefault();
      setSelectedIndex(
        (current) =>
          (current - 1 + suggestions().length) % suggestions().length,
      );
      return;
    }
    if (suggestions().length && event.key === "Escape") {
      event.preventDefault();
      props.onValue("");
      return;
    }
    if (
      suggestions().length &&
      event.key === "Enter" &&
      !event.metaKey &&
      !event.ctrlKey
    ) {
      event.preventDefault();
      choose(suggestions()[selectedIndex()]);
      return;
    }
    if (event.key === "Enter" && (event.metaKey || event.ctrlKey))
      event.currentTarget.form?.requestSubmit();
  };
  return (
    <form class="agent-composer" data-testid="agent-composer" onSubmit={send}>
      <Show when={suggestions().length}>
        <div
          id="agent-composer-suggestions"
          class="agent-composer-suggestions"
          role="listbox"
          aria-label="Composer suggestions"
        >
          <For each={suggestions()}>
            {(suggestion, index) => (
              <button
                type="button"
                role="option"
                aria-selected={selectedIndex() === index()}
                onClick={() => choose(suggestion)}
              >
                {suggestion}
                <Show when={suggestion.startsWith("/")}>
                  <small>
                    {
                      commands.find((command) => command.input === suggestion)
                        ?.label
                    }
                  </small>
                </Show>
              </button>
            )}
          </For>
        </div>
      </Show>
      <div class="agent-composer-row">
        <textarea class="input"
          aria-label="Message agent"
          aria-controls="agent-composer-suggestions"
          placeholder="Message this session…"
          value={props.value}
          onInput={(event) => {
            props.onValue(event.currentTarget.value);
            setSelectedIndex(0);
          }}
          onKeyDown={keyDown}
        />
        <Show
          when={props.running}
          fallback={
            <button type="submit" class="agent-send-button">
              Send
            </button>
          }
        >
          <button
            type="button"
            class="agent-stop-button"
            onClick={() => props.onStop?.()}
          >
            Stop
          </button>
        </Show>
      </div>
      <Show when={queued().length}>
        <ol
          class="agent-queued-prompts"
          data-testid="agent-queued-prompts"
          aria-label="Queued prompts"
        >
          <For each={queued()}>{(prompt) => <li>{prompt}</li>}</For>
        </ol>
      </Show>
      <span class="agent-composer-hint">
        ⌘↵ send · / commands · @ files, symbols, tasks
      </span>
    </form>
  );
}
