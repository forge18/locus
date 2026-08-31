import { render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { MemoryArtifactsFixture } from "../../src/demo/MemoryFixtures";

describe("artifacts/media-viewers", () => {
  it("keeps image and recording viewers in the artifact surface", () => {
    const { getByTestId } = render(() => <MemoryArtifactsFixture />);
    const viewers = getByTestId("artifacts-media-viewers");
    const image = viewers.querySelector('[data-media-kind="image"] img');
    const video = viewers.querySelector('[data-media-kind="recording"] video');
    expect(image).toBeTruthy();
    expect(image?.getAttribute("src")).toContain("data:image/webp");
    expect(video).toBeTruthy();
    expect(video?.getAttribute("data-artifact-id")).toBe("a-4");
    expect(video?.querySelector("source")?.getAttribute("src")).toContain(
      "data:video/webm",
    );
    expect(viewers.textContent).toContain("keyframes for context");
  });
});
