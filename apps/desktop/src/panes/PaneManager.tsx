import {
  For,
  Match,
  Show,
  Switch,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  type JSX,
} from "solid-js";
import { detachPane } from "./detach";
import {
  close,
  focus,
  panes,
  promote,
  split,
  type Pane,
  type PaneLayout,
  type PaneTree,
} from "./manager";
import "./pane-manager.css";

type SplitDirection = "horizontal" | "vertical";

export interface PaneManagerProps {
  initialPane: Pane;
  renderPane: (pane: Pane) => JSX.Element;
  createPane?: (source: Pane, direction: SplitDirection) => Pane | undefined;
  onDetach?: (pane: Pane) => Promise<unknown> | unknown;
}

function clampRatio(ratio: number): number {
  return Math.min(0.9, Math.max(0.1, ratio));
}

function resizeAt(
  tree: PaneTree,
  path: readonly ("first" | "second")[],
  ratio: number,
): PaneTree {
  if (tree.type === "leaf") return tree;
  if (path.length === 0) return { ...tree, ratio: clampRatio(ratio) };
  const [branch, ...rest] = path;
  return branch === "first"
    ? { ...tree, first: resizeAt(tree.first, rest, ratio) }
    : { ...tree, second: resizeAt(tree.second, rest, ratio) };
}

function appendPane(
  tree: PaneTree | undefined,
  pane: Pane,
  direction: SplitDirection = "horizontal",
): PaneTree {
  if (!tree) return { type: "leaf", pane };
  const first = panes(tree)[0];
  return first ? split(tree, first.id, pane, direction) : { type: "leaf", pane };
}

function leastRecentlyFocused(items: readonly Pane[]): Pane | undefined {
  return items.reduce<Pane | undefined>(
    (least, item) =>
      !least || item.focusedAt < least.focusedAt ? item : least,
    undefined,
  );
}

function PaneDivider(props: {
  direction: SplitDirection;
  onResize: (ratio: number) => void;
}) {
  let divider!: HTMLButtonElement;
  let host: HTMLElement | undefined;

  const onMove = (event: PointerEvent) => {
    if (!host) return;
    const bounds = host.getBoundingClientRect();
    const size = props.direction === "horizontal" ? bounds.width : bounds.height;
    const position =
      props.direction === "horizontal"
        ? event.clientX - bounds.left
        : event.clientY - bounds.top;
    if (size > 0) props.onResize(position / size);
  };
  const stop = () => {
    document.removeEventListener("pointermove", onMove);
    document.removeEventListener("pointerup", stop);
    host = undefined;
  };
  const start = (event: PointerEvent) => {
    event.preventDefault();
    host = divider.parentElement ?? undefined;
    document.addEventListener("pointermove", onMove);
    document.addEventListener("pointerup", stop);
  };
  onCleanup(stop);

  return (
    <button
      ref={divider}
      type="button"
      class="pane-divider"
      role="separator"
      aria-label="Resize panes"
      aria-orientation={props.direction === "horizontal" ? "vertical" : "horizontal"}
      onPointerDown={start}
    />
  );
}

function PaneCard(props: {
  pane: Pane;
  renderPane: (pane: Pane) => JSX.Element;
  onFocus: (id: string) => void;
  onSplit: (id: string, direction: SplitDirection) => void;
  onMinimize: (id: string) => void;
  onDetach: (pane: Pane) => void;
  onClose: (id: string) => void;
}) {
  return (
    <section
      class="managed-pane"
      data-testid={`managed-pane-${props.pane.id}`}
      data-pane-kind={props.pane.kind}
      onFocusIn={() => props.onFocus(props.pane.id)}
    >
      <header class="managed-pane-header">
        <span class="managed-pane-kind">{props.pane.kind} pane</span>
        <code>{props.pane.runId ?? props.pane.id}</code>
        <div class="managed-pane-actions">
          <button
            type="button"
            data-testid={`pane-split-${props.pane.id}`}
            onClick={() => props.onSplit(props.pane.id, "horizontal")}
          >
            Split
          </button>
          <button
            type="button"
            data-testid={`pane-split-vertical-${props.pane.id}`}
            onClick={() => props.onSplit(props.pane.id, "vertical")}
          >
            Split below
          </button>
          <button
            type="button"
            data-testid={`pane-minimize-${props.pane.id}`}
            onClick={() => props.onMinimize(props.pane.id)}
          >
            Minimize
          </button>
          <button
            type="button"
            data-testid={`pane-detach-${props.pane.id}`}
            onClick={() => props.onDetach(props.pane)}
          >
            Detach
          </button>
          <button
            type="button"
            data-testid={`pane-close-${props.pane.id}`}
            onClick={() => props.onClose(props.pane.id)}
          >
            Close
          </button>
        </div>
      </header>
      <div class="managed-pane-content">{props.renderPane(props.pane)}</div>
    </section>
  );
}

