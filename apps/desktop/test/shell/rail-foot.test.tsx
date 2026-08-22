import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { Rail } from "../../src/shell/Rail";
import { read, rules } from "../css";

describe("shell/rail-foot", () => {
  it("carries the branch and account glyphs", () => {
    const { getByTestId } = render(() => (
      <Rail view="inbox" onNavigate={() => {}} inboxCount={0} />
    ));
    expect(
      [...getByTestId("rail-foot").querySelectorAll("use")].map((u) =>
        u.getAttribute("href"),
      ),
    ).toEqual(["#ph-git-branch", "#ph-user-circle"]);
  });

  it("sets them in --mu2, below the inactive items", () => {
    const rule = rules(read("shell/shell.css")).find(
      (r) => r.selector === ".rail-foot",
    )!;
    expect(rule.body).toContain("color: var(--mu2)");
    expect(rule.body).toContain("margin-top: auto");
  });

  it("is not a navigation target — the foot is status, not a category", () => {
    const { getByTestId } = render(() => (
      <Rail view="inbox" onNavigate={() => {}} inboxCount={0} />
    ));
    expect(getByTestId("rail-foot").querySelectorAll("button").length).toBe(0);
  });
});
