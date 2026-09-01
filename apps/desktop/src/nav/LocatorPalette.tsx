import { createEffect, createMemo, createSignal, For, Show } from "solid-js";
import { Sheet } from "../ui/Sheet";
import { Input } from "../ui/Input";
import { InlineError } from "../ui/InlineError";
import { Button } from "../ui/Button";
import { LOCATOR_SCHEME } from "./locator";
import { destinationDesktop, navigateDesktop } from "./desktop-navigation";
import type { DesktopNavTarget } from "./desktop-locator";
import { Desktop_ROUTE_KINDS } from "./desktop-route-kinds";

export const PALETTE_SECTIONS = [
  "Needs you",
  "Running now",
  "Where you were",
] as const;
export type PaletteSection = (typeof PALETTE_SECTIONS)[number];

export interface PaletteSessionState {
  project: string;
  needsAttention: boolean;
}

export interface PaletteDestination {
  label: string;
  locator: string;
  section: PaletteSection;
}

export interface PaletteState {
  current?: string;
  history?: readonly string[];
  sessions?: readonly PaletteSessionState[];
}

function sessionLocators(
  sessions: readonly PaletteSessionState[] | undefined,
  needsAttention: boolean,
): Set<string> {
  return new Set(
    (sessions ?? [])
      .filter((session) => session.needsAttention === needsAttention)
      .map((session) => destinationDesktop("sessions", session.project)),
  );
}

export function v2PaletteDestinations(
  project = "tapestry",
  state: PaletteState = {},
): PaletteDestination[] {
  const whereYouWere = new Set(
    [state.current, ...(state.history ?? [])].filter(
      (locator): locator is string => Boolean(locator),
    ),
  );
  const needsYou = sessionLocators(state.sessions, true);
  const running = sessionLocators(state.sessions, false);

  return Desktop_ROUTE_KINDS.map((route) => {
    const locator = destinationDesktop(
      route.id,
      route.scope === "project" ? project : undefined,
    );
    const section: PaletteSection = needsYou.has(locator)
      ? "Needs you"
      : running.has(locator)
        ? "Running now"
        : whereYouWere.has(locator)
          ? "Where you were"
          : route.category === "pill"
            ? "Needs you"
            : route.scope === "project"
              ? "Where you were"
              : "Running now";
    return { label: route.label, locator, section };
  });
}

export type PaletteMode = "locator" | "search";

export interface PaletteResult {
  kind: string;
  project: string;
  label: string;
  locator: string;
  score: number;
}

export interface LocatorPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  current: string;
  mode?: PaletteMode;
  /** Cmd-P delegates to the unified search_all result stream. */
  searchAll?: (query: string) => PaletteResult[];
  onResolve: (target: DesktopNavTarget) => void;
  /** Object locators use the shared NavStore resolver rather than the view-only adapter. */
  onOpenLocator?: (locator: string) => void;
  /** Navigation and session state used to keep the suggested sections live. */
  project?: string;
  history?: readonly string[];
  sessions?: readonly PaletteSessionState[];
}

