import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { TabBar } from "../../src/shell/TabBar";
import { read, rules } from "../css";

const rule = (sel: string) =>
  rules(read("shell/shell.css")).find((r) => r.selector === sel);

describe("shell/tabbar", () => {
  it("carries a bottom hairline over the surface-to-ground gradient", () => {
    const body = rule(".tabbar")!.body;
    expect(body).toContain("height: 40px");
    expect(body).toContain("background: linear-gradient(var(--surface-raised), var(--surface-ground))");
    expect(body).toContain("border-bottom: 1px solid var(--border-subtle)");
  });

  it("leads with the current category label", () => {
    const { getByTestId } = render(() => (
      <TabBar
        view="telemetry"
        onNavigate={() => {}}
        locator="tapestry/telemetry"
      />
    ));
    expect(getByTestId("tabbar-category").textContent).toBe("Review");
  });

  it("sets the label at 19px/500 uppercase with .1em tracking in --mu", () => {
    const body = rule(".tabbar-category")!.body;
    expect(body).toContain("font-size: var(--t-row)");
    expect(body).toContain("font-weight: 500");
    expect(body).toContain("letter-spacing: .1em");
    expect(body).toContain("text-transform: uppercase");
    expect(body).toContain("color: var(--mu)");
  });

  it('names the dashboard category "Inbox", as the rail does', () => {
    const { getByTestId } = render(() => (
      <TabBar view="status" onNavigate={() => {}} locator="tapestry/status" />
    ));
    expect(getByTestId("tabbar-category").textContent).toBe("Inbox");
  });
});
