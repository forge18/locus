// The nav store: the only thing that sets `view`.
//
// A component that flips the view itself is a navigation path that will drift
// from the other six. Everything goes through `go` or `open`, and
// scripts/check-single-resolver.sh keeps it that way.

import { createMemo, createSignal } from "solid-js";
import type { Accessor } from "solid-js";
import { CATEGORY_LABELS, categoryOf } from "./views";
import type { Category, View } from "./views";
import { tabsFor } from "./tabs";
import type { CategoryTab } from "./tabs";
import { format, resolve } from "./locator";
import type { NavTarget, ViewParams } from "./locator";
import { DESKTOP_ROUTES } from "./desktop-screen-inventory";

/** The latest locator, retained for compatibility with the first persistence format. */
export const NAV_STORAGE_KEY = "locus.navigation.current";
/** The per-window history and cursor retained alongside the latest locator. */
export const NAV_HISTORY_STORAGE_KEY = "locus.navigation.history";

const BROWSER_HISTORY_KEY = "__locusNavigation";
const BROWSER_HISTORY_SOURCE = "locus-navigation";
const OBJECT_LOCATOR_PARAMS = new Set([
  "sessionId",
  "taskId",
  "runId",
  "artifactId",
  "slug",
  "workflowId",
  "executionId",
  "agentName",
  "agentVersion",
  "botId",
]);

interface PersistedNavigation {
  stack: string[];
  cursor: number;
  /** Stable across a reload so browser history entries remain this store's entries. */
  owner?: string;
}

interface BrowserNavigationState extends PersistedNavigation {
  source: typeof BROWSER_HISTORY_SOURCE;
  locator: string;
}

export interface NavStore {
  view: Accessor<View>;
  params: Accessor<ViewParams>;
  category: Accessor<Category>;
  categoryLabel: Accessor<string>;
  /** The full locator for where you are. */
  locator: Accessor<string>;
  /** The same, without the scheme — what the title bar and tab bar show. */
  locatorPath: Accessor<string>;
  tabs: Accessor<CategoryTab[]>;

  /** Navigate to a view. The one way `view` changes. */
  go: (view: View, params?: Partial<ViewParams>) => void;
  /** Navigate by locator — ⌘K, a deep link, an inbox item, a board-card link. */
  open: (locator: string) => NavTarget;

  /** Detail opens in place, as a sheet over the current category. */
  detail: Accessor<NavTarget | null>;
  openDetail: (locator: string) => void;
  closeDetail: () => void;

  canBack: Accessor<boolean>;
  canForward: Accessor<boolean>;
  back: () => void;
  forward: () => void;
  /** The history stack, as locators. Per window. */
  history: Accessor<string[]>;
}

export interface NavStoreOptions {
  view?: View;
  project?: string;
}

