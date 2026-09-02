import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { configureDataProvider } from "../../src/data/provider";
import { configureDemoProvider } from "../../src/data/demo/bootstrap";
import { ProvidersView } from "../../src/screens/workshop/ProvidersView";
import { ExtensionEditor } from "../../src/screens/workshop/ExtensionEditor";
import { WorkflowView } from "../../src/screens/workshop/WorkflowView";
import { CliToolsView } from "../../src/screens/workshop/CliToolsView";

describe("workshop and canvas editing controls", () => {
  it("loads and saves providers through the live data boundary", async () => {
    const calls: { command: string; args?: Record<string, unknown> }[] = [];
    const provider = {
      id: "00000000-0000-0000-0000-000000000001",
      identifier: "anthropic",
      keychainReference: "os-keychain://locus/anthropic",
      verificationAt: null,
      verificationModelCount: null,
      verificationStatus: null,
      verificationExpiresAt: null,
      authenticationMethod: "api-key" as const,
      baseUrl: null,
      models: [],
    };
    configureDataProvider({
      kind: "live",
      async query<T>(command: string, args?: Record<string, unknown>) {
        calls.push({ command, args });
        return { status: "ready", data: [provider] as T[] };
      },
      async queryOne<T>(command: string, args?: Record<string, unknown>) {
        calls.push({ command, args });
        return { status: "ready", data: provider as T };
      },
    });
    try {
      const view = render(() => <ProvidersView />);
      await waitFor(() =>
        expect(view.getByTestId("provider-save")).toBeTruthy(),
      );
      await fireEvent.click(view.getByTestId("provider-save"));
      expect(calls).toEqual([
        { command: "providers_list", args: undefined },
        {
          command: "provider_save",
          args: {
            request: {
              id: provider.id,
              identifier: provider.identifier,
              keychainReference: provider.keychainReference,
              authenticationMethod: provider.authenticationMethod,
              baseUrl: undefined,
            },
          },
        },
      ]);
      expect(view.getByTestId("provider-saved").textContent).toContain("store");
    } finally {
      configureDemoProvider();
    }
  });

  it("loads and saves an extension through the live data boundary", async () => {
    const calls: { command: string; args?: Record<string, unknown> }[] = [];
    const extension = {
      id: "00000000-0000-0000-0000-000000000011",
      extensionType: "skills",
      name: "verify-loop",
      version: 3,
      frontmatter: {
        budget_tokens: "12000",
        lazy: "true",
        tags: "verification",
      },
      body: "Run the verification command.",
      updatedAt: "2026-09-02T00:00:00Z",
    };
    configureDataProvider({
      kind: "live",
      async query<T>(command: string, args?: Record<string, unknown>) {
        calls.push({ command, args });
        return {
          status: "ready",
          data: (command === "harness_registry_list" ? [] : [extension]) as T[],
        };
      },
      async queryOne<T>(command: string, args?: Record<string, unknown>) {
        calls.push({ command, args });
        return { status: "ready", data: { ...extension, version: 4 } as T };
      },
    });
    try {
      const view = render(() => <ExtensionEditor type="skills" />);
      await waitFor(() =>
        expect(view.getByTestId("extension-name")).toBeTruthy(),
      );
      await fireEvent.input(view.getByTestId("extension-body-input"), {
        target: { value: "Updated verification command." },
      });
      await fireEvent.click(view.getByTestId("extension-save"));
      expect(calls).toContainEqual({
        command: "extensions_list",
        args: { extensionType: "skills" },
      });
      expect(calls).toContainEqual({
        command: "extension_save",
        args: {
          request: {
            id: extension.id,
            extensionType: "skills",
            name: extension.name,
            frontmatter: extension.frontmatter,
            body: "Updated verification command.",
          },
        },
      });
    } finally {
      configureDemoProvider();
    }
  });

  it("keeps extension edits local until an explicit save", async () => {
    const view = render(() => <ExtensionEditor type="skills" />);
    await fireEvent.click(view.getByTestId("extension-new"));
    expect(view.getByTestId("extension-total").textContent).toBe("4");
    await fireEvent.input(
      view.getByTestId("frontmatter-control-budget_tokens"),
      {
        target: { value: "9000" },
      },
    );
    expect(view.getByTestId("extension-editor").textContent).toContain(
      "unsaved changes",
    );
    await fireEvent.click(view.getByTestId("extension-save"));
    expect(view.getByTestId("extension-editor").textContent).toContain("saved");
  });

  it("loads and toggles CLI tools through the live data boundary", async () => {
    const calls: { command: string; args?: Record<string, unknown> }[] = [];
    const tool = {
      id: "00000000-0000-0000-0000-000000000101",
      name: "git",
      version: "2.49",
      category: "source-control",
      enabled: true,
      source: "builtin" as const,
      binarySha256: null,
      installCommand: "apt-get install git",
      verifyCommand: "git --version",
      documentationUrl: "https://git-scm.com/docs",
      lastRebuiltAt: null,
    };
    configureDataProvider({
      kind: "live",
      async query<T>(command: string, args?: Record<string, unknown>) {
        calls.push({ command, args });
        return { status: "ready", data: [tool] as T[] };
      },
      async queryOne<T>(command: string, args?: Record<string, unknown>) {
        calls.push({ command, args });
        return { status: "ready", data: { ...tool, enabled: false } as T };
      },
    });
    try {
      const view = render(() => <CliToolsView />);
      await waitFor(() =>
        expect(view.getByTestId("cli-tools").textContent).toContain("git"),
      );
      const toggle = view
        .getByTestId("cli-category-source-control")
        .querySelector("input");
      expect(toggle).toBeTruthy();
      await fireEvent.click(toggle!);
      expect(calls).toContainEqual({
        command: "cli_tool_enabled_set",
        args: { request: { id: tool.id, enabled: false } },
      });
    } finally {
      configureDemoProvider();
    }
  });

  it("authors a workflow through the live data boundary", async () => {
    const calls: { command: string; args?: Record<string, unknown> }[] = [];
    const detail = {
      id: "00000000-0000-0000-0000-000000000021",
      projectId: "project-1",
      name: "build-and-verify",
      version: 1,
      graph: {
        nodes: [
          { id: "start", kind: "agent", label: "builder", x: 0, y: 0 },
          {
            id: "verify",
            kind: "verify",
            label: "cargo test",
            command: "cargo test",
            x: 200,
            y: 0,
          },
        ],
        edges: [{ from: "start", to: "verify", label: null, dashed: false }],
      },
      spec: {
        version: 1,
        goal: "Ship it",
        guardrails: [],
        success_criteria: [],
      },
      verifyCommand: "cargo test",
    };
    configureDataProvider({
      kind: "live",
      async query<T>(command: string, args?: Record<string, unknown>) {
        calls.push({ command, args });
        const data =
          command === "projects_list"
            ? [{ id: "project-1", name: "locus" }]
            : command === "workflow_node_vocabulary"
              ? [
                  {
                    kind: "agent",
                    label: "Agent",
                    icon: "robot",
                    tone: "default",
                    required: false,
                  },
                  {
                    kind: "task",
                    label: "Task",
                    icon: "check-square",
                    tone: "default",
                    required: false,
                  },
                  {
                    kind: "verify",
                    label: "Verify",
                    icon: "flag-checkered",
                    tone: "default",
                    required: true,
                  },
                ]
              : command === "workflow_presets"
                ? [
                    {
                      name: "Ralph loop",
                      note: "pick · act · validate · commit · reset",
                    },
                  ]
                : command === "workflow_condition_operands"
                  ? ["verify.passed"]
                  : [];
        return { status: "ready", data: data as T[] };
      },
      async queryOne<T>(command: string, args?: Record<string, unknown>) {
        calls.push({ command, args });
        return { status: "ready", data: { ...detail, version: 2 } as T };
      },
    });
    try {
      const view = render(() => <WorkflowView projectId="project-1" />);
      await waitFor(() =>
        expect(view.getByTestId("workflow-new")).toBeTruthy(),
      );
      await fireEvent.click(view.getByTestId("workflow-new"));
      await waitFor(() =>
        expect(view.getByTestId("guardrail-add")).toBeTruthy(),
      );
      await fireEvent.click(view.getByTestId("guardrail-add"));
      await waitFor(() =>
        expect(
          calls.some((call) => call.command === "workflow_definition_save"),
        ).toBe(true),
      );
    } finally {
      configureDemoProvider();
    }
  });

  it("adds workflow nodes, clauses, and guardrail edits", async () => {
    const view = render(() => <WorkflowView />);
    const initialNodes = view
      .getByTestId("wf-canvas")
      .querySelectorAll(".wf-flow-node").length;
    await fireEvent.drop(view.getByTestId("wf-canvas"), {
      dataTransfer: {
        getData: (type: string) =>
          type === "application/x-locus-node" ? "task" : "",
      },
      clientX: 120,
      clientY: 80,
    });
    expect(
      view.getByTestId("wf-canvas").querySelectorAll(".wf-flow-node").length,
    ).toBe(initialNodes + 1);
    await fireEvent.click(view.getByTestId("wf-preset-Ralph-loop"));
    expect(
      view.getByTestId("wf-canvas").querySelectorAll(".wf-flow-node").length,
    ).toBeGreaterThan(initialNodes + 1);
    await fireEvent.click(view.getByTestId("clause-add"));
    expect(
      view.getByTestId("wf-inspector").querySelectorAll(".clause"),
    ).toHaveLength(3);
    await fireEvent.click(view.getByTestId("wf-node-n-build"));
    expect(view.getByTestId("wf-inspector-title").textContent).toContain(
      "agent · builder@4",
    );
    const toggle = view.getByTestId("guardrail-toggle-reflection_before_retry");
    const before = toggle.getAttribute("data-on");
    await fireEvent.click(toggle);
    await Promise.resolve();
    expect(
      view
        .getByTestId("guardrail-toggle-reflection_before_retry")
        .getAttribute("data-on"),
    ).not.toBe(before);
  });
});
