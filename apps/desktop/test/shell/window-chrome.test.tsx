import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => {
  const calls: string[] = [];
  const win = {
    close: vi.fn(() => {
      calls.push("close");
      return Promise.resolve();
    }),
    minimize: vi.fn(() => {
      calls.push("minimize");
      return Promise.resolve();
    }),
    toggleMaximize: vi.fn(() => {
      calls.push("toggleMaximize");
      return Promise.resolve();
    }),
  };
  return { calls, getCurrentWindow: vi.fn(() => win) };
});

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: mocks.getCurrentWindow,
}));

import { render } from "@solidjs/testing-library";
import { fireEvent } from "@solidjs/testing-library";
import { AppTitleBar } from "../../src/shell/AppTitleBar";
import { configureProjectsStub } from "../projects/provider-stub";
import { read, rules } from "../css";

const railRule = (sel: string) =>
  rules(read("shell/shell.css")).find((r) => r.selector === sel);

describe("shell/window-chrome", () => {
  it("native decorations are off — the custom title bar is the only chrome", () => {
    const conf = JSON.parse(
      readFileSync(
        resolve(__dirname, "../../src-tauri/tauri.conf.json"),
        "utf8",
      ),
    );
    expect(conf.app.windows[0].decorations).toBe(false);
  });

  it("the title bar surface is a drag region", () => {
    configureProjectsStub();
    const { getByTestId } = render(() => (
      <AppTitleBar categoryLabel="Project" viewLabel="Setup" running={0} needsYou={0} />
    ));
    expect(
      getByTestId("app-titlebar").hasAttribute("data-tauri-drag-region"),
    ).toBe(true);
  });

  it("the traffic lights are real controls that call the window API", () => {
    configureProjectsStub();
    const { getByTestId } = render(() => (
      <AppTitleBar categoryLabel="Project" viewLabel="Setup" running={0} needsYou={0} />
    ));

    fireEvent.click(getByTestId("window-minimize"));
    fireEvent.click(getByTestId("window-maximize"));
    fireEvent.click(getByTestId("window-close"));

    expect(mocks.calls).toEqual(["minimize", "toggleMaximize", "close"]);
  });

  it("the rail buttons carry the shared control reset and tokens", () => {
    const reset = railRule(".project-rail button")!.body;
    expect(reset).toContain("appearance: none");
    expect(reset).toContain("background: transparent");
    expect(reset).toContain("border: 0");
    expect(reset).toContain("color: var(--text-secondary)");
    const current = railRule('.project-rail button[aria-current="true"]')!.body;
    expect(current).toContain("background: var(--surface-selected)");
  });
});
