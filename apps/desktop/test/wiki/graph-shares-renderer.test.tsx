import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { MemoryWikiFixture } from "../../src/demo/MemoryFixtures";

describe("wiki/graph-shares-renderer", () => {
  it("uses the shared GraphRenderer for Wiki links", () => {
    const { getByTestId } = render(() => <MemoryWikiFixture />);
    const graph = getByTestId("wiki-graph-renderer");
    expect(graph.querySelector("svg")).toBeTruthy();
    expect(graph.querySelectorAll(".graph-edge")).toHaveLength(3);
    expect(
      graph.querySelector('[data-testid="graph-node-bare-remote"]'),
    ).toBeTruthy();
  });
});
