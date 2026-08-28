import { createSignal, For, Show } from "solid-js";
import { Sheet } from "../ui/Sheet";
import { Input } from "../ui/Input";
import { InlineError } from "../ui/InlineError";
import { Button } from "../ui/Button";
import { LOCATOR_SCHEME } from "./locator";
import { destinationDesktop, navigateDesktop } from "./desktop-navigation";
import type { DesktopNavTarget } from "./desktop-locator";
import { Desktop_ROUTE_KINDS } from "./desktop-route-kinds";

export function v2PaletteDestinations(project = "tapestry") {
  return Desktop_ROUTE_KINDS.map((route) => ({
    label: route.label,
    locator: destinationDesktop(
      route.id,
      route.scope === "project" ? project : undefined,
    ),
    section:
      route.category === "pill"
        ? "Needs you"
        : route.scope === "project"
          ? "Where you were"
          : "Running now",
  }));
}

export interface LocatorPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  current: string;
  onResolve: (target: DesktopNavTarget) => void;
  /** Object locators use the shared NavStore resolver rather than the view-only adapter. */
  onOpenLocator?: (locator: string) => void;
}

export function LocatorPalette(props: LocatorPaletteProps) {
  const [value, setValue] = createSignal(props.current);
  const [error, setError] = createSignal<string | null>(null);
  const submit = () => {
    try {
      if (props.onOpenLocator) props.onOpenLocator(value());
      else props.onResolve(navigateDesktop(value()));
      setError(null);
      props.onOpenChange(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };
  const destinations = v2PaletteDestinations();
  const sections = ["Needs you", "Running now", "Where you were"] as const;
  return (
    <Sheet
      open={props.open}
      onOpenChange={props.onOpenChange}
      title="Go to locator"
    >
      <div
        style={{
          display: "flex",
          "flex-direction": "column",
          gap: "var(--g-4)",
        }}
      >
        <Input
          mono
          autofocus
          data-testid="locator-palette-input"
          value={value()}
          placeholder={`${LOCATOR_SCHEME}tapestry/view/plan`}
          onInput={(e) => setValue(e.currentTarget.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") submit();
          }}
        />
        <div aria-label="Suggested destinations" data-testid="palette-results">
          <For each={sections}>
            {(section) => (
              <section>
                <h3>{section}</h3>
                <For
                  each={destinations.filter(
                    (destination) => destination.section === section,
                  )}
                >
                  {(destination) => (
                    <button
                      type="button"
                      onClick={() => {
                        props.onResolve(navigateDesktop(destination.locator));
                        props.onOpenChange(false);
                      }}
                    >
                      <span>{destination.label}</span>
                      <code>{destination.locator}</code>
                    </button>
                  )}
                </For>
              </section>
            )}
          </For>
        </div>
        <p>Opens on a list — recognition, not recall.</p>
        <small>↑↓ move · ↵ open · ⇧↵ scope · esc close</small>
        <Show when={error()}>
          <InlineError
            cause={error()!}
            next="Fix the segment named above, or press Escape to stay where you are."
          />
        </Show>
        <Button
          variant="primary"
          onClick={submit}
          data-testid="locator-palette-go"
        >
          Go
        </Button>
      </div>
    </Sheet>
  );
}
