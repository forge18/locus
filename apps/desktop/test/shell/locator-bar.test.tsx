import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { LocatorBar } from "../../src/shell/LocatorBar";
import { read, rules } from "../css";

const rule = (sel: string) =>
  rules(read("shell/shell.css")).find((r) => r.selector === sel);

describe("shell/locator-bar", () => {
  it("flexes toward 520px on --sf, with a hairline and a 6px radius", () => {
    const body = rule(".locator-bar")!.body;
    expect(body).toContain("width: clamp(240px, 40vw, 520px)");
    expect(body).toContain("height: 26px");
    expect(body).toContain("background: var(--surface-raised)");
    expect(body).toContain("border: 1px solid var(--border-subtle)");
    expect(body).toContain("border-radius: var(--r-md)");
  });

  it("shows the scheme and the path as two separate spans", () => {
    const { getByTestId } = render(() => (
      <LocatorBar path="tapestry/session/8f21" />
    ));
    expect(getByTestId("locator-scheme").textContent).toBe("locus://");
    expect(getByTestId("locator-path").textContent).toBe(
      "tapestry/session/8f21",
    );
  });

  it("sets both in mono — a locator is an identifier, not prose", () => {
    expect(rule(".locator-scheme")!.body).toContain("font-family: var(--fm)");
    expect(rule(".locator-path")!.body).toContain("font-family: var(--fm)");
  });

  it("dims the scheme below the path, so the address reads first", () => {
    expect(rule(".locator-scheme")!.body).toContain("color: var(--text-muted)");
    expect(rule(".locator-path")!.body).toContain("color: var(--mu)");
  });

  it("carries the ⌘K affordance in a hairline box on the right", () => {
    const { getByTestId } = render(() => <LocatorBar path="tapestry/inbox" />);
    expect(getByTestId("locator-key").textContent).toBe("⌘K");
    const body = rule(".locator-key")!.body;
    expect(body).toContain("margin-left: auto");
    expect(body).toContain("border: 1px solid var(--border-strong)");
    expect(body).toContain("border-radius: 4px");
  });

  it("opens the palette when it is reached, rather than being decoration", () => {
    let opened = 0;
    const { getByTestId } = render(() => (
      <LocatorBar path="tapestry/inbox" onOpen={() => opened++} />
    ));
    (getByTestId("locator-bar") as HTMLElement).click();
    expect(opened).toBe(1);
  });
});
