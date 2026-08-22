// The locator grammar — one address space, so there is one resolver.
//
//   locus://<project>/session/<id>[/run/<id>]
//   locus://<project>/task/<id>       artifact/<id>       page/<slug>
//   locus://<project>/workflow/<id>[/execution/<id>]      agent/<name>@<version>
//
// Plus a view form for the screens that address no single object:
//
//   locus://<project>/<view>
//
// The command palette, global search, inbox items, board-card links, artifact
// comments, deep links, and a detached window's identity are seven navigation
// paths. Against this grammar they are one resolver with seven callers.

import { VIEWS } from "./views";
import type { View } from "./views";

export const LOCATOR_SCHEME = "locus://";

/** The six addressable object kinds. */
export const KINDS = [
  "session",
  "task",
  "artifact",
  "page",
  "workflow",
  "agent",
] as const;

export type LocatorKind = (typeof KINDS)[number];

/**
 * How each view addresses its object. One viewer per kind, however you reached it.
 *
 * `sessions` and `runs` share the `session` kind and are told apart by the run
 * sub-segment: a live session is operated in Automate, and a finished run is
 * examined in Review. Dashboard is now, Review is after — and the locator says
 * which of the two you asked for.
 */
interface ViewForm {
  kind: LocatorKind;
  idParam: string;
  sub?: { segment: string; param: string; required: boolean };
}

const VIEW_FORM: Partial<Record<View, ViewForm>> = {
  sessions: { kind: "session", idParam: "sessionId" },
  runs: {
    kind: "session",
    idParam: "sessionId",
    sub: { segment: "run", param: "runId", required: true },
  },
  board: { kind: "task", idParam: "taskId" },
  artifact: { kind: "artifact", idParam: "artifactId" },
  wiki: { kind: "page", idParam: "slug" },
  canvas: {
    kind: "workflow",
    idParam: "workflowId",
    sub: { segment: "execution", param: "executionId", required: false },
  },
  agents: { kind: "agent", idParam: "agentName" },
};

/** The sub-segment each kind may carry. Parsing needs this before a view is known. */
const KIND_SUB: Partial<Record<LocatorKind, string>> = {
  session: "run",
  workflow: "execution",
};

/** Which view a parsed locator opens. */
function viewFor(kind: LocatorKind, hasSub: boolean): View {
  if (kind === "session") return hasSub ? "runs" : "sessions";
  const found = Object.entries(VIEW_FORM).find(
    ([view, form]) => form!.kind === kind && !(view === "runs"),
  )!;
  return found[0] as View;
}

export interface ViewParams {
  project: string;
  [key: string]: string;
}

export interface Locator {
  project: string;
  /** Null for a view-form locator, which addresses a screen rather than an object. */
  kind: LocatorKind | null;
  /** The object id, or the view name for a view-form locator. */
  id: string;
  /** The sub-object id, where the kind allows one. */
  subId: string | null;
}

export interface NavTarget {
  view: View;
  params: ViewParams;
}

const SEGMENT = /^[A-Za-z0-9._@-]+$/;

export class LocatorError extends Error {}

/** Structural parse. Throws with the offending segment named. */
export function parse(locator: string): Locator {
  if (!locator.startsWith(LOCATOR_SCHEME)) {
    throw new LocatorError(
      `scheme: expected "${LOCATOR_SCHEME}", got "${locator.split("/")[0]}//"`,
    );
  }

  const rest = locator.slice(LOCATOR_SCHEME.length);
  const segments = rest.split("/");

  const [project, ...tail] = segments;
  if (!project || !SEGMENT.test(project)) {
    throw new LocatorError(`project: "${project}" is not a project segment`);
  }
  if (tail.length === 0) {
    throw new LocatorError(
      "view: a locator addresses a view or an object, and this names neither",
    );
  }

  // View form: locus://<project>/<view>
  if (tail.length === 1) {
    if (!(VIEWS as readonly string[]).includes(tail[0])) {
      throw new LocatorError(
        `view: "${tail[0]}" is not one of the fourteen views`,
      );
    }
    return { project, kind: null, id: tail[0], subId: null };
  }

  const [kind, id, ...sub] = tail;
  if (!(KINDS as readonly string[]).includes(kind)) {
    throw new LocatorError(`kind: "${kind}" is not one of ${KINDS.join(", ")}`);
  }
  const k = kind as LocatorKind;

  if (!id || !SEGMENT.test(id)) {
    throw new LocatorError(`id: "${id}" is not an id for kind "${kind}"`);
  }
  if (k === "agent" && !/^[A-Za-z0-9._-]+@\d+$/.test(id)) {
    throw new LocatorError(`id: "${id}" is not <name>@<version>`);
  }

  if (sub.length === 0) return { project, kind: k, id, subId: null };

  const allowed = KIND_SUB[k];
  if (!allowed) {
    throw new LocatorError(
      `sub: kind "${kind}" carries no sub-object, got "${sub[0]}"`,
    );
  }
  if (sub.length !== 2) {
    throw new LocatorError(
      `sub: expected "${allowed}/<id>", got "${sub.join("/")}"`,
    );
  }
  if (sub[0] !== allowed) {
    throw new LocatorError(
      `sub: kind "${kind}" carries "${allowed}", not "${sub[0]}"`,
    );
  }
  if (!SEGMENT.test(sub[1])) {
    throw new LocatorError(`sub: "${sub[1]}" is not an id`);
  }

  return { project, kind: k, id, subId: sub[1] };
}

/** The parser's inverse. */
export function format(view: View, params: ViewParams): string {
  const form = VIEW_FORM[view];
  const base = `${LOCATOR_SCHEME}${params.project}`;
  if (!form) return `${base}/${view}`;

  let id = params[form.idParam];
  if (form.kind === "agent" && id && params.agentVersion)
    id = `${id}@${params.agentVersion}`;

  const subId = form.sub ? params[form.sub.param] : undefined;
  // With no id — or with a required sub-object missing — this addresses the screen
  // rather than one object on it.
  if (!id || (form.sub?.required && !subId)) return `${base}/${view}`;

  return subId
    ? `${base}/${form.kind}/${id}/${form.sub!.segment}/${subId}`
    : `${base}/${form.kind}/${id}`;
}

/** The single navigation entry point. Everything that navigates comes through here. */
export function resolve(locator: string): NavTarget {
  const parsed = parse(locator);

  if (parsed.kind === null) {
    return { view: parsed.id as View, params: { project: parsed.project } };
  }

  const view = viewFor(parsed.kind, parsed.subId !== null);
  const form = VIEW_FORM[view]!;

  const params: ViewParams = { project: parsed.project };
  if (parsed.kind === "agent") {
    const [name, version] = parsed.id.split("@");
    params.agentName = name;
    params.agentVersion = version;
  } else {
    params[form.idParam] = parsed.id;
  }
  if (form.sub && parsed.subId) params[form.sub.param] = parsed.subId;

  return { view, params };
}
