import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { Shell } from "../../src/shell/Shell";
import { VIEWS, createNavStore } from "../../src/nav";

describe("shell/strip-persists", () => {
  it("keeps the strip through every category change", () => {
    const nav = createNavStore({ view: "inbox" });
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ));
    const first = getByTestId("strip");

    for (const v of VIEWS) {
      nav.go(v);
      expect(getByTestId("strip"), v).toBeTruthy();
      expect(
        getByTestId("strip").querySelectorAll(".strip-card").length,
        v,
      ).toBeGreaterThan(0);
    }
    // The same element throughout — it is not torn down and rebuilt per category.
    expect(getByTestId("strip")).toBe(first);
  });

  it("keeps the other three bands too — none of them is per-screen markup", () => {
    const nav = createNavStore({ view: "inbox" });
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ));
    for (const v of VIEWS) {
      nav.go(v);
      for (const band of ["titlebar", "rail", "tabbar", "strip"]) {
        expect(getByTestId(band), `${v}: ${band}`).toBeTruthy();
      }
    }
  });

  it("changes only the tab set and the lit category as the view moves", () => {
    const nav = createNavStore({ view: "board" });
    const { getByTestId } = render(() => (
      <Shell nav={nav}>
        <p>body</p>
      </Shell>
    ));
    expect(getByTestId("rail-automate").getAttribute("aria-current")).toBe(
      "true",
    );
    nav.go("telemetry");
    expect(getByTestId("rail-automate").getAttribute("aria-current")).toBe(
      null,
    );
    expect(getByTestId("rail-review").getAttribute("aria-current")).toBe(
      "true",
    );
    expect(getByTestId("tabbar-category").textContent).toBe("Review");
  });
});
