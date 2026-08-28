import { describe, expect, it } from "vitest";
import { format, resolve } from "../../src/nav/locator";
import {
  destinationDesktop,
  navigateDesktop,
} from "../../src/nav/desktop-navigation";
import { resolveDesktopLocator } from "../../src/nav/desktop-locator";

describe("bots/navigation", () => {
  it("round-trips the project bot list and bot detail locators", () => {
    expect(format("bots", { project: "tapestry" })).toBe(
      "locus://tapestry/bots",
    );
    expect(resolve("locus://tapestry/bots")).toEqual({
      view: "bots",
      params: { project: "tapestry" },
    });
    expect(format("bots", { project: "tapestry", botId: "keeper" })).toBe(
      "locus://tapestry/bots/keeper",
    );
    expect(resolve("locus://tapestry/bots/keeper")).toEqual({
      view: "bots",
      params: { project: "tapestry", botId: "keeper" },
    });
  });

  it("uses one desktop resolver for the bots category and detail", () => {
    const list = destinationDesktop("bots", "tapestry");
    const detail = destinationDesktop("bots", "tapestry", "keeper");
    expect(list).toBe("locus://tapestry/bots");
    expect(detail).toBe("locus://tapestry/bots/keeper");
    expect(navigateDesktop(list).route).toBe("bots");
    expect(resolveDesktopLocator(detail)).toEqual({
      route: "bots",
      scope: { kind: "project", project: "tapestry" },
      botId: "keeper",
    });
  });
});
