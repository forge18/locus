import { describe, expect, it } from "vitest";
import { VIEWS, format, parse, resolve } from "../../src/nav";
import type { View, ViewParams } from "../../src/nav";

/** One params set per view, covering the object form where the view has one. */
const CASES: Array<[View, ViewParams]> = [
    ["inbox", {}],
    ["status", {}],
    ["telemetry", {}],
    ["mail", {}],
    ["projects", {}],
    ["plan", {}],
    ["sessions", {}],
    ["workers", {}],
    ["workers", { project: "tapestry", botId: "keeper" }],
    ["qa", {}],
    ["autorun", {}],
    ["schedule", {}],
    ["runs", {}],
    ["short", {}],
    ["memory", {}],
    ["artifact", {}],
    ["wiki", {}],
    ["settings", {}],
    ["agents", {}],
    ["cli", {}],
    ["commands", {}],
    ["harnesses", {}],
    ["hooks", {}],
    ["linters", {}],
    ["styles", {}],
    ["providers", {}],
    ["rules", {}],
    ["skills", {}],
    ["canvas", {}],
    ["workflows", {}],
    ["sessions", { project: "loom-db", taskId: "t-004" }],
    ["sessions", { project: "tapestry", sessionId: "8f21" }],
    ["runs", { project: "weaver", sessionId: "5a71", runId: "9c02" }],
    ["artifact", { project: "weaver", artifactId: "a-1" }],
    ["wiki", { project: "texere", slug: "event-vocabulary" }],
    ["canvas", { project: "tapestry", workflowId: "wf-1" }],
    [
        "canvas",
        { project: "tapestry", workflowId: "wf-1", executionId: "ex-1" },
    ],
    [
        "agents",
        { agentName: "builder", agentVersion: "4", project: "tapestry" },
    ],
];

describe("nav/locator-roundtrip", () => {
    it("resolves back to what it formatted, for every case", () => {
        for (const [view, params] of CASES) {
            const locator = format(view, params);
            expect(resolve(locator), locator).toEqual({ view, params });
        }
    });

    it("covers every registered view", () => {
        expect(new Set(CASES.map(([v]) => v)).size).toBe(VIEWS.length);
    });

    it("formats back to the same string it parsed, for every case", () => {
        for (const [view, params] of CASES) {
            const locator = format(view, params);
            const back = resolve(locator);
            expect(format(back.view, back.params), locator).toBe(locator);
            expect(parse(locator).id).not.toBe("");
        }
    });

    it("is stable under a second round trip — normalization converges", () => {
        for (const [view, params] of CASES) {
            const once = format(view, params);
            const twice = format(resolve(once).view, resolve(once).params);
            expect(twice).toBe(once);
        }
    });
});
