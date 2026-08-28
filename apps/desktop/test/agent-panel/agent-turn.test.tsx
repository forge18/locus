import { expect, it } from "vitest";
import { AgentPane } from "../../src/panes/AgentPane";
import { event, mount, session } from "./support";

it("agent-panel/agent-turn renders markdown content families", () => {
  const view = mount(<AgentPane runId="run-1" live={false} session={session} events={[event("assistant", 0, { text: "answer\n\n```ts\nconst value = 1;\n```\n\n| key | value |\n| --- | --- |\n| one | two |\n\n![diagram](https://example.com/diagram.png)" })]} />);
  const turn = view.getByTestId("agent-turn");
  expect(turn.querySelector("pre")).toBeTruthy();
  expect(turn.querySelector("table")).toBeTruthy();
  expect(turn.querySelector("img")).toBeTruthy();
});
