import type { AgentEvent } from "../types/event";
import type {
  AgentFieldValue,
  AgentPaneCitation,
  AgentPaneElicitation,
  AgentPaneFinding,
} from "./agent-panel-model";

export function rawObject(value: unknown): Record<string, unknown> | undefined {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}

function nestedRawObjects(event: AgentEvent): Record<string, unknown>[] {
  const raw = event.raw;
  const params = rawObject(raw.params);
  const update = rawObject(params?.update ?? raw.update);
  const content = rawObject(update?.content);
  const args = rawObject(event.args ?? raw.args);
  return [raw, params, update, content, args].filter(
    (value): value is Record<string, unknown> => value !== undefined,
  );
}

export function rawString(event: AgentEvent, key: string): string | undefined {
  return nestedRawObjects(event).reduce<string | undefined>(
    (found, object) =>
      found ??
      (typeof object[key] === "string" && object[key].trim()
        ? object[key]
        : undefined),
    undefined,
  );
}

export function eventText(event: AgentEvent): string {
  return (
    event.text ??
    rawString(event, "text") ??
    rawString(event, "body") ??
    rawString(event, "message") ??
    `${event.verb.replace(/_/g, " ")} · event ${event.seq + 1}`
  );
}

export function eventPath(event: AgentEvent): string | undefined {
  return (
    rawString(event, "path") ??
    rawString(event, "file") ??
    rawString(event, "filePath")
  );
}

const TOOL_STATUSES = [
  "queued",
  "pending",
  "running",
  "in_progress",
  "completed",
  "cancelled",
  "failed",
] as const;
export type ToolStatus = (typeof TOOL_STATUSES)[number];

export function eventStatus(event: AgentEvent): ToolStatus {
  const status = rawString(event, "status")?.replace("-", "_");
  if (status && TOOL_STATUSES.includes(status as ToolStatus)) {
    return status as ToolStatus;
  }
  if (event.verb === "tool_call") return "running";
  if (event.verb === "tool_error") return "failed";
  if (event.verb === "tool_result") return "completed";
  return "pending";
}

export function eventDiff(event: AgentEvent): {
  path?: string;
  before?: string;
  after?: string;
} {
  const diff = nestedRawObjects(event)
    .map((object) => rawObject(object.diff))
    .find((value) => value !== undefined);
  return {
    path:
      (diff && typeof diff.path === "string" ? diff.path : undefined) ??
      (diff && typeof diff.file === "string" ? diff.file : undefined) ??
      eventPath(event),
    before:
      (diff && typeof diff.before === "string" ? diff.before : undefined) ??
      rawString(event, "before"),
    after:
      (diff && typeof diff.after === "string" ? diff.after : undefined) ??
      rawString(event, "after"),
  };
}

export interface DiffRow {
  kind: "context" | "removed" | "added";
  oldLine: number | undefined;
  newLine: number | undefined;
  text: string;
}

/** A bounded LCS diff keeps insertions and deletions from mislabelling later context. */
export function diffRows(before: string, after: string): DiffRow[] {
  const oldLines = before.split(/\r?\n/);
  const newLines = after.split(/\r?\n/);
  const common = Array.from({ length: oldLines.length + 1 }, () =>
    new Array<number>(newLines.length + 1).fill(0),
  );
  for (let oldIndex = oldLines.length - 1; oldIndex >= 0; oldIndex--) {
    for (let newIndex = newLines.length - 1; newIndex >= 0; newIndex--) {
      common[oldIndex][newIndex] =
        oldLines[oldIndex] === newLines[newIndex]
          ? common[oldIndex + 1][newIndex + 1] + 1
          : Math.max(
              common[oldIndex + 1][newIndex],
              common[oldIndex][newIndex + 1],
            );
    }
  }
  const rows: DiffRow[] = [];
  let oldIndex = 0;
  let newIndex = 0;
  let oldLine = 1;
  let newLine = 1;
  while (oldIndex < oldLines.length || newIndex < newLines.length) {
    if (
      oldLines[oldIndex] !== undefined &&
      oldLines[oldIndex] === newLines[newIndex]
    ) {
      rows.push({
        kind: "context",
        oldLine: oldLine++,
        newLine: newLine++,
        text: oldLines[oldIndex],
      });
      oldIndex++;
      newIndex++;
    } else if (
      newIndex >= newLines.length ||
      (oldIndex < oldLines.length &&
        common[oldIndex + 1][newIndex] >= common[oldIndex][newIndex + 1])
    ) {
      rows.push({
        kind: "removed",
        oldLine: oldLine++,
        newLine: undefined,
        text: oldLines[oldIndex++],
      });
    } else {
      rows.push({
        kind: "added",
        oldLine: undefined,
        newLine: newLine++,
        text: newLines[newIndex++],
      });
    }
  }
  return rows;
}

