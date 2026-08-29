import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { MailView } from "../../src/screens/mail/MailView";
import { MAIL_WAIT_BANNER } from "../../src/data/mail";

const mount = () => render(() => <MailView />);
type View = ReturnType<typeof mount>;

const composer = (view: View) =>
  view.getByTestId("mail-thread-view").querySelector("textarea") as HTMLTextAreaElement;

const threadButton = (view: View, id: string) =>
  view.getByTestId(`mail-thread-${id}`) as HTMLButtonElement;

const tabButton = (view: View, label: string) =>
  [...view.container.querySelectorAll<HTMLButtonElement>(".mail-tab")].find(
    (button) => button.textContent === label,
  )!;

describe("screens/desktop-mail", () => {
  it("renders the three-pane fixture with the waiting thread selected", () => {
    const view = mount();

    expect(view.getByTestId("mail").getAttribute("data-three-pane")).toBe("true");
    expect(threadButton(view, "thread-1").getAttribute("aria-selected")).toBe("true");
    expect(threadButton(view, "thread-1").getAttribute("data-status")).toBe("waiting");
    expect(view.getByTestId("mail-wait-banner").textContent).toContain(MAIL_WAIT_BANNER);
    expect(view.container.querySelectorAll(".mail-message")).toHaveLength(4);
  });

  it("keeps the reply draft controlled and sends it as mail reply from you", async () => {
    const view = mount();
    const textarea = composer(view);

    await fireEvent.input(textarea, { target: { value: "Payload stays id-only on my side." } });
    expect(textarea.value).toBe("Payload stays id-only on my side.");

    await fireEvent.click(view.getByTestId("mail-send"));

    const replies = [
      ...view.container.querySelectorAll<HTMLElement>('.mail-message[data-verb="reply"]'),
    ];
    expect(replies).toHaveLength(2); // the fixture reply plus the one just sent
    expect(replies[1]!.querySelector("strong")!.textContent).toBe("you");
    expect(replies[1]!.querySelector("p")!.textContent).toBe(
      "Payload stays id-only on my side.",
    );
    expect(textarea.value).toBe("");
  });

  it("refuses an empty reply with an inline error instead of a silent no-op", async () => {
    const view = mount();
    const before = view.container.querySelectorAll(".mail-message").length;

    await fireEvent.click(view.getByTestId("mail-send"));

    expect(view.getByTestId("inline-error").getAttribute("role")).toBe("alert");
    expect(view.getByTestId("inline-error-cause").textContent).toContain("Write a reply");
    expect(view.container.querySelectorAll(".mail-message")).toHaveLength(before);

    await fireEvent.input(composer(view), { target: { value: "now it has a body" } });
    expect(view.queryByTestId("inline-error")).toBeNull();
  });

  it("drains the selected thread into a handoff and locks the composer", async () => {
    const view = mount();

    await fireEvent.click(view.getByTestId("mail-drain"));

    expect(threadButton(view, "thread-1").getAttribute("data-status")).toBe("drained");
    expect(view.getByTestId("mail-handoff-artifact")).toBeTruthy();
    expect(view.queryByTestId("mail-wait-banner")).toBeNull();
    expect(view.queryByTestId("mail-unblock")).toBeNull();
    expect(composer(view).disabled).toBe(true);
    expect(view.getByTestId("mail-send").hasAttribute("disabled")).toBe(true);
    expect(view.getByTestId("mail-drain").hasAttribute("disabled")).toBe(true);
  });

  it("unblocks a waiting thread, returning it to open", async () => {
    const view = mount();

    expect(view.getByTestId("mail-unblock")).toBeTruthy();
    await fireEvent.click(view.getByTestId("mail-unblock"));

    expect(view.queryByTestId("mail-unblock")).toBeNull();
    expect(view.queryByTestId("mail-wait-banner")).toBeNull();
    expect(threadButton(view, "thread-1").getAttribute("data-status")).toBe("open");
    expect(composer(view).disabled).toBe(false);
    expect(view.getByTestId("mail-send").hasAttribute("disabled")).toBe(false);

    await fireEvent.click(tabButton(view, "Waiting"));
    expect(view.queryByTestId("mail-thread-thread-1")).toBeNull();
  });

  it("keeps the drained fixture thread read-only with its handoff artifact", async () => {
    const view = mount();

    await fireEvent.click(threadButton(view, "thread-5"));

    expect(threadButton(view, "thread-5").getAttribute("aria-selected")).toBe("true");
    expect(view.getByTestId("mail-handoff-artifact")).toBeTruthy();
    expect(composer(view).disabled).toBe(true);
    expect(composer(view).placeholder).toContain("handoff");
    expect(view.getByTestId("mail-send").hasAttribute("disabled")).toBe(true);
  });

  it("clears the draft and any error when switching threads", async () => {
    const view = mount();

    await fireEvent.click(view.getByTestId("mail-send"));
    expect(view.getByTestId("inline-error")).toBeTruthy();
    await fireEvent.input(composer(view), { target: { value: "half-written" } });
    await fireEvent.click(threadButton(view, "thread-2"));

    expect(composer(view).value).toBe("");
    expect(view.queryByTestId("inline-error")).toBeNull();
    expect(view.queryByTestId("mail-wait-banner")).toBeNull();
    expect(view.queryByTestId("mail-unblock")).toBeNull();
  });
});
