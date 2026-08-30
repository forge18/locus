import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { Strip } from "../../src/shell/Strip";
import { fetchStripCards } from "../../src/data/strip";
import { configureProjectsStub } from "../projects/provider-stub";
import { read, rules } from "../css";

const rule = (sel: string) =>
  rules(read("shell/shell.css")).find((r) => r.selector === sel);

describe("shell/strip", () => {
  it("is a footer on the deep ground with a top hairline", () => {
    const body = rule(".strip")!.body;
    expect(body).toContain("height: 54px");
    expect(body).toContain("background: var(--surface-chrome)");
    expect(body).toContain("border-top: 1px solid var(--border-subtle)");
  });

  it("carries the vertical STRIP label", () => {
    configureProjectsStub();
    const { getByTestId } = render(() => <Strip cards={[]} />);
    expect(getByTestId("strip-label").textContent).toBe("Strip");
    expect(rule(".strip-label")!.body).toContain("writing-mode: vertical-rl");
    expect(rule(".strip-label")!.body).toContain("text-transform: uppercase");
  });

  it("draws one card per live running agent", async () => {
    configureProjectsStub({
      stripCards: [
        { id: "run-1", project: "tapestry", agent: "builder", status: "running", startedEpoch: Math.floor(Date.now() / 1000) },
        { id: "run-2", project: "loom-db", agent: "builder", status: "running", startedEpoch: Math.floor(Date.now() / 1000) - 120 },
      ],
    });
    const envelope = await fetchStripCards();
    expect(envelope.status).toBe("ready");
    const cards = envelope.status === "ready" ? envelope.data : [];
    const { getByTestId } = render(() => <Strip cards={cards} />);
    expect(getByTestId("strip").querySelectorAll(".strip-card").length).toBe(
      cards.length,
    );
  });

  it("states its own ordering on the right, so the order is not a mystery", () => {
    const { getByTestId } = render(() => <Strip cards={[]} />);
    expect(getByTestId("strip").textContent).toContain(
      "sorted by needs-attention, then activity",
    );
  });
});
