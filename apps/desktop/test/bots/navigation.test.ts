import { describe, expect, it } from "vitest";
import { format, resolve } from "../../src/nav/locator";
import {
  destinationDesktop,
  navigateDesktop,
} from "../../src/nav/desktop-navigation";
import { resolveDesktopLocator } from "../../src/nav/desktop-locator";

describe("workers/navigation", () => {
  it("round-trips the all-project Worker list and Worker detail locators", () => {
    expect(format("workers", {})).toBe("locus://all/view/workers");
    expect(resolve("locus://all/view/workers")).toEqual({
      view: "workers",
      params: {},
    });
    expect(format("workers", { project: "tapestry", botId: "keeper" })).toBe(
      "locus://tapestry/workers/keeper",
    );
    expect(resolve("locus://tapestry/workers/keeper")).toEqual({
      view: "workers",
      params: { project: "tapestry", botId: "keeper" },
    });
  });

  it("uses one desktop resolver for the Workers category and detail", () => {
    const list = destinationDesktop("workers");
    const detail = destinationDesktop("workers", "tapestry", "keeper");
    expect(list).toBe("locus://all/view/workers");
    expect(detail).toBe("locus://tapestry/workers/keeper");
    expect(navigateDesktop(list).route).toBe("workers");
    expect(resolveDesktopLocator(detail)).toEqual({
      route: "workers",
      scope: { kind: "project", project: "tapestry" },
      botId: "keeper",
    });
  });
});
