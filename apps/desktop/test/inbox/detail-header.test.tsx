import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { InboxDetail } from "../../src/screens/inbox/InboxDetail";
import { PENDING } from "./deliveries";
import { read, rules } from "../css";

const gate = PENDING[0];
const rule = (sel: string) =>
  rules(read("screens/screens.css")).find((r) => r.selector === sel);
const mount = (item = gate) =>
  render(() => (
    <InboxDetail
      item={item}
      onApprove={() => {}}
      onSendBack={() => {}}
    />
  ));

import { configureProjectsStub } from "../projects/provider-stub";
configureProjectsStub();

describe("inbox/detail-header", () => {
  it("tags the kind in accent", () => {
    const { getByTestId } = mount();
    expect(getByTestId("inbox-detail-kind").textContent).toContain("Agent");
    expect(rule(".inbox-detail-kind")!.body).toContain(
      "color: var(--action-attention-ink)",
    );
    expect(rule(".inbox-detail-kind")!.body).toContain(
      "background: var(--action-attention)",
    );
  });

  it("sets the title at 19px/500", () => {
    const { getByTestId } = mount();
    expect(getByTestId("inbox-detail-title").textContent).toBe(gate.subject);
    expect(rule(".inbox-detail-title")!.body).toContain(
      "font-size: var(--t-title)",
    );
    expect(rule(".inbox-detail-title")!.body).toContain("font-weight: 500");
  });

  it("carries a metadata row of project · sender · held-time", () => {
    const { getByTestId } = mount();
    const meta = getByTestId("inbox-detail-meta");
    expect(meta.textContent).toContain(gate.project);
    expect(meta.textContent).toContain("agent");
    expect(meta.textContent).toContain("held");
    expect(rule(".inbox-detail-meta")!.body).toContain(
      "font-family: var(--fm)",
    );
  });
});
