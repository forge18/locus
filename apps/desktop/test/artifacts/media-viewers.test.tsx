import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { MemoryArtifactsFixture } from "../../src/screens/memory/MemoryFixtures";

describe("artifacts/media-viewers", () => {
  it("keeps image and recording viewers in the artifact surface", () => {
    const { getByTestId } = render(() => <MemoryArtifactsFixture />);
    const viewers = getByTestId("artifacts-media-viewers");
    expect(viewers.querySelector('[data-media-kind="image"] img')).toBeTruthy();
    expect(
      viewers.querySelector('[data-media-kind="recording"] video'),
    ).toBeTruthy();
    expect(viewers.textContent).toContain("keyframes for context");
  });
});