function newHistoryOwner(): string {
  // Do not require crypto.randomUUID: the desktop test runtime and older webviews
  // do not all expose it. The owner only prevents one store's popstate events from
  // changing another store in the same window.
  return `nav-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

function validNavigation(
  value: unknown,
  fallbackOwner?: string,
): PersistedNavigation | null {
  if (typeof value !== "object" || value === null) return null;
  const candidate = value as {
    stack?: unknown;
    cursor?: unknown;
    owner?: unknown;
  };
  if (
    !Array.isArray(candidate.stack) ||
    !candidate.stack.length ||
    typeof candidate.cursor !== "number" ||
    !Number.isInteger(candidate.cursor) ||
    candidate.cursor < 0 ||
    candidate.cursor >= candidate.stack.length ||
    candidate.stack.some((entry) => typeof entry !== "string")
  ) {
    return null;
  }

  // A locator is the persisted boundary. Rejecting the whole value on one bad
  // entry is safer than restoring a stack whose back button can later throw.
  for (const entry of candidate.stack) {
    try {
      resolve(entry);
    } catch {
      return null;
    }
  }

  return {
    stack: [...candidate.stack] as string[],
    cursor: candidate.cursor as number,
    owner:
      typeof candidate.owner === "string" ? candidate.owner : fallbackOwner,
  };
}

function decodeStoredNavigation(
  raw: string | null,
): PersistedNavigation | null {
  if (!raw) return null;
  try {
    // NAV_STORAGE_KEY historically held the locator directly, rather than a
    // JSON string. Accept that shape as well as the structured history format.
    if (raw.startsWith("locus://")) {
      resolve(raw);
      return { stack: [raw], cursor: 0 };
    }
    const value: unknown = JSON.parse(raw);
    if (typeof value === "string") {
      resolve(value);
      return { stack: [value], cursor: 0 };
    }
    return validNavigation(value);
  } catch {
    // A corrupt preference must never prevent the desktop from opening.
    return null;
  }
}

function readPersistedNavigation(): PersistedNavigation | null {
  if (typeof window === "undefined") return null;
  try {
    const history = decodeStoredNavigation(
      window.localStorage.getItem(NAV_HISTORY_STORAGE_KEY),
    );
    if (history) return history;
    return decodeStoredNavigation(window.localStorage.getItem(NAV_STORAGE_KEY));
  } catch {
    return null;
  }
}

function browserState(state: unknown): BrowserNavigationState | null {
  if (typeof state !== "object" || state === null) return null;
  const value = (state as Record<string, unknown>)[BROWSER_HISTORY_KEY];
  if (typeof value !== "object" || value === null) return null;
  const candidate = value as Record<string, unknown>;
  if (candidate.source !== BROWSER_HISTORY_SOURCE) return null;
  const navigation = validNavigation(candidate);
  if (!navigation || typeof candidate.locator !== "string") return null;
  try {
    resolve(candidate.locator);
  } catch {
    return null;
  }
  return {
    ...navigation,
    source: BROWSER_HISTORY_SOURCE,
    locator: candidate.locator,
  };
}

export function createNavStore(options: NavStoreOptions = {}): NavStore {
  const project = options.project;
  const fallbackView = options.view ?? "inbox";
  const fallback: NavTarget = {
    view: fallbackView,
    params:
      DESKTOP_ROUTES.find((route) => route.id === fallbackView)?.scope ===
      "project"
        ? project
          ? { project }
          : {}
        : {},
  };

  // An explicit target is an intentional deep link (and is used by detached
  // windows), so it wins over the last window snapshot.
  const restored =
    !options.view && !options.project ? readPersistedNavigation() : null;
  const start = restored ? resolve(restored.stack[restored.cursor]) : fallback;
  const initialStack = restored?.stack ?? [format(start.view, start.params)];
  const initialCursor = restored?.cursor ?? 0;
  const historyOwner = restored?.owner ?? newHistoryOwner();

  const [target, setTarget] = createSignal<NavTarget>(start);
  const [stack, setStack] = createSignal<string[]>(initialStack);
  const [cursor, setCursor] = createSignal(initialCursor);
  const [detail, setDetail] = createSignal<NavTarget | null>(null);

  const view = createMemo(() => target().view);
  const params = createMemo(() => target().params);
  const locator = createMemo(() => format(target().view, target().params));

  const persist = () => {
    if (typeof window === "undefined") return;
    const current = stack()[cursor()];
    if (!current) return;
    try {
      // Keep the small current-locator key readable for callers that used the
      // original persistence contract, and retain the complete stack separately.
      window.localStorage.setItem(NAV_STORAGE_KEY, current);
      window.localStorage.setItem(
        NAV_HISTORY_STORAGE_KEY,
        JSON.stringify({
          stack: stack(),
          cursor: cursor(),
          owner: historyOwner,
        } satisfies PersistedNavigation),
      );
    } catch {
      // Storage is optional (private browsing and a disabled webview are valid).
    }
  };

  const writeBrowserHistory = (mode: "push" | "replace") => {
    if (typeof window === "undefined") return;
    const state: BrowserNavigationState = {
      source: BROWSER_HISTORY_SOURCE,
      owner: historyOwner,
      stack: stack(),
      cursor: cursor(),
      locator: stack()[cursor()],
    };
    try {
      const current =
        typeof window.history.state === "object" &&
        window.history.state !== null
          ? window.history.state
          : {};
      const next = {
        ...(current as Record<string, unknown>),
        [BROWSER_HISTORY_KEY]: state,
      };
      if (mode === "push") window.history.pushState(next, "");
      else window.history.replaceState(next, "");
    } catch {
      // History can be unavailable in non-browser hosts; the Solid state remains
      // authoritative in that case.
    }
  };

  /**
   * Push onto the stack, discarding anything forward of the cursor.
   *
   * The target is set from `resolve(locator)` rather than from what the caller
   * passed, so the store's params are always exactly what the locator encodes.
   * Anything the grammar does not carry cannot survive a back button, and this
   * is where that becomes true instead of merely intended.
   */
  const push = (next: NavTarget) => {
    const at = format(next.view, next.params);
    const currentStack = stack();
    const currentCursor = cursor();
    setTarget(resolve(at));
    if (at === currentStack[currentCursor]) return;
    const nextStack = [...currentStack.slice(0, currentCursor + 1), at];
    setStack(nextStack);
    setCursor(currentCursor + 1);
    writeBrowserHistory("push");
  };

  const go: NavStore["go"] = (nextView, nextParams) => {
    const routeScope = DESKTOP_ROUTES.find(
      (route) => route.id === nextView,
    )?.scope;
    const { project: requestedProject, ...otherParams } = nextParams ?? {};
    const carriesObjectIdentity = Object.keys(otherParams).some((key) =>
      OBJECT_LOCATOR_PARAMS.has(key),
    );
    const params =
      routeScope === "project" || carriesObjectIdentity
        ? {
            project: requestedProject ?? target().params.project ?? project,
            ...otherParams,
          }
        : otherParams;
    push({ view: nextView, params });
  };

  const open: NavStore["open"] = (at) => {
    const resolved = resolve(at);
    push(resolved);
    return resolved;
  };

  const applyBrowserNavigation = (event: PopStateEvent) => {
    const state = browserState(event.state);
    if (!state || state.owner !== historyOwner) return;

    const currentStack = stack();
    let nextStack = currentStack;
    let nextCursor = currentStack.indexOf(state.locator);
    if (nextCursor < 0) {
      // This can happen after a restored session encounters an entry created by
      // an older app version. Only adopt a fully valid state from our marker.
      nextStack = state.stack;
      nextCursor = state.cursor;
    }
    if (nextCursor < 0 || nextCursor >= nextStack.length) return;
    setStack(nextStack);
    setCursor(nextCursor);
    setTarget(resolve(nextStack[nextCursor]));
    persist();
  };

  const step = (delta: number) => {
    const currentCursor = cursor();
    const to = currentCursor + delta;
    if (to < 0 || to >= stack().length) return;
    setCursor(to);
    setTarget(resolve(stack()[to]));

    // Keep the browser's session history aligned with the store. The state is
    // updated synchronously for keyboard/button consumers; popstate confirms it
    // once the browser move completes.
    if (typeof window !== "undefined") {
      try {
        window.history.go(delta);
      } catch {
        writeBrowserHistory("replace");
      }
    }
  };

  if (typeof window !== "undefined") {
    // Replace the current document entry rather than changing the URL: locators
    // are an app address space and are not valid web URLs.
    writeBrowserHistory("replace");
    window.addEventListener("popstate", applyBrowserNavigation);
    window.addEventListener("beforeunload", persist);
    window.addEventListener("pagehide", persist);
    window.addEventListener("storage", (event) => {
      if (
        (event.key !== NAV_HISTORY_STORAGE_KEY &&
          event.key !== NAV_STORAGE_KEY) ||
        (event.storageArea && event.storageArea !== window.localStorage)
      )
        return;
      const restored = decodeStoredNavigation(event.newValue);
      if (!restored) return;
      const nextCursor = restored.cursor;
      setStack(restored.stack);
      setCursor(nextCursor);
      setTarget(resolve(restored.stack[nextCursor]));
      writeBrowserHistory("replace");
    });
  }

  return {
    view,
    params,
    category: createMemo(() => categoryOf(view())),
    categoryLabel: createMemo(() => CATEGORY_LABELS[categoryOf(view())]),
    locator,
    locatorPath: createMemo(() => locator().replace("locus://", "")),
    tabs: createMemo(() => tabsFor(categoryOf(view()))),
    go,
    open,
    detail,
    // Detail is a sheet over the current category. It does not touch `view`, which
    // is why the rail does not move when one opens.
    openDetail: (at) => setDetail(resolve(at)),
    closeDetail: () => setDetail(null),
    canBack: createMemo(() => cursor() > 0),
    canForward: createMemo(() => cursor() < stack().length - 1),
    back: () => step(-1),
    forward: () => step(1),
    history: stack,
  };
}
