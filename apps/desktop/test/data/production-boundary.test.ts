import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));

import { demoProvider } from "../../src/data/demo/demo-provider";
import {
  configureDataProvider,
  dataProvider,
  liveProvider,
} from "../../src/data/provider";

const read = (path: string) =>
  readFileSync(resolve(__dirname, "../../src/data", path), "utf8");

describe("data/production-boundary", () => {
  beforeEach(() => {
    mocks.invoke.mockReset();
  });

  it("refuses to serve before an explicit provider is configured", () => {
    // Fresh module instance: vitest isolates per file, and this test is first.
    expect(() => dataProvider()).toThrow(/configureDataProvider/);
    configureDataProvider(liveProvider);
    expect(dataProvider().kind).toBe("live");
  });

  it("never reaches the demo provider except through its explicit demo module", () => {
    // The seam's imports reference neither fixtures nor the demo provider, so a
    // Tauri runtime cannot silently serve demo data through it.
    const imports = [
      ...read("provider.ts").matchAll(/from\s+["']([^"']+)['"]/g),
    ].map((entry) => entry[1]);
    expect(imports).toContain("@tauri-apps/api/core");
    for (const specifier of imports) {
      expect(specifier).not.toMatch(/demo/);
      expect(specifier).not.toMatch(/fixtures/);
    }
  });

  it("the Tauri bootstrap configures the live provider", () => {
    const app = readFileSync(resolve(__dirname, "../../src/App.tsx"), "utf8");
    expect(app).toContain("configureDataProvider(liveProvider)");
  });

  it("wraps invoke results into typed envelopes", async () => {
    mocks.invoke.mockResolvedValueOnce([{ id: "s-0000" }]);
    const listed = await liveProvider.query<{ id: string }>("sessions_list");
    expect(mocks.invoke).toHaveBeenCalledWith("sessions_list", undefined);
    expect(listed).toEqual({ status: "ready", data: [{ id: "s-0000" }] });

    mocks.invoke.mockResolvedValueOnce([]);
    expect(await liveProvider.query("sessions_list")).toEqual({
      status: "empty",
    });

    mocks.invoke.mockResolvedValueOnce({ id: "s-0000" });
    expect(await liveProvider.queryOne<{ id: string }>("session")).toEqual({
      status: "ready",
      data: { id: "s-0000" },
    });

    mocks.invoke.mockRejectedValueOnce(new Error("unknown command"));
    const boom = await liveProvider.query("analytics_stats");
    expect(boom).toEqual({
      status: "failed",
      error: { command: "analytics_stats", message: "unknown command" },
    });
  });

  it("demo provider answers only its declared fixtures, as typed failures otherwise", async () => {
    const projects = await demoProvider.query("projects_list");
    expect(projects.status).toBe("ready");

    const scoped = await demoProvider.query("repos_list", {
      projectId: "p-tapestry",
    });
    expect(scoped.status).toBe("ready");

    const missing = await demoProvider.query("analytics_stats");
    expect(missing).toEqual({
      status: "failed",
      error: {
        command: "analytics_stats",
        message: "demo provider has no fixture for analytics_stats",
      },
    });

    const single = await demoProvider.queryOne("session");
    expect(single.status).toBe("failed");
  });
});
