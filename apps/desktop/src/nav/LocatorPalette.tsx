import { createSignal, For } from "solid-js";
import { Show } from "solid-js";
import { Sheet } from "../ui/Sheet";
import { Input } from "../ui/Input";
import { InlineError } from "../ui/InlineError";
import { Button } from "../ui/Button";
import { LOCATOR_SCHEME } from "./locator";
import {
  destinationDesktop,
  navigateDesktop,
} from "./desktop-navigation";
import type { DesktopNavTarget } from "./desktop-locator";
import { Desktop_ROUTE_KINDS } from "./desktop-route-kinds";

export function v2PaletteDestinations(project = "tapestry") {
  return Desktop_ROUTE_KINDS.map((route) => ({
    label: route.label,
    locator: route.scope === "project"
      ? destinationDesktop(route.id, project)
      : destinationDesktop(route.id),
  }));
}

export interface LocatorPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Where you are, so the field opens on it rather than empty. */
  current: string;
  /** Receives a locator resolved by the desktop navigation boundary. */
  onResolve: (target: DesktopNavTarget) => void;
}

/** ⌘K resolves a locator. ⌘P searches for one, and that lands with project-search. */
export function LocatorPalette(props: LocatorPaletteProps) {
  const [value, setValue] = createSignal(props.current);
  const [error, setError] = createSignal<string | null>(null);

  const submit = () => {
    try {
      props.onResolve(navigateDesktop(value()));
      setError(null);
      props.onOpenChange(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

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
          placeholder={`${LOCATOR_SCHEME}tapestry/session/8f21`}
          onInput={(e) => setValue(e.currentTarget.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") submit();
          }}
        />
        <div aria-label="Suggested destinations" data-testid="palette-results">
          <For each={v2PaletteDestinations()}>
            {(destination) => (
              <button
                type="button"
                onClick={() => props.onResolve(navigateDesktop(destination.locator))}
              >
                {destination.label}
              </button>
            )}
          </For>
        </div>
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
