// Canonical locator grammar shared by view links and object deep links.
// Views use locus://<project|all|app>/view/<id>; objects use
// locus://<project>/<kind>/<id>[/<sub-kind>/<id>].
import { Desktop_FIXTURE_ROUTES } from "../fixtures/desktop-screen-inventory";
import type { View } from "./views";

export const LOCATOR_SCHEME = "locus://";
export const KINDS = [
  "session",
  "task",
  "artifact",
  "page",
  "workflow",
  "agent",
  "bot",
] as const;
export type LocatorKind = (typeof KINDS)[number];
export type ViewParams = {
  project?: string;
  [key: string]: string | undefined;
};

const VIEW_IDS = Desktop_FIXTURE_ROUTES.map(
  (route) => route.id,
) as readonly string[];
const PROJECT_VIEWS = new Set(
  Desktop_FIXTURE_ROUTES.filter((route) => route.scope === "project").map(
    (route) => route.id,
  ),
);
const APP_VIEWS = new Set(
  Desktop_FIXTURE_ROUTES.filter((route) => route.scope === "app").map(
    (route) => route.id,
  ),
);
const SEGMENT = /^[A-Za-z0-9._@-]+$/;
const KIND_SUB: Partial<Record<LocatorKind, string>> = {
  session: "run",
  workflow: "execution",
};

export class LocatorError extends Error {}

export interface Locator {
  project: string;
  kind: LocatorKind | null;
  id: string;
  subId: string | null;
}
export interface NavTarget {
  view: View;
  params: ViewParams;
}

function isScope(value: string): boolean {
  return value === "all" || value === "app";
}
function viewFor(kind: LocatorKind, hasSub: boolean): View {
  if (kind === "session") return (hasSub ? "runs" : "sessions") as View;
  if (kind === "task") return "sessions";
  if (kind === "artifact") return "artifact" as View;
  if (kind === "page") return "wiki" as View;
  if (kind === "workflow") return "canvas" as View;
  if (kind === "bot") return "bots" as View;
  return "agents" as View;
}

export function parse(locator: string): Locator {
  if (!locator.startsWith(LOCATOR_SCHEME))
    throw new LocatorError(`scheme: expected "${LOCATOR_SCHEME}"`);
  const segments = locator.slice(LOCATOR_SCHEME.length).split("/");
  const [scope, kind, id, ...sub] = segments;
  if (!scope || !SEGMENT.test(scope))
    throw new LocatorError(`scope: "${scope}" is invalid`);

  if (kind === "bots") {
    if (isScope(scope))
      throw new LocatorError("scope: bot locators require a project");
    if (segments.length === 2) {
      return { project: scope, kind: null, id: "bots", subId: null };
    }
    if (segments.length === 3 && id && SEGMENT.test(id)) {
      return { project: scope, kind: "bot", id, subId: null };
    }
    throw new LocatorError("bot: expected locus://<project>/bots[/<bot-id>]");
  }

  if (kind === "view") {
    if (segments.length !== 3 || !VIEW_IDS.includes(id)) {
      throw new LocatorError(`view: "${id}" is not one of the 30 views`);
    }
    return { project: scope, kind: null, id, subId: null };
  }

  if (isScope(scope))
    throw new LocatorError("scope: object locators require a project");
  if (!kind) throw new LocatorError("view: expected a route after the scope");
  if (!(KINDS as readonly string[]).includes(kind))
    throw new LocatorError(`kind: "${kind}" is not supported`);
  const k = kind as LocatorKind;
  if (!id || !SEGMENT.test(id))
    throw new LocatorError(`id: "${id}" is invalid`);
  if (k === "agent" && !/^[A-Za-z0-9._-]+@\d+$/.test(id))
    throw new LocatorError(`id: "${id}" is not <name>@<version>`);
  if (!sub.length) return { project: scope, kind: k, id, subId: null };
  const allowed = KIND_SUB[k];
  if (
    !allowed ||
    sub.length !== 2 ||
    sub[0] !== allowed ||
    !SEGMENT.test(sub[1])
  ) {
    throw new LocatorError(`sub: expected "${allowed ?? "none"}/<id>"`);
  }
  return { project: scope, kind: k, id, subId: sub[1] };
}