function PaneTreeView(props: {
  tree: PaneTree;
  path: readonly ("first" | "second")[];
  renderPane: (pane: Pane) => JSX.Element;
  onFocus: (id: string) => void;
  onSplit: (id: string, direction: SplitDirection) => void;
  onMinimize: (id: string) => void;
  onDetach: (pane: Pane) => void;
  onClose: (id: string) => void;
  onResize: (path: readonly ("first" | "second")[], ratio: number) => void;
}) {
  return (
    <Switch>
      <Match when={props.tree.type === "leaf"}>
        <PaneCard
          pane={(props.tree as Extract<PaneTree, { type: "leaf" }>).pane}
          renderPane={props.renderPane}
          onFocus={props.onFocus}
          onSplit={props.onSplit}
          onMinimize={props.onMinimize}
          onDetach={props.onDetach}
          onClose={props.onClose}
        />
      </Match>
      <Match when={props.tree.type === "split"}>
        <div
          class={`pane-tree pane-tree-${(
            props.tree as Extract<PaneTree, { type: "split" }>
          ).direction}`}
          style={{
            "--pane-ratio": (
              props.tree as Extract<PaneTree, { type: "split" }>
            ).ratio,
          }}
        >
          <div class="pane-tree-slot pane-tree-first">
            <PaneTreeView
              tree={(
                props.tree as Extract<PaneTree, { type: "split" }>
              ).first}
              path={[...props.path, "first"]}
              renderPane={props.renderPane}
              onFocus={props.onFocus}
              onSplit={props.onSplit}
              onMinimize={props.onMinimize}
              onDetach={props.onDetach}
              onClose={props.onClose}
              onResize={props.onResize}
            />
          </div>
          <PaneDivider
            direction={(
              props.tree as Extract<PaneTree, { type: "split" }>
            ).direction}
            onResize={(ratio) => props.onResize(props.path, ratio)}
          />
          <div class="pane-tree-slot pane-tree-second">
            <PaneTreeView
              tree={(
                props.tree as Extract<PaneTree, { type: "split" }>
              ).second}
              path={[...props.path, "second"]}
              renderPane={props.renderPane}
              onFocus={props.onFocus}
              onSplit={props.onSplit}
              onMinimize={props.onMinimize}
              onDetach={props.onDetach}
              onClose={props.onClose}
              onResize={props.onResize}
            />
          </div>
        </div>
      </Match>
    </Switch>
  );
}

