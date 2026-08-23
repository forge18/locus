import { render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";

const { invoke, isTauri } = vi.hoisted(() => ({
  invoke: vi.fn(),
  isTauri: vi.fn(() => true),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke, isTauri }));

import { ExtensionsView } from "../../src/screens/workshop/ExtensionsView";

describe("extensions/load-error", () => {
  it("surfaces a real IPC failure on the surface that failed", async () => {
    invoke.mockRejectedValueOnce(new Error("daemon unreachable"));

    const { getByTestId } = render(() => (
      <ExtensionsView onNavigate={() => {}} />
    ));

    await waitFor(() => expect(getByTestId("extensions-error")).toBeTruthy());
    expect(getByTestId("inline-error-cause").textContent).toContain(
      "daemon unreachable",
    );
  });
});
