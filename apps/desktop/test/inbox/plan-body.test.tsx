import { describe, expect, it } from "vitest";
import { render } from "@solidjs/testing-library";
import { InboxDetail } from "../../src/screens/inbox/InboxDetail";
import { PENDING } from "./deliveries";
import { configureInboxStub } from "./inbox-stub";
import { read, rules } from "../css";

const [gate] = PENDING;
const rule = (sel: string) => rules(read("screens/screens.css")).find((r) => r.selector === sel);
const mount = (item = gate) =>
  render(() => (
    <InboxDetail item={item} onApprove={() => {}} onSendBack={() => {}} />
  ));

configureInboxStub();

describe("inbox/plan-body", () => {
  it("labels the body as a message, in accent uppercase", () => {
    const { getByTestId } = mount();
    expect(getByTestId("inbox-body-label").textContent).toBe("Message");
    expect(rule(".inbox-body-label")!.body).toContain("color: var(--action-attention)");
    expect(rule(".inbox-body-label")!.body).toContain("text-transform: uppercase");
  });

  it("renders the delivery body verbatim", () => {
    const { getByTestId } = mount();
    expect(getByTestId("inbox-steps").textContent).toBe(gate.body);
  });

  it("sets them at 15px on a 1.6 line", () => {
    const body = rule(".inbox-steps")!.body;
    expect(body).toContain("font-size: var(--t-row)");
    expect(body).toContain("line-height: 1.6");
  });

  it("carries the comment caption under the body", () => {
    const { getByTestId } = mount();
    expect(getByTestId("inbox-comment-caption").textContent).toContain(
      "Comment steers the agent",
    );
  });
});