export function PaneManager(props: PaneManagerProps) {
  const [tree, setTree] = createSignal<PaneTree | undefined>({
    type: "leaf",
    pane: props.initialPane,
  });
  const [strip, setStrip] = createSignal<Pane[]>([]);
  const [message, setMessage] = createSignal<string>();
  const [error, setError] = createSignal<string>();
  let initialKey = "";
  let generatedPane = 0;

  createEffect(() => {
    const pane = props.initialPane;
    const key = `${pane.id}:${pane.kind}:${pane.runId ?? ""}`;
    if (!initialKey) {
      initialKey = key;
      return;
    }
    if (key === initialKey) return;
    initialKey = key;
    setTree({ type: "leaf", pane });
    setStrip([]);
    setMessage(undefined);
    setError(undefined);
  });

  const focused = createMemo(() => {
    const current = tree();
    return current ? panes(current) : [];
  });
  const layout = createMemo<PaneLayout>(() => ({
    focused: focused(),
    strip: strip(),
  }));

  const splitPane = (id: string, direction: SplitDirection) => {
    const source = focused().find((pane) => pane.id === id);
    if (!source) return;
    const pane =
      props.createPane?.(source, direction) ?? {
        ...source,
        id: `${source.id}-split-${++generatedPane}`,
        focusedAt: Date.now(),
      };
    if (
      [...focused(), ...strip()].some((candidate) => candidate.id === pane.id)
    ) {
      setError(`Pane id \`${pane.id}\` is already open.`);
      return;
    }
    const current = tree();
    if (!current) return;
    const nextTree = focus(split(current, id, pane, direction), pane.id);
    const demoted =
      panes(nextTree).length > 4
        ? leastRecentlyFocused(panes(nextTree))
        : undefined;
    if (demoted) {
      setTree(close(nextTree, demoted.id));
      setStrip((currentStrip) => [
        ...currentStrip.filter((candidate) => candidate.id !== demoted.id),
        demoted,
      ]);
    } else {
      setTree(nextTree);
    }
    setMessage(undefined);
    setError(undefined);
  };

  const minimizePane = (id: string) => {
    const current = tree();
    const pane = focused().find((candidate) => candidate.id === id);
    if (!current || !pane) return;
    setTree(close(current, id));
    setStrip((currentStrip) => [
      ...currentStrip.filter((candidate) => candidate.id !== id),
      pane,
    ]);
  };

  const closePane = (id: string) => {
    const current = tree();
    if (!current) return;
    setTree(close(current, id));
    setMessage(undefined);
    setError(undefined);
  };

  const promotePane = (id: string) => {
    const promoted = strip().find((pane) => pane.id === id);
    if (!promoted) return;
    const current = focused();
    const leastRecent =
      current.length >= 4 ? leastRecentlyFocused(current) : undefined;
    const next = promote(layout(), id, Date.now());
    let nextTree = tree();
    if (leastRecent) nextTree = nextTree ? close(nextTree, leastRecent.id) : nextTree;
    nextTree = appendPane(nextTree, { ...promoted, focusedAt: Date.now() });
    setTree(nextTree);
    setStrip(next.strip);
    setMessage(undefined);
    setError(undefined);
  };

  const detach = async (pane: Pane) => {
    setError(undefined);
    try {
      if (props.onDetach) await props.onDetach(pane);
      else await detachPane(pane.id, pane.runId);
      setMessage(`${pane.kind} pane detached.`);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  };

  const resizePane = (
    path: readonly ("first" | "second")[],
    ratio: number,
  ) => {
    const current = tree();
    if (current) setTree(resizeAt(current, path, ratio));
  };

  return (
    <section class="pane-manager" data-testid="pane-manager">
      <header class="pane-manager-toolbar">
        <span>Focused panes</span>
        <strong data-testid="pane-focused-count">{focused().length}</strong>
        <span>Strip</span>
        <strong data-testid="pane-strip-count">{strip().length}</strong>
        <Show when={message()}>
          <output data-testid="pane-manager-message">{message()}</output>
        </Show>
        <Show when={error()}>
          <output role="alert" data-testid="pane-manager-error">
            {error()}
          </output>
        </Show>
      </header>
      <Show
        when={tree()}
        fallback={<p data-testid="pane-manager-empty">No focused panes.</p>}
      >
        <div class="pane-manager-focused">
          <PaneTreeView
            tree={tree()!}
            path={[]}
            renderPane={props.renderPane}
            onFocus={(id) => {
              const currentTree = tree();
              if (currentTree) setTree(focus(currentTree, id, Date.now()));
            }}
            onSplit={splitPane}
            onMinimize={minimizePane}
            onDetach={(pane) => void detach(pane)}
            onClose={closePane}
            onResize={resizePane}
          />
        </div>
      </Show>
      <Show when={strip().length > 0}>
        <footer class="pane-strip" data-testid="pane-strip">
          <span class="pane-strip-label">Minimized</span>
          <For each={strip()}>
            {(pane) => (
              <button
                type="button"
                class="pane-strip-entry"
                data-testid={`pane-promote-${pane.id}`}
                onClick={() => promotePane(pane.id)}
              >
                <span>{pane.kind}</span>
                <strong>{pane.id}</strong>
                <small>Promote</small>
              </button>
            )}
          </For>
        </footer>
      </Show>
    </section>
  );
}

export default PaneManager;
