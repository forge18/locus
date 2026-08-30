import { render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { InboxDetail } from "../../src/screens/inbox/InboxDetail";
import { InboxView } from "../../src/screens/inbox/InboxView";
import { createNavStore } from "../../src/nav";
import { PENDING } from "./deliveries";
import { configureInboxStub } from "./inbox-stub";

const [gate, second] = PENDING;

/** Same contract comment-box.test.tsx uses: set the value, then let Solid see it. */
const typeInto = (box: Element, text: string) => {
  const textarea = box as HTMLTextAreaElement;
  textarea.value = text;
  textarea.dispatchEvent(new Event("input", { bubbles: true }));
};

interface DetailHandlers {
  onApprove?: (comment: string) => void;
  onSendBack?: (comment: string) => void;
}

const mountDetail = (handlers: DetailHandlers = {}) => {
  const seen = { approve: null as string | null, sendBack: null as string | null };
  const r = render(() => (
    <InboxDetail
      item={gate}
      onApprove={(comment) => {
        seen.approve = comment;
        handlers.onApprove?.(comment);
      }}
      onSendBack={(comment) => {
        seen.sendBack = comment;
        handlers.onSendBack?.(comment);
      }}
    />
  ));
  return { seen, ...r };
};

const mountView = () => render(() => <InboxView nav={createNavStore()} />);


configureInboxStub();

describe("inbox/send-back — the detail pane", () => {
  it("blocks an empty comment and says why", () => {
    const { seen, getByTestId } = mountDetail();
    getByTestId("inbox-send-back").click();
    expect(seen.sendBack).toBe(null);
    const error = getByTestId("inbox-send-back-error");
    expect(error.getAttribute("role")).toBe("alert");
    expect(error.textContent).toContain("Write a comment");
    expect(getByTestId("inbox-comment").getAttribute("aria-invalid")).toBe(
      "true",
    );
  });

  it("blocks a whitespace-only comment too", () => {
    const { seen, getByTestId } = mountDetail();
    typeInto(getByTestId("inbox-comment"), "   \n  ");
    getByTestId("inbox-send-back").click();
    expect(seen.sendBack).toBe(null);
    expect(getByTestId("inbox-send-back-error")).toBeTruthy();
  });

  it("clears the block once text arrives", () => {
    const { seen, getByTestId, queryByTestId } = mountDetail();
    getByTestId("inbox-send-back").click();
    expect(getByTestId("inbox-send-back-error")).toBeTruthy();
    typeInto(getByTestId("inbox-comment"), "Split the sink work out.");
    expect(queryByTestId("inbox-send-back-error")).toBe(null);
    expect(
      getByTestId("inbox-comment").getAttribute("aria-invalid"),
    ).toBe(null);
    getByTestId("inbox-send-back").click();
    expect(seen.sendBack).toBe("Split the sink work out.");
  });

  it("hands what was typed to the send-back action", () => {
    const { seen, getByTestId } = mountDetail();
    typeInto(getByTestId("inbox-comment"), "Keep the HTTP sink out of scope.");
    getByTestId("inbox-send-back").click();
    expect(seen.sendBack).toBe("Keep the HTTP sink out of scope.");
  });

  it("approves without text while send-back demands it — two paths, not one", () => {
    const { seen, getByTestId } = mountDetail();
    getByTestId("inbox-approve").click();
    expect(seen.approve).toBe("");
    expect(seen.sendBack).toBe(null);
  });
});

describe("inbox/send-back — the view wiring", () => {
  it("resolves a send-back that carries its comment", async () => {
    const { getByTestId, queryByTestId } = mountView();
    await waitFor(() => getByTestId("inbox-comment"));
    typeInto(getByTestId("inbox-comment"), "  Split the sink work out.  ");
    getByTestId("inbox-send-back").click();
    await waitFor(() =>
      expect(queryByTestId(`inbox-card-${gate.id}`)).toBe(null),
    );
  });

  it("keeps the reason in the completed record — the decision is auditable", async () => {
    const { getByTestId } = mountView();
    await waitFor(() => getByTestId("inbox-comment"));
    typeInto(getByTestId("inbox-comment"), "  Split the sink work out.  ");
    getByTestId("inbox-send-back").click();
    getByTestId("inbox-tab-completed").click();
    await waitFor(() =>
      expect(
        getByTestId(`resolved-decision-${gate.id}`).textContent,
      ).toBe("Sent back: Split the sink work out."),
    );
  });

  it("records no decision line when a comment-less approve resolves", async () => {
    const { getByTestId, queryByTestId } = mountView();
    await waitFor(() => getByTestId("inbox-approve"));
    getByTestId("inbox-approve").click();
    await waitFor(() =>
      expect(getByTestId("inbox-todo-count").textContent).toBe("2"),
    );
    getByTestId("inbox-tab-completed").click();
    expect(queryByTestId(`resolved-decision-${gate.id}`)).toBe(null);
  });

  it("starts the next item with a clean comment — a draft never steers the wrong agent", async () => {
    const { getByTestId } = mountView();
    await waitFor(() =>
      expect(getByTestId("inbox-detail-title").textContent).toContain(gate.subject),
    );
    typeInto(getByTestId("inbox-comment"), "Only about the first item.");
    getByTestId("inbox-approve").click();
    await waitFor(() =>
      expect(getByTestId("inbox-detail-title").textContent).toContain(second.subject),
    );
    expect((getByTestId("inbox-comment") as HTMLTextAreaElement).value).toBe("");
  });
});
