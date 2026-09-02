import { describe, expect, it } from "vitest";
import { render, waitFor } from "@solidjs/testing-library";
import { Shell } from "../../src/shell/Shell";
import { createNavStore } from "../../src/nav";
import { read } from "../css";

const mount = (view: "sessions" | "telemetry" = "sessions") => {
  document.body.innerHTML = "";
  const root = document.createElement("div");
  root.id = "root";
  document.body.appendChild(root);
  const nav = createNavStore({ view });
  const r = render(
    () => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ),
    { container: root },
  );
  return { nav, ...r };
};

describe("nav/detail-in-place", () => {
  it("opens detail as a sheet, not as a new view", async () => {
    const { nav } = mount();
    nav.openDetail("locus://weaver/artifact/a-1");
    await waitFor(() =>
      expect(document.querySelector('[data-testid="sheet"]')).not.toBe(null),
    );
    expect(nav.view()).toBe("sessions");
  });

  it("leaves the rail exactly where it was", async () => {
    const { nav, getByTestId } = mount();
    expect(getByTestId("title-category").textContent).toBe("Manage");
    nav.openDetail("locus://weaver/artifact/a-1");
    await waitFor(() =>
      expect(document.querySelector('[data-testid="sheet"]')).not.toBe(null),
    );
    expect(getByTestId("title-category").textContent).toBe("Manage");
    expect(getByTestId("project-rail")).toBeTruthy();
  });

  it("leaves the locator on the current view — you have not gone anywhere", async () => {
    const { nav } = mount();
    nav.openDetail("locus://weaver/artifact/a-1");
    await waitFor(() =>
      expect(document.querySelector('[data-testid="sheet"]')).not.toBe(null),
    );
    expect(nav.locator()).toBe("locus://all/view/sessions");
  });

  it("resolves the detail through the same resolver as everything else", async () => {
    const { nav } = mount();
    nav.openDetail("locus://weaver/artifact/a-1");
    expect(nav.detail()).toEqual({
      view: "artifact",
      params: { project: "weaver", artifactId: "a-1" },
    });
    await waitFor(() =>
      expect(
        document.querySelector('[data-testid="detail-body"]')?.textContent,
      ).toContain("a-1"),
    );
  });

  it("renders inside the app root, never as a second window", async () => {
    const { nav } = mount();
    nav.openDetail("locus://weaver/artifact/a-1");
    await waitFor(() =>
      expect(document.querySelector('[data-testid="sheet"]')).not.toBe(null),
    );
    expect(
      document
        .getElementById("root")!
        .contains(document.querySelector('[data-testid="sheet"]')!),
    ).toBe(true);
    expect(read("shell/Shell.tsx")).not.toMatch(/WebviewWindow|window\.open/);
  });

  it("closes back to where you were", async () => {
    const { nav } = mount();
    nav.openDetail("locus://weaver/artifact/a-1");
    await waitFor(() =>
      expect(document.querySelector('[data-testid="sheet"]')).not.toBe(null),
    );
    nav.closeDetail();
    await waitFor(() =>
      expect(document.querySelector('[data-testid="sheet"]')).toBe(null),
    );
    expect(nav.view()).toBe("sessions");
  });
});
