import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { Shell } from "../../src/shell/Shell";
import { createNavStore } from "../../src/nav";
import { read, rules } from "../css";

const css = read("shell/shell.css");
const rule = (sel: string) => rules(css).find((r) => r.selector === sel);

const mount = () =>
  render(() => (
    <Shell nav={createNavStore()}>
      <p>screen body</p>
    </Shell>
  ));

describe("shell/frame", () => {
  it("fills its host rather than being a 1440x900 picture of one", () => {
    const win = rule(".window")!.body;
    expect(win).toContain("width: 100%");
    expect(win).toContain("height: 100%");
    expect(win).not.toContain("1440px");
    expect(win).not.toContain("900px");
  });

  it("is a column flex on --bg", () => {
    const win = rule(".window")!.body;
    expect(win).toContain("flex-direction: column");
    expect(win).toContain("background: var(--bg)");
  });

  it("renders the desktop title bar and project-scoped rail", () => {
    const { getByTestId } = mount();
    expect(getByTestId("app-titlebar")).toBeTruthy();
    expect(getByTestId("project-rail")).toBeTruthy();
    expect(getByTestId("selected-project-card")).toBeTruthy();
  });

  it("gives the screen its own body beside the project rail", () => {
    const { getByTestId } = mount();
    expect(getByTestId("screen").textContent).toBe("screen body");
  });

  it("scrolls the screen, not the window — the bands never move", () => {
    expect(rule(".window")!.body).toContain("overflow: hidden");
    expect(rule(".screen")!.body).toContain("overflow: auto");
  });
});