function citationFromObject(
  value: Record<string, unknown>,
): AgentPaneCitation | undefined {
  if (
    typeof value.id !== "string" ||
    typeof value.label !== "string" ||
    typeof value.source !== "string"
  ) {
    return undefined;
  }
  return {
    id: value.id,
    label: value.label,
    source: value.source,
    summary: typeof value.summary === "string" ? value.summary : undefined,
  };
}

export function eventCitations(event: AgentEvent): AgentPaneCitation[] {
  const result: AgentPaneCitation[] = [];
  for (const object of nestedRawObjects(event)) {
    const candidates = [rawObject(object.citation)];
    if (Array.isArray(object.citations)) candidates.push(...object.citations.map(rawObject));
    for (const candidate of candidates) {
      const citation = candidate ? citationFromObject(candidate) : undefined;
      if (citation && !result.some((item) => item.id === citation.id)) result.push(citation);
    }
  }
  return result;
}

export function formatTokens(value: number): string {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}m`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
  return String(value);
}

export function isSafeUrl(value: string): boolean {
  try {
    return ["http:", "https:"].includes(new URL(value).protocol);
  } catch {
    return false;
  }
}

export function workspacePath(path: string): string | undefined {
  const relative = path.startsWith("/workspace/")
    ? path.slice("/workspace/".length)
    : path;
  if (!relative.trim() || relative.startsWith("/") || relative.includes(":") || relative.includes("..")) {
    return undefined;
  }
  return relative;
}

const SENSITIVE_KEY =
  /(?:token|secret|password|api[_-]?key|credential|authorization|cookie)/i;

function redactSecrets(value: unknown, key = ""): unknown {
  if (SENSITIVE_KEY.test(key)) return "[redacted]";
  if (Array.isArray(value)) return value.map((item) => redactSecrets(item));
  const object = rawObject(value);
  if (object) {
    return Object.fromEntries(
      Object.entries(object).map(([name, item]) => [
        name,
        redactSecrets(item, name),
      ]),
    );
  }
  return value;
}

export function safeJson(value: unknown): string {
  try {
    return JSON.stringify(redactSecrets(value ?? {}), null, 2);
  } catch {
    return "[tool payload unavailable]";
  }
}

export interface RichBlock {
  kind: "paragraph" | "code" | "table" | "image";
  text?: string;
  language?: string;
  lines?: string[];
  rows?: string[][];
  alt?: string;
  source?: string;
}

export function markdownBlocks(text: string): RichBlock[] {
  const lines = text.split(/\r?\n/);
  const blocks: RichBlock[] = [];
  let paragraph: string[] = [];
  let code: string[] | undefined;
  let language = "";
  const flushParagraph = () => {
    if (paragraph.length) {
      blocks.push({ kind: "paragraph", text: paragraph.join("\n") });
      paragraph = [];
    }
  };
  for (let index = 0; index < lines.length; index++) {
    const line = lines[index];
    if (code) {
      if (line.trimStart().startsWith("```")) {
        blocks.push({ kind: "code", language, lines: code });
        code = undefined;
        language = "";
      } else {
        code.push(line);
      }
      continue;
    }
    const fence = line.match(/^\s*```(.*)$/);
    if (fence) {
      flushParagraph();
      code = [];
      language = fence[1].trim();
      continue;
    }
    const image = line.match(/^!\[([^\]]*)\]\(([^)]+)\)\s*$/);
    if (image) {
      flushParagraph();
      blocks.push({ kind: "image", alt: image[1], source: image[2] });
      continue;
    }
    if (line.includes("|") && /^\s*\|?(?:\s*:?-+:?\s*\|)+\s*$/.test(lines[index + 1] ?? "")) {
      flushParagraph();
      const tableLines = [line];
      let end = index + 2;
      while (end < lines.length && lines[end].includes("|")) {
        tableLines.push(lines[end]);
        end++;
      }
      blocks.push({
        kind: "table",
        rows: tableLines.map((row) =>
          row
            .split("|")
            .map((cell) => cell.trim())
            .filter(Boolean),
        ),
      });
      index = end - 1;
      continue;
    }
    if (line.trim()) paragraph.push(line);
    else flushParagraph();
  }
  if (code) blocks.push({ kind: "code", language, lines: code });
  flushParagraph();
  return blocks;
}

export function eventsForRun(
  events: readonly AgentEvent[],
  runId: string,
): AgentEvent[] {
  return events
    .filter((event) => event.runId === runId)
    .sort(
      (left, right) => left.seq - right.seq || left.id.localeCompare(right.id),
    );
}

export function mergeEvents(
  current: readonly AgentEvent[],
  incoming: readonly AgentEvent[],
): AgentEvent[] {
  const byId = new Map(current.map((event) => [event.id, event]));
  for (const event of incoming) byId.set(event.id, event);
  return eventsForRun(
    [...byId.values()],
    incoming[0]?.runId ?? current[0]?.runId ?? "",
  );
}

export function mergeFindings(
  current: readonly AgentPaneFinding[],
  incoming: readonly AgentPaneFinding[],
): AgentPaneFinding[] {
  const byId = new Map(current.map((finding) => [finding.id, finding]));
  for (const finding of incoming) byId.set(finding.id, finding);
  return [...byId.values()];
}

function validateType(
  field: AgentPaneElicitation["fields"][number],
  value: string,
): string | undefined {
  if (field.type === "number" || field.type === "integer") {
    if (!Number.isFinite(Number(value)))
      return `${field.label} must be a number.`;
    if (field.type === "integer" && !Number.isInteger(Number(value))) {
      return `${field.label} must be a whole number.`;
    }
  }
  if (field.type === "boolean" && value !== "true" && value !== "false") {
    return `${field.label} must be true or false.`;
  }
  if (field.type === "enum" && !field.options?.includes(value)) {
    return `${field.label} must use one of the available options.`;
  }
  return undefined;
}

function validateFormat(
  field: AgentPaneElicitation["fields"][number],
  value: string,
): string | undefined {
  if (field.format === "email" && !/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(value)) {
    return `${field.label} must be a valid email address.`;
  }
  if (field.format === "uri" && !isSafeUrl(value)) {
    return `${field.label} must be an http or https URL.`;
  }
  if (!field.pattern) return undefined;
  try {
    return new RegExp(field.pattern).test(value)
      ? undefined
      : `${field.label} has an invalid format.`;
  } catch {
    return `${field.label} has an invalid validation rule.`;
  }
}

function validateBounds(
  field: AgentPaneElicitation["fields"][number],
  value: string,
): string | undefined {
  if (field.minLength !== undefined && value.length < field.minLength) {
    return `${field.label} must have at least ${field.minLength} characters.`;
  }
  const number = Number(value);
  if (field.minimum !== undefined && number < field.minimum)
    return `${field.label} must be at least ${field.minimum}.`;
  if (field.maximum !== undefined && number > field.maximum)
    return `${field.label} must be at most ${field.maximum}.`;
  return undefined;
}

export function validateElicitationField(
  field: AgentPaneElicitation["fields"][number],
  value: string,
): string | undefined {
  const trimmed = value.trim();
  if (!trimmed)
    return field.required
      ? `${field.label} is required before sending.`
      : undefined;
  return (
    validateType(field, value) ??
    validateFormat(field, value) ??
    validateBounds(field, value)
  );
}

export function typedElicitationValues(
  elicitation: AgentPaneElicitation,
  values: Record<string, string>,
): Record<string, AgentFieldValue> {
  const typed: Array<[string, AgentFieldValue]> = [];
  for (const field of elicitation.fields) {
    const value = values[field.id] ?? "";
    if (!value.trim() && !field.required) continue;
    if (field.type === "number" || field.type === "integer")
      typed.push([field.id, Number(value)]);
    else if (field.type === "boolean") typed.push([field.id, value === "true"]);
    else typed.push([field.id, value]);
  }
  return Object.fromEntries(typed);
}
