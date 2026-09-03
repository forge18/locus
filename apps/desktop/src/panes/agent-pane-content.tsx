import { For, Match, Show, Switch, createSignal } from "solid-js";
import type { AgentEvent } from "../types/event";
import type {
  AgentPaneCitation,
  AgentPaneProps,
  AgentPermissionPosture,
  AgentThinkingDisplay,
  AgentToolDisplay,
} from "./agent-panel-model";
import {
  diffRows,
  eventCitations,
  eventDiff,
  eventPath,
  eventStatus,
  eventText,
  isSafeUrl,
  markdownBlocks,
  rawObject,
  rawString,
  safeJson,
  workspacePath,
} from "./agent-pane-utils";

function inlineFilePath(value: string): string | undefined {
  return value.includes("/") ? workspacePath(value) : undefined;
}

export function FileLink(props: {
  path: string;
  onOpenFile?: (path: string) => void;
}) {
  if (isSafeUrl(props.path)) {
    return (
      <a
        class="agent-file-link"
        data-source-url={props.path}
        href={props.path}
        target="_blank"
        rel="noreferrer"
      >
        {props.path}
      </a>
    );
  }
  const path = workspacePath(props.path);
  if (!path) {
    return (
      <span
        class="agent-file-link agent-file-link-rejected"
        data-file-path={props.path}
      >
        {props.path}
      </span>
    );
  }
  return (
    <button
      type="button"
      class="agent-file-link"
      data-file-path={path}
      data-open-pane="editor"
      onClick={() => props.onOpenFile?.(path)}
    >
      {path}
    </button>
  );
}

function InlineMarkdown(props: {
  text: string;
  onOpenFile?: (path: string) => void;
}) {
  const parts = props.text.split(
    /(`[^`]+`|!?\[[^\]]*\]\([^)]+\)|https?:\/\/\S+|(?:\.{0,2}\/|[\w.-]+\/)[\w./-]+\.[A-Za-z0-9]+)/g,
  );
  return (
    <>
      {parts.map((part) => {
        const code = part.match(/^`([^`]+)`$/);
        if (code) {
          const path = inlineFilePath(code[1]);
          return path ? (
            <FileLink path={path} onOpenFile={props.onOpenFile} />
          ) : (
            <code>{code[1]}</code>
          );
        }
        const path = inlineFilePath(part);
        if (path && path === part) return <FileLink path={path} onOpenFile={props.onOpenFile} />;
        const link = part.match(/^!?\[([^\]]*)\]\(([^)]+)\)$/);
        if (link && isSafeUrl(link[2])) {
          return part.startsWith("!") ? (
            <img src={link[2]} alt={link[1]} loading="lazy" />
          ) : (
            <a href={link[2]} target="_blank" rel="noreferrer">
              {link[1]}
            </a>
          );
        }
        return isSafeUrl(part) ? (
          <a href={part} target="_blank" rel="noreferrer">
            {part}
          </a>
        ) : (
          part
        );
      })}
    </>
  );
}

