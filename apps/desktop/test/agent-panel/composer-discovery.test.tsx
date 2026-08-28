import { fireEvent } from "@solidjs/testing-library";
import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { mount, session } from "./support";

it("agent-panel/composer-discovery offers slash commands and mentions", async () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} />);
  await fireEvent.input(view.getByLabelText("Message agent"), { target: { value: "/" } });
  expect(view.getAllByRole("option", { name: /New session/ }).length).toBeGreaterThan(0);
  await fireEvent.input(view.getByLabelText("Message agent"), { target: { value: "@" } });
  expect(view.getByRole("option", { name: "@file" })).toBeTruthy();
});
