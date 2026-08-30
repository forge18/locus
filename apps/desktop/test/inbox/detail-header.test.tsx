import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { InboxDetail } from "../../src/screens/inbox/InboxDetail";
import { useInboxItems } from "../../src/data/inbox";
import { read, rules } from "../css";

const [gate] = useInboxItems();
const rule = (sel: string) =>
  rules(read("screens/screens.css")).find((r) => r.selector === sel);
const mount = (item = gate) =>
  render(() => (
    <InboxDetail
      item={item}
      onApprove={() => {}}
      onSendBack={() => {}}
      onOpenWork={() => {}}
    />
  ));

import { configureProjectsStub } from "../projects/provider-stub";
configureProjectsStub();

describe("inbox/detail-header", () => {
  it("tags the kind in accent", () => {
    const { getByTestId } = mount();
    expect(getByTestId("inbox-detail-kind").textContent).toBe("gate");
    expect(rule(".inbox-detail-kind")!.body).toContain(
      "color: var(--action-attention)",
    );
    expect(rule(".inbox-detail-kind")!.body).toContain(
      "background: var(--action-attention-wash)",
    );
  });

  it("sets the title at 19px/500", () => {
    const { getByTestId } = mount();
    expect(getByTestId("inbox-detail-title").textContent).toBe(gate.title);
    expect(rule(".inbox-detail-title")!.body).toContain(
      "font-size: var(--t-title)",
    );
    expect(rule(".inbox-detail-title")!.body).toContain("font-weight: 500");
  });

  it("carries a mono metadata row of locator · agent · role · gate", () => {
    const { getByTestId } = mount();
    const meta = getByTestId("inbox-detail-meta");
    expect(meta.textContent).toContain(gate.opensAt);
    expect(meta.textContent).toContain("planner@3");
    expect(meta.textContent).toContain("planner");
    expect(meta.textContent).toContain("Gate: human");
    expect(rule(".inbox-detail-meta")!.body).toContain(
      "font-family: var(--fm)",
    );
  });

  it("says the gate is the agent when the item is not a human gate", () => {
    const { getByTestId } = mount(useInboxItems()[2]);
    expect(getByTestId("inbox-detail-meta").textContent).toContain(
      "Gate: agent",
    );
  });

  it("shows the locator, which is how the work is opened later", () => {
    const { getByTestId } = mount();
    expect(getByTestId("inbox-detail-locator").textContent).toBe(gate.opensAt);
  });
});
