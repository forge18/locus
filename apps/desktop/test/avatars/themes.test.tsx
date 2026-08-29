import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { applyTheme } from "../../src/styles/theme";
import BotsView from "../../src/screens/bots/BotsView";
import { read } from "../css";

describe("avatars/themes", () => {
  it("keeps the same transparent avatar under both installed themes", () => {
    const view = render(() => <BotsView />);
    const avatar = view.getByTestId("bot-avatar-keeper");
    const source = avatar.getAttribute("src");

    applyTheme(document.documentElement, "dark");
    expect(avatar.getAttribute("src")).toBe(source);
    applyTheme(document.documentElement, "light");
    expect(avatar.getAttribute("src")).toBe(source);
    expect(read("screens/bots/bots.css")).toContain("background: transparent");
  });
});
