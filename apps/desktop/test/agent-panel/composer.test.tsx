import { fireEvent } from "@solidjs/testing-library";
import { expect, it, vi } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { mount, session } from "./support";

it("agent-panel/composer sends when idle, queues while running, and stops when empty", async () => {
  const onSend = vi.fn();
  const onQueue = vi.fn();
  const onStop = vi.fn();
  const idle = mount(<AgentPane runId="run-1" live={false} session={{ ...session, status: "idle" }} onSend={onSend} />);
  await fireEvent.input(idle.getByLabelText("Message agent"), { target: { value: "hello" } });
  await fireEvent.submit(idle.getByTestId("agent-composer"));
  expect(onSend).toHaveBeenCalledWith("hello");
  const running = mount(<AgentPane runId="run-2" live={false} session={session} onQueue={onQueue} onStop={onStop} />);
  await fireEvent.input(running.getByLabelText("Message agent"), { target: { value: "next turn" } });
  await fireEvent.submit(running.getByTestId("agent-composer"));
  expect(onQueue).toHaveBeenCalledWith("next turn");
  await fireEvent.click(running.getByRole("button", { name: "Stop" }));
  expect(onStop).toHaveBeenCalledTimes(1);
});
