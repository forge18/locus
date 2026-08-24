import { describe, expect, it } from "vitest";
import { parse } from "../../src/nav";

const fails = (locator: string) => {
  try {
    parse(locator);
  } catch (e) {
    return (e as Error).message;
  }
  throw new Error(`expected "${locator}" to be rejected`);
};

describe("nav/locator-rejects", () => {
  it("names the scheme when it is wrong", () => {
    expect(fails("https://tapestry/inbox")).toContain("scheme:");
    expect(fails("https://tapestry/inbox")).toContain("locus://");
  });

  it("names the project segment when it is empty", () => {
    expect(fails("locus:///inbox")).toContain("scope:");
  });

  it("names the view when it is not one of the 29 registered views", () => {
    const message = fails("locus://all/view/dashboard");
    expect(message).toContain("view:");
    expect(message).toContain('"dashboard"');
  });

  it("names the kind when it is not one of the six", () => {
    const message = fails("locus://tapestry/widget/x");
    expect(message).toContain("kind:");
    expect(message).toContain('"widget"');
  });

  it("names the id when an agent is missing its version", () => {
    const message = fails("locus://tapestry/agent/builder");
    expect(message).toContain("id:");
    expect(message).toContain("<name>@<version>");
  });

  it("names the sub-segment when a kind carries none", () => {
    const message = fails("locus://tapestry/task/t-004/step/2");
    expect(message).toContain("sub:");
    expect(message).toContain("none/<id>");
  });

  it("names the sub-segment when it is the wrong one", () => {
    const message = fails("locus://tapestry/session/8f21/turn/2");
    expect(message).toContain("sub:");
    expect(message).toContain("run/<id>");
  });

  it("rejects a bare project with nothing after it", () => {
    expect(fails("locus://tapestry")).toContain("view:");
  });
});