function viewScope(view: View): "project" | "all" | "app" {
  if (PROJECT_VIEWS.has(view)) return "project";
  if (APP_VIEWS.has(view)) return "app";
  return "all";
}

export function format(view: View, params: ViewParams): string {
  const scope = viewScope(view);
  if (view === "sessions" && params.taskId) {
    const project = params.project;
    if (!project || isScope(project))
      throw new LocatorError("project: object locators require a project");
    return `${LOCATOR_SCHEME}${project}/task/${params.taskId}`;
  }
  const project = params.project;
  if (view === "bots") {
    const project = params.project;
    if (!project || isScope(project) || !SEGMENT.test(project))
      throw new LocatorError("project: bot views require a project segment");
    if (params.botId !== undefined) {
      if (!SEGMENT.test(params.botId))
        throw new LocatorError("bot: bot id must be one locator segment");
      return `${LOCATOR_SCHEME}${project}/bots/${params.botId}`;
    }
    return `${LOCATOR_SCHEME}${project}/bots`;
  }
  const forms: Partial<Record<View, [LocatorKind, string, string?]>> = {
    sessions: ["session", "sessionId"],
    runs: ["session", "sessionId", "runId"],
    artifact: ["artifact", "artifactId"],
    wiki: ["page", "slug"],
    canvas: ["workflow", "workflowId", "executionId"],
    agents: ["agent", "agentName"],
  };
  const form = forms[view];
  if (form) {
    const [kind, idParam, subParam] = form;
    let id = params[idParam];
    if (kind === "agent" && id && params.agentVersion)
      id = `${id}@${params.agentVersion}`;
    if (id) {
      if (!project || isScope(project))
        throw new LocatorError(`project: object locators require a project`);
      if (subParam === "runId" && !params[subParam])
        return `${LOCATOR_SCHEME}${project}/view/${view}`;
      const sub = subParam && params[subParam];
      return `${LOCATOR_SCHEME}${project}/${kind}/${id}${sub ? `/${kind === "session" ? "run" : "execution"}/${sub}` : ""}`;
    }
  }
  if (scope === "project") {
    if (!project || isScope(project))
      throw new LocatorError(
        "project: a project-scoped view requires a project",
      );
    return `${LOCATOR_SCHEME}${project}/view/${view}`;
  }
  // Cross-project and app routes have canonical scopes; an inherited project is ignored.
  return `${LOCATOR_SCHEME}${scope}/view/${view}`;
}

export function resolve(locator: string): NavTarget {
  const parsed = parse(locator);
  if (parsed.kind === null) {
    const view = parsed.id as View;
    const expectedScope = viewScope(view);
    if (
      (expectedScope === "project" && isScope(parsed.project)) ||
      (expectedScope !== "project" && parsed.project !== expectedScope)
    ) {
      throw new LocatorError(
        `scope: ${view} requires the ${expectedScope} scope`,
      );
    }
    return {
      view,
      params: isScope(parsed.project) ? {} : { project: parsed.project },
    };
  }
  const view = viewFor(parsed.kind, parsed.subId !== null);
  const params: ViewParams = { project: parsed.project };
  if (parsed.kind === "agent") {
    const [name, version] = parsed.id.split("@");
    params.agentName = name;
    params.agentVersion = version;
  } else if (parsed.kind === "bot") {
    params.botId = parsed.id;
  } else {
    const key =
      parsed.kind === "session"
        ? "sessionId"
        : parsed.kind === "artifact"
          ? "artifactId"
          : parsed.kind === "page"
            ? "slug"
            : parsed.kind === "workflow"
              ? "workflowId"
              : "taskId";
    params[key] = parsed.id;
  }
  if (parsed.subId)
    params[parsed.kind === "session" ? "runId" : "executionId"] = parsed.subId;
  return { view, params };
}