export function LocatorPalette(props: LocatorPaletteProps) {
  const mode = () => props.mode ?? "locator";
  const [value, setValue] = createSignal(props.current);
  const [error, setError] = createSignal<string | null>(null);
  const [selected, setSelected] = createSignal(-1);
  const resultButtons: HTMLButtonElement[] = [];

  createEffect(() => {
    if (props.open) {
      setValue(mode() === "search" ? "" : props.current);
      setSelected(-1);
      setError(null);
    }
  });

  const project = () => {
    if (props.project) return props.project;
    const scope = props.current.slice(LOCATOR_SCHEME.length).split("/")[0];
    return scope && scope !== "all" && scope !== "app" ? scope : "tapestry";
  };
  const destinations = createMemo(() =>
    v2PaletteDestinations(project(), {
      current: props.current,
      history: props.history,
      sessions: props.sessions,
    }),
  );
  const sections = PALETTE_SECTIONS;
  const selectedDestination = () => {
    const index = selected();
    return index >= 0 ? destinations()[index] : undefined;
  };
  const openLocator = (locator: string) => {
    if (props.onOpenLocator) props.onOpenLocator(locator);
    else props.onResolve(navigateDesktop(locator));
    props.onOpenChange(false);
  };
  const searchResults = createMemo(() =>
    value() && props.searchAll ? props.searchAll(value()) : [],
  );
  const submit = () => {
    if (mode() === "search") {
      const result = searchResults()[selected()] ?? searchResults()[0];
      if (result) openLocator(result.locator);
      return;
    }
    const destination = selectedDestination();
    if (destination) {
      openLocator(destination.locator);
      return;
    }
    try {
      if (props.onOpenLocator) props.onOpenLocator(value());
      else props.onResolve(navigateDesktop(value()));
      setError(null);
      props.onOpenChange(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };
  const moveSelection = (delta: number) => {
    const count =
      mode() === "search" ? searchResults().length : destinations().length;
    if (!count) return;
    const current = selected();
    const next =
      current < 0
        ? delta > 0
          ? 0
          : count - 1
        : (current + delta + count) % count;
    setSelected(next);
    resultButtons[next]?.focus();
  };
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      moveSelection(1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      moveSelection(-1);
    } else if (e.key === "Enter") {
      e.preventDefault();
      submit();
    }
  };
  return (
    <Sheet
      open={props.open}
      onOpenChange={props.onOpenChange}
      title={mode() === "search" ? "Search everything" : "Go to locator"}
    >
      <div
        style={{
          display: "flex",
          "flex-direction": "column",
          gap: "var(--g-4)",
        }}
      >
        <Input
          mono={mode() !== "search"}
          autofocus
          data-testid="locator-palette-input"
          data-mode={mode()}
          value={value()}
          placeholder={
            mode() === "search"
              ? "Search code, wiki, tasks, and runs"
              : `${LOCATOR_SCHEME}tapestry/view/plan`
          }
          onInput={(e) => {
            setValue(e.currentTarget.value);
            setSelected(-1);
          }}
          onKeyDown={handleKeyDown}
        />
        <Show
          when={mode() === "search"}
          fallback={
            <div
              aria-label="Suggested destinations"
              data-testid="palette-results"
            >
              <For each={sections}>
                {(section) => (
                  <section>
                    <h3>{section}</h3>
                    <For
                      each={destinations().filter(
                        (destination) => destination.section === section,
                      )}
                    >
                      {(destination) => {
                        const index = () => destinations().indexOf(destination);
                        return (
                          <button
                            type="button"
                            ref={(element) => {
                              resultButtons[index()] = element;
                            }}
                            aria-selected={selected() === index()}
                            onMouseEnter={() => setSelected(index())}
                            onKeyDown={handleKeyDown}
                            onClick={() => openLocator(destination.locator)}
                          >
                            <span>{destination.label}</span>
                            <code>{destination.locator}</code>
                          </button>
                        );
                      }}
                    </For>
                  </section>
                )}
              </For>
            </div>
          }
        >
          <div aria-label="Search results" data-testid="palette-results">
            <For each={searchResults()}>
              {(result, index) => (
                <button
                  type="button"
                  ref={(element) => {
                    resultButtons[index()] = element;
                  }}
                  aria-selected={selected() === index()}
                  onMouseEnter={() => setSelected(index())}
                  onKeyDown={handleKeyDown}
                  onClick={() => openLocator(result.locator)}
                >
                  <span>{result.label}</span>
                  <small>
                    {result.kind} · {result.project}
                  </small>
                  <code>{result.locator}</code>
                </button>
              )}
            </For>
          </div>
        </Show>
        <p>
          {mode() === "search"
            ? "Searches code, wiki, tasks, and run history."
            : "Opens on a list — recognition, not recall."}
        </p>
        <small>
          ↑↓ move · ↵ {mode() === "search" ? "open result" : "open"} · esc close
        </small>
        <Show when={error()}>
          <InlineError
            cause={error()!}
            next="Fix the segment named above, or press Escape to stay where you are."
          />
        </Show>
        <Show when={mode() === "locator"}>
          <Button
            variant="primary"
            onClick={submit}
            data-testid="locator-palette-go"
          >
            Go
          </Button>
        </Show>
      </div>
    </Sheet>
  );
}
