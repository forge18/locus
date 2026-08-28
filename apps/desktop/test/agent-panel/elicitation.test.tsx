import { fireEvent } from "@solidjs/testing-library";
import { expect, it, vi } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { mount, session } from "./support";

it("agent-panel/elicitation validates and sends typed form values", async () => {
  const onAccept = vi.fn();
  const request = { id: "ask-typed", title: "Configure", detail: "Review the values.", fields: [{ id: "retries", label: "Retries", type: "integer" as const, required: true }, { id: "enabled", label: "Enabled", type: "boolean" as const, required: true }] };
  const view = mount(<AgentPane runId="run-1" live={false} session={session} elicitation={request} onAcceptElicitation={onAccept} />);
  await fireEvent.click(view.getByRole("button", { name: "Accept" }));
  expect(view.getByRole("alert").textContent).toContain("Retries");
  await fireEvent.input(view.getByLabelText("Retries"), { target: { value: "3" } });
  await fireEvent.click(view.getByLabelText("Enabled"));
  await fireEvent.click(view.getByRole("button", { name: "Accept" }));
  expect(onAccept).toHaveBeenCalledWith(request, { retries: 3, enabled: true });
});
