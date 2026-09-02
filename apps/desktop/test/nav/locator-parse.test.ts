import { describe, expect, it } from "vitest";
import { KINDS, parse } from "../../src/nav";

describe("nav/locator-parse", () => {
  it("parses a session", () => {
    expect(parse("locus://tapestry/session/8f21")).toEqual({
      project: "tapestry",
      kind: "session",
      id: "8f21",
      subId: null,
    });
  });

  it("parses a session with its run", () => {
    expect(parse("locus://tapestry/session/8f21/run/3c04")).toEqual({
      project: "tapestry",
      kind: "session",
      id: "8f21",
      subId: "3c04",
    });
  });

  it("parses a task, an artifact and a page", () => {
    expect(parse("locus://loom-db/task/t-004").kind).toBe("task");
    expect(parse("locus://weaver/artifact/a-1").id).toBe("a-1");
    expect(parse("locus://texere/page/notification-sinks").id).toBe(
      "notification-sinks",
    );
  });

  it("parses a workflow, with and without an execution", () => {
    expect(parse("locus://tapestry/workflow/wf-1").subId).toBe(null);
    expect(parse("locus://tapestry/workflow/wf-1/execution/ex-1").subId).toBe(
      "ex-1",
    );
  });

  it("parses an agent as name@version", () => {
    expect(parse("locus://tapestry/agent/builder@4")).toEqual({
      project: "tapestry",
      kind: "agent",
      id: "builder@4",
      subId: null,
    });
  });

  it("parses the Workers view and a worker detail locator", () => {
    expect(parse("locus://all/view/workers")).toEqual({
      project: "all",
      kind: null,
      id: "workers",
      subId: null,
    });
    expect(parse("locus://tapestry/workers/keeper")).toEqual({
      project: "tapestry",
      kind: "bot",
      id: "keeper",
      subId: null,
    });
  });

  it("covers all seven kinds", () => {
    expect([...KINDS]).toEqual([
      "session",
      "task",
      "artifact",
      "page",
      "workflow",
      "agent",
      "bot",
    ]);
  });

  it("parses the canonical view form, which addresses a screen rather than an object", () => {
    expect(parse("locus://all/view/inbox")).toEqual({
      project: "all",
      kind: null,
      id: "inbox",
      subId: null,
    });
  });

  it("keeps the project as the first segment for project-scoped views", () => {
    for (const p of ["tapestry", "loom-db", "weaver", "texere"]) {
      expect(parse(`locus://${p}/view/plan`).project).toBe(p);
    }
  });
});