export function RichContent(props: {
  text: string;
  onOpenFile?: (path: string) => void;
}) {
  return (
    <div class="agent-rich-content">
      <For each={markdownBlocks(props.text)}>
        {(block) => (
          <Switch>
            <Match when={block.kind === "code"}>
              <pre class="agent-markdown-code" data-language={block.language}>
                {block.lines?.join("\n")}
              </pre>
            </Match>
            <Match when={block.kind === "table"}>
              <table>
                <thead>
                  <tr>
                    <For each={block.rows?.[0] ?? []}>
                      {(cell) => <th>{cell}</th>}
                    </For>
                  </tr>
                </thead>
                <tbody>
                  <For each={(block.rows ?? []).slice(1)}>
                    {(row) => (
                      <tr>
                        <For each={row}>
                          {(cell) => (
                            <td>
                              <InlineMarkdown
                                text={cell}
                                onOpenFile={props.onOpenFile}
                              />
                            </td>
                          )}
                        </For>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </Match>
            <Match when={block.kind === "image"}>
              <Show
                when={block.source && isSafeUrl(block.source)}
                fallback={
                  <FileLink
                    path={block.source ?? "image"}
                    onOpenFile={props.onOpenFile}
                  />
                }
              >
                <img
                  src={block.source}
                  alt={block.alt ?? "Agent image"}
                  loading="lazy"
                />
              </Show>
            </Match>
            <Match when={block.kind === "paragraph"}>
              <p>
                <InlineMarkdown
                  text={block.text ?? ""}
                  onOpenFile={props.onOpenFile}
                />
              </p>
            </Match>
          </Switch>
        )}
      </For>
    </div>
  );
}

export function ThinkingBlock(props: {
  event: AgentEvent;
  display: AgentThinkingDisplay;
  onDisplay: (display: AgentThinkingDisplay) => void;
}) {
  const summary =
    eventText(props.event).split(".")[0] || eventText(props.event);
  return (
    <article
      class="agent-stream-card agent-thinking"
      data-testid="agent-thinking-block"
    >
      <header>
        <span class="agent-card-kind">agent thought</span>
        <span class="agent-card-meta">summary-first</span>
        <div
          class="agent-disclosure-actions"
          role="group"
          aria-label="Thinking disclosure"
        >
          <button
            type="button"
            aria-pressed={props.display === "summary"}
            onClick={() => props.onDisplay("summary")}
          >
            Summary
          </button>
          <button
            type="button"
            aria-pressed={props.display === "full"}
            onClick={() => props.onDisplay("full")}
          >
            Full
          </button>
          <button
            type="button"
            aria-pressed={props.display === "hidden"}
            onClick={() => props.onDisplay("hidden")}
          >
            Hidden
          </button>
        </div>
      </header>
      <Show when={props.display !== "hidden"}>
        <p>{props.display === "summary" ? summary : eventText(props.event)}</p>
      </Show>
    </article>
  );
}

export function UserCard(props: {
  event: AgentEvent;
  onResubmit?: (event: AgentEvent, prompt: string) => void;
  onCopyPrompt?: (prompt: string) => void;
}) {
  const [expanded, setExpanded] = createSignal(false);
  const [editing, setEditing] = createSignal(false);
  const [draft, setDraft] = createSignal(eventText(props.event));
  return (
    <article
      class="agent-stream-card agent-user-card"
      data-testid="agent-user-card"
    >
      <header>
        <span class="agent-card-kind">you</span>
        <span class="agent-card-meta">turn {props.event.seq + 1}</span>
        <button
          type="button"
          aria-expanded={expanded()}
          onClick={() => setExpanded((value) => !value)}
        >
          {expanded() ? "Collapse" : "Expand"}
        </button>
      </header>
      <Show
        when={expanded()}
        fallback={<p class="agent-user-preview">{eventText(props.event)}</p>}
      >
        <Show
          when={!editing()}
          fallback={
            <textarea class="input"
              aria-label="Edit user prompt"
              value={draft()}
              onInput={(event) => setDraft(event.currentTarget.value)}
            />
          }
        >
          <p>{eventText(props.event)}</p>
        </Show>
        <div class="agent-card-actions">
          <button type="button" onClick={() => setEditing((value) => !value)}>
            {editing() ? "Preview" : "Edit"}
          </button>
          <button type="button" onClick={() => props.onCopyPrompt?.(draft())}>Copy</button>
          <button
            type="button"
            onClick={() => props.onResubmit?.(props.event, draft())}
          >
            Resubmit
          </button>
        </div>
      </Show>
    </article>
  );
}

export function AgentTurn(props: {
  event: AgentEvent;
  onOpenFile?: (path: string) => void;
  onPinCitation?: (citation: AgentPaneCitation) => void;
}) {
  const path = eventPath(props.event);
  const citations = eventCitations(props.event);
  return (
    <article class="agent-stream-card agent-turn" data-testid="agent-turn">
      <header>
        <span class="agent-card-kind">agent output</span>
        <span class="agent-card-meta">{props.event.ts}</span>
      </header>
      <RichContent
        text={eventText(props.event)}
        onOpenFile={props.onOpenFile}
      />
      <Show when={path}>
        {(file) => <FileLink path={file()} onOpenFile={props.onOpenFile} />}
      </Show>
      <Show when={citations.length}>
        <div class="agent-citations">
          <For each={citations}>
            {(citation) => (
              <button
                type="button"
                class="agent-citation"
                data-testid="agent-pin-citation"
                onClick={() => props.onPinCitation?.(citation)}
              >
                Pin citation · {citation.label}
              </button>
            )}
          </For>
        </div>
      </Show>
    </article>
  );
}

export function ToolCard(props: {
  event: AgentEvent;
  display: AgentToolDisplay;
  onOpenFile?: (path: string) => void;
}) {
  const [open, setOpen] = createSignal(props.display === "expanded");
  const path = eventPath(props.event);
  const status = eventStatus(props.event);
  return (
    <Show when={props.display !== "hidden"}>
      <article
        class={`agent-stream-card agent-tool-card agent-tool-${status}`}
        data-testid="agent-tool-card"
        data-tool-status={status}
      >
        <header>
          <button
            type="button"
            class="agent-tool-toggle"
            aria-expanded={open()}
            onClick={() => setOpen((value) => !value)}
          >
            <span class="agent-card-kind">tool</span>
            <strong>
              {props.event.tool ??
                rawString(props.event, "name") ??
                "tool call"}
            </strong>
          </button>
          <span class="agent-tool-status">{status.replace("_", " ")}</span>
        </header>
        <Show when={open()}>
          <pre>
            {safeJson(
              props.event.args ?? rawObject(props.event.raw.args) ?? {},
            )}
          </pre>
          <p>{eventText(props.event)}</p>
          <Show when={path}>
            {(file) => <FileLink path={file()} onOpenFile={props.onOpenFile} />}
          </Show>
        </Show>
      </article>
    </Show>
  );
}

export function PermissionCard(props: {
  event: AgentEvent;
  posture: AgentPermissionPosture;
  resolved?: boolean;
  onApprove?: (event: AgentEvent) => void;
  onDecline?: (event: AgentEvent) => void;
  onApproveRemaining?: (event: AgentEvent) => void;
  onOpenFile?: (path: string) => void;
}) {
  const diff = eventDiff(props.event);
  const rows =
    diff.before !== undefined && diff.after !== undefined
      ? diffRows(diff.before, diff.after)
      : [];
  return (
    <article
      class="agent-stream-card agent-permission-card"
      data-testid="agent-permission-card"
      data-resolved={props.resolved ? "true" : "false"}
    >
      <header>
        <span class="agent-card-kind">permission request</span>
        <span class="agent-gate-mode agent-gate-gated">gated</span>
      </header>
      <Show
        when={props.resolved}
        fallback={
          <>
            <div class="agent-diff" data-testid="agent-inline-diff">
              <header>
                <Show
                  when={diff.path}
                  fallback={<span>file path unavailable</span>}
                >
                  {(path) => (
                    <FileLink path={path()} onOpenFile={props.onOpenFile} />
                  )}
                </Show>
                <span>proposed edit</span>
              </header>
              <Show
                when={rows.length}
                fallback={
                  <p class="agent-diff-unavailable">
                    Diff unavailable; inspect the recorded request before
                    approving.
                  </p>
                }
              >
                <div
                  class="agent-diff-lines"
                  role="table"
                  aria-label="Proposed diff"
                >
                  <For each={rows}>
                    {(row) => (
                      <div
                        class={`agent-diff-row agent-diff-${row.kind}`}
                        data-testid="agent-diff-row"
                        data-diff-kind={row.kind}
                        role="row"
                      >
                        {/* The +/- glyph is decoration; the state is announced
                            as the word so a reader never needs the glyph. */}
                        <span class="sr-only">{row.kind}</span>
                        <span class="agent-diff-gutter" aria-hidden="true">
                          {row.kind === "removed"
                            ? "−"
                            : row.kind === "added"
                              ? "+"
                              : " "}
                        </span>
                        <span class="agent-diff-line-number">
                          {row.oldLine ?? ""}
                        </span>
                        <span class="agent-diff-line-number">
                          {row.newLine ?? ""}
                        </span>
                        <code>{row.text || " "}</code>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </div>
            <div class="agent-card-actions">
              <button
                type="button"
                class="agent-action-approve"
                onClick={() => props.onApprove?.(props.event)}
              >
                Approve
              </button>
              <button
                type="button"
                onClick={() => props.onDecline?.(props.event)}
              >
                Decline
              </button>
              <button
                type="button"
                onClick={() => props.onApproveRemaining?.(props.event)}
              >
                Approve remaining this turn
              </button>
            </div>
          </>
        }
      >
        <p class="agent-permission-resolved">
          Permission request resolved; the transcript remains intact.
        </p>
      </Show>
    </article>
  );
}

export function PermissionAlarm(props: { event: AgentEvent }) {
  return (
    <article
      class="agent-stream-card agent-permission-alarm"
      data-testid="agent-permission-alarm"
      data-alarm="true"
    >
      <header>
        <span class="agent-card-kind">permission request alarm</span>
        <span class="agent-card-meta">bypass · {props.event.id}</span>
      </header>
      <p>
        The run continued under its container allowlist; an unexpected
        permission request was recorded.
      </p>
    </article>
  );
}

export function ActivityRow(props: { event: AgentEvent }) {
  return (
    <div class="agent-activity-row" data-testid="agent-activity-row">
      <span class="agent-activity-dot" />
      <span>{eventText(props.event)}</span>
      <code>{props.event.verb}</code>
    </div>
  );
}

export function AgentEventStream(props: {
  events: AgentEvent[];
  posture: AgentPermissionPosture;
  thinkingDisplay: AgentThinkingDisplay;
  toolCallsDisplay: AgentToolDisplay;
  resolved: string[];
  onThinkingDisplay: (display: AgentThinkingDisplay) => void;
  onApprove?: AgentPaneProps["onApprovePermission"];
  onDecline?: AgentPaneProps["onDeclinePermission"];
  onApproveRemaining?: AgentPaneProps["onApproveRemainingTurn"];
  onResubmit?: AgentPaneProps["onResubmit"];
  onCopyPrompt?: AgentPaneProps["onCopyPrompt"];
  onOpenFile?: AgentPaneProps["onOpenFile"];
  onPinCitation?: AgentPaneProps["onPinCitation"];
}) {
  return (
    <Show
      when={props.events.length}
      fallback={<p class="agent-empty-state">Waiting for ACP events from this run.</p>}
    >
      <For each={props.events}>
        {(event) => (
          <Switch fallback={<ActivityRow event={event} />}>
            <Match when={event.verb === "user"}>
              <UserCard event={event} onResubmit={props.onResubmit} onCopyPrompt={props.onCopyPrompt} />
            </Match>
            <Match when={event.verb === "assistant"}>
              <AgentTurn event={event} onOpenFile={props.onOpenFile} onPinCitation={props.onPinCitation} />
            </Match>
            <Match when={event.verb === "thinking"}>
              <ThinkingBlock event={event} display={props.thinkingDisplay} onDisplay={props.onThinkingDisplay} />
            </Match>
            <Match when={event.verb === "tool_call" || event.verb === "tool_result" || event.verb === "tool_error"}>
              <ToolCard event={event} display={props.toolCallsDisplay} onOpenFile={props.onOpenFile} />
            </Match>
            <Match when={event.verb === "permission_request" && props.posture === "gated"}>
              <PermissionCard event={event} posture="gated" resolved={props.resolved.includes(event.id)} onApprove={props.onApprove} onDecline={props.onDecline} onApproveRemaining={props.onApproveRemaining} onOpenFile={props.onOpenFile} />
            </Match>
            <Match when={event.verb === "permission_request"}>
              <PermissionAlarm event={event} />
            </Match>
          </Switch>
        )}
      </For>
    </Show>
  );
}
