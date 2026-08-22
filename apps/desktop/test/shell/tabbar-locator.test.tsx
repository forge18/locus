import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { TabBar } from "../../src/shell/TabBar";
import { read, rules } from "../css";

describe("shell/tabbar-locator", () => {
  it("shows the current view locator on the right", () => {
    const { getByTestId } = render(() => (
      <TabBar view="runs" onNavigate={() => {}} locator="tapestry/run/8f21" />
    ));
    expect(getByTestId("tabbar-locator").textContent).toContain(
      "tapestry/run/8f21",
    );
  });

  it("sets it in mono, pushed right", () => {
    const rule = rules(read("shell/shell.css")).find(
      (r) => r.selector === ".tabbar-locator",
    )!;
    expect(rule.body).toContain("font-family: var(--fm)");
    expect(rule.body).toContain("margin-left: auto");
    expect(rule.body).toContain("color: var(--mu2)");
  });

  it("offers the detach affordance only when detaching is possible", () => {
    const without = render(() => (
      <TabBar view="runs" onNavigate={() => {}} locator="x" />
    ));
    expect(without.getByTestId("tabbar-locator").querySelector("use")).toBe(
      null,
    );

    const withDetach = render(() => (
      <TabBar
        view="runs"
        onNavigate={() => {}}
        locator="x"
        onDetach={() => {}}
      />
    ));
    expect(
      withDetach
        .getByTestId("tabbar-locator")
        .querySelector("use")!
        .getAttribute("href"),
    ).toBe("#ph-arrows-out-simple");
  });

  it("detaches through the caller — a second window is never opened here", () => {
    let detached = 0;
    const { getByLabelText } = render(() => (
      <TabBar
        view="runs"
        onNavigate={() => {}}
        locator="x"
        onDetach={() => detached++}
      />
    ));
    getByLabelText("Detach").click();
    expect(detached).toBe(1);
    expect(read("shell/TabBar.tsx")).not.toMatch(/WebviewWindow|window\.open/);
  });
});
