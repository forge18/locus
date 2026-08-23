import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { TitleBar } from "../../src/shell/TitleBar";
import { read, rules } from "../css";

const css = read("shell/shell.css");
const rule = (sel: string) => rules(css).find((r) => r.selector === sel);

const mount = () =>
  render(() => (
    <TitleBar
      locatorPath="tapestry/session/8f21"
      projects={[{ id: "p", name: "tapestry" }]}
      selectedProjects={[]}
      onProjectsChange={() => {}}
      runningCount={8}
    />
  ));

describe("shell/titlebar", () => {
  it("sits on the deep ground with a bottom hairline", () => {
    const body = rule(".titlebar")!.body;
    expect(body).toContain("height: 42px");
    expect(body).toContain("background: var(--surface-chrome)");
    expect(body).toContain("border-bottom: 1px solid var(--border-subtle)");
  });

  it("draws three traffic lights in the macOS colors", () => {
    const { getByTestId } = mount();
    expect(getByTestId("traffic-lights").querySelectorAll("span").length).toBe(
      3,
    );
    expect(rule(".traffic-close")!.body).toContain("var(--tl-close)");
    expect(rule(".traffic-min")!.body).toContain("var(--tl-min)");
    expect(rule(".traffic-max")!.body).toContain("var(--tl-max)");
  });

  it("sets the wordmark at 19px/500 uppercase with .14em tracking", () => {
    const { getByTestId } = mount();
    expect(getByTestId("wordmark").textContent).toBe("Locus");
    const body = rule(".wordmark")!.body;
    expect(body).toContain("font-size: var(--t-row)");
    expect(body).toContain("font-weight: 500");
    expect(body).toContain("letter-spacing: .14em");
    expect(body).toContain("text-transform: uppercase");
    expect(body).toContain("color: var(--mu)");
  });

  it("holds the locator bar, the project filter and the running count", () => {
    const { getByTestId } = mount();
    expect(getByTestId("locator-bar")).toBeTruthy();
    expect(getByTestId("project-filter")).toBeTruthy();
    expect(getByTestId("running-count")).toBeTruthy();
  });
});
