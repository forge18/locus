import { describe, expect, it } from "vitest";
import { render, waitFor } from "@solidjs/testing-library";
import { Shell } from "../../src/shell/Shell";
import { createNavStore } from "../../src/nav";

const mount = () => {
  document.body.innerHTML = "";
  const root = document.createElement("div");
  root.id = "root";
  document.body.appendChild(root);
  const nav = createNavStore();
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

const cmdK = () =>
  document.dispatchEvent(
    new KeyboardEvent("keydown", { key: "k", metaKey: true, bubbles: true }),
  );

describe("nav/cmd-k-resolves", () => {
  it("stays shut until ⌘K", () => {
    mount();
    expect(
      document.querySelector('[data-testid="locator-palette-input"]'),
    ).toBe(null);
  });

  it("opens on ⌘K", async () => {
    mount();
    cmdK();
    await waitFor(() =>
      expect(
        document.querySelector('[data-testid="locator-palette-input"]'),
      ).not.toBe(null),
    );
  });

  it("opens on where you are, rather than empty", async () => {
    mount();
    cmdK();
    await waitFor(() =>
      expect(
        document.querySelector('[data-testid="locator-palette-input"]'),
      ).not.toBe(null),
    );
    const input = document.querySelector(
      '[data-testid="locator-palette-input"]',
    ) as HTMLInputElement;
    expect(input.value).toBe("locus://all/view/inbox");
  });

  it("resolves what is typed and navigates there", async () => {
    const { nav } = mount();
    cmdK();
    await waitFor(() =>
      expect(
        document.querySelector('[data-testid="locator-palette-input"]'),
      ).not.toBe(null),
    );
    const input = document.querySelector(
      '[data-testid="locator-palette-input"]',
    ) as HTMLInputElement;
    input.value = "locus://loom-db/view/sessions";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Enter", bubbles: true }),
    );

    await waitFor(() => expect(nav.view()).toBe("sessions"));
    expect(nav.params()).toEqual({ project: "loom-db" });
  });

  it("reports a bad locator by naming the segment, and stays put", async () => {
    const { nav } = mount();
    cmdK();
    await waitFor(() =>
      expect(
        document.querySelector('[data-testid="locator-palette-input"]'),
      ).not.toBe(null),
    );
    const input = document.querySelector(
      '[data-testid="locator-palette-input"]',
    ) as HTMLInputElement;
    input.value = "locus://tapestry/view/widget";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Enter", bubbles: true }),
    );

    await waitFor(() =>
      expect(
        document.querySelector('[data-testid="inline-error-cause"]')
          ?.textContent,
      ).toContain("route:"),
    );
    expect(nav.view()).toBe("inbox");
  });
});
