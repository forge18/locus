import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => {
  const invoke = vi.fn();
  class Channel {
    onmessage: (event: unknown) => void = () => undefined;
  }
  return { Channel, invoke };
});

vi.mock("@tauri-apps/api/core", () => ({
  Channel: mocks.Channel,
  invoke: mocks.invoke,
}));

import { attachTauriLsp } from "../../src/editor/tauriLsp";

describe("editor/tauri-lsp", () => {
  beforeEach(() => {
    mocks.invoke.mockReset();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "lsp_attach") {
        return Promise.resolve({
          projectRoot: "/workspace",
          paneId: "pane-1",
          descriptorId: "rust",
        });
      }
      if (command === "lsp_diagnostics_subscribe") return Promise.resolve(7);
      return Promise.resolve(undefined);
    });
  });

  it("unsubscribes diagnostics before detaching and ignores stale callbacks", async () => {
    const onDiagnostics = vi.fn();
    const supervisor = await attachTauriLsp({
      projectRoot: "/workspace",
      paneId: "pane-1",
      filePath: "src/main.rs",
      onDiagnostics,
    });
    await supervisor.dispose();

    expect(mocks.invoke.mock.calls.map(([command]) => command)).toEqual([
      "lsp_attach",
      "lsp_diagnostics_subscribe",
      "lsp_diagnostics_unsubscribe",
      "lsp_detach",
    ]);
    expect(onDiagnostics).not.toHaveBeenCalled();
  });

  it("hydrates persisted project descriptors before attaching", async () => {
    const supervisor = await attachTauriLsp({
      projectRoot: "/workspace",
      projectId: "project-1",
      paneId: "pane-1",
      filePath: "src/main.rs",
    });
    await supervisor.dispose();

    expect(mocks.invoke.mock.calls.map(([command]) => command)).toEqual([
      "lsp_load_project_descriptors",
      "lsp_attach",
      "lsp_diagnostics_subscribe",
      "lsp_diagnostics_unsubscribe",
      "lsp_detach",
    ]);
  });

  it("detaches an already attached pane when diagnostics subscription fails", async () => {
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "lsp_attach") {
        return Promise.resolve({
          projectRoot: "/workspace",
          paneId: "pane-1",
          descriptorId: "rust",
        });
      }
      if (command === "lsp_diagnostics_subscribe") {
        return Promise.reject(new Error("channel unavailable"));
      }
      return Promise.resolve(undefined);
    });

    await expect(
      attachTauriLsp({
        projectRoot: "/workspace",
        paneId: "pane-1",
        filePath: "src/main.rs",
      }),
    ).rejects.toThrow("channel unavailable");
    expect(mocks.invoke.mock.calls.map(([command]) => command)).toEqual([
      "lsp_attach",
      "lsp_diagnostics_subscribe",
      "lsp_detach",
    ]);
  });
});
