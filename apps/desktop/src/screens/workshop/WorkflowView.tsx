import {
  createEffect,
  createMemo,
  createSignal,
  For,
  onMount,
  Show,
} from "solid-js";
import { Button } from "../../ui/Button";
import { Input } from "../../ui/Input";
import { dataProvider } from "../../data/provider";
import { InlineError } from "../../ui/InlineError";
import { PageProjectFilter } from "../PageProjectFilter";
import { FixtureNotice } from "../../ui/FixtureNotice";
import { Icon } from "../../ui/Icon";
import {
  BUDGET_NOTE,
  COMPILED_NOTE,
  OPERAND_NOTE,
  PRESET_NOTE,
  WAITING_NOTE,
  useCanvas,
  useExpression,
  useGuardrails,
  useOperands,
  usePalette,
  fetchWorkflowDefinitions,
  fetchWorkflowDefinition,
  fetchWorkflowOperands,
  fetchWorkflowPalette,
  fetchWorkflowPresets,
  saveWorkflowDefinition,
  readGuardrailBody,
  writeGuardrail,
  usePresets,
  type CanvasEdge,
  type CanvasNode,
  type PaletteNode,
  type PersistedWorkflowDefinition,
  type Preset,
  type WorkflowDefinitionSummary,
} from "../../data/workflow";
import type { Envelope } from "../../data/envelope";
import { WorkflowCanvas } from "../../workflow-canvas/WorkflowCanvas";

function record(value: unknown): Record<string, unknown> {
  return typeof value === "object" && value !== null
    ? (value as Record<string, unknown>)
    : {};
}

function text(value: unknown, fallback = ""): string {
  return typeof value === "string" ? value : fallback;
}

function canvasFromGraph(graph: Record<string, unknown>): {
  nodes: CanvasNode[];
  edges: CanvasEdge[];
} {
  const nodes = Array.isArray(graph.nodes)
    ? graph.nodes.flatMap((value) => {
        const node = record(value);
        const id = text(node.id);
        const kind = text(node.kind, "task").toLowerCase();
        if (!id) return [];
        const position = record(node.position);
        return [
          {
            id,
            kind,
            label: text(node.label, text(node.command, id)),
            state: text(node.state, "draft"),
            x: typeof node.x === "number" ? node.x : Number(position.x ?? 0),
            y: typeof node.y === "number" ? node.y : Number(position.y ?? 0),
            tone: kind === "condition" ? "condition" : "default",
          } satisfies CanvasNode,
        ];
      })
    : [];
  const edges = Array.isArray(graph.edges)
    ? graph.edges.flatMap((value) => {
        const edge = record(value);
        const from = text(edge.from, text(edge.source));
        const to = text(edge.to, text(edge.target));
        if (!from || !to) return [];
        return [
          {
            from,
            to,
            label: typeof edge.label === "string" ? edge.label : null,
            dashed: edge.dashed === true || edge.loopBack === true,
          } satisfies CanvasEdge,
        ];
      })
    : [];
  return { nodes, edges };
}

function graphFromCanvas(
  nodes: CanvasNode[],
  edges: CanvasEdge[],
): Record<string, unknown> {
  return { nodes, edges };
}

const EMPTY_LOOP = { x: 20, y: 20, width: 420, height: 220, label: "loop" };

function newWorkflowGraph(): Record<string, unknown> {
  return {
    nodes: [
      {
        id: "start",
        kind: "agent",
        role: "builder",
        label: "builder",
        state: "draft",
        x: 60,
        y: 100,
        tone: "default",
      },
      {
        id: "verify",
        kind: "verify",
        command: "cargo test",
        label: "cargo test",
        state: "draft",
        x: 280,
        y: 100,
        tone: "default",
      },
    ],
    edges: [{ from: "start", to: "verify", label: null, dashed: false }],
  };
}

function newWorkflowDetail(projectId: string): PersistedWorkflowDefinition {
  return {
    id: "",
    projectId,
    name: "new-workflow",
    version: 1,
    graph: newWorkflowGraph(),
    spec: {
      version: 1,
      goal: "Complete the workflow goal.",
      guardrails: [],
      success_criteria: [{ kind: "command", checker: "cargo test" }],
      verify_command: "cargo test",
    },
    verifyCommand: "cargo test",
  };
}

function guardrailsFromSpec(spec: Record<string, unknown>) {
  const values = Array.isArray(spec.guardrails) ? spec.guardrails : [];
  return values.flatMap((value, index) => {
    const guardrail = record(value);
    const name = text(guardrail.name, `guardrail-${index + 1}`);
    return [
      {
        key: name,
        label: name,
        kind: "value" as const,
        value: readGuardrailBody(guardrail),
      },
    ];
  });
}

function LiveWorkflowList(props: { projectId?: string }) {
  const [selectedProjectId, setSelectedProjectId] = createSignal(
    props.projectId,
  );
  const [definitions, setDefinitions] = createSignal<
    Envelope<WorkflowDefinitionSummary[]>
  >({ status: "loading" });
  const [selectedId, setSelectedId] = createSignal<string>();
  const [detail, setDetail] = createSignal<
    Envelope<PersistedWorkflowDefinition>
  >({ status: "empty" });
  const [palette, setPalette] = createSignal<Envelope<PaletteNode[]>>({
    status: "loading",
  });
  const [presets, setPresets] = createSignal<Envelope<Preset[]>>({
    status: "loading",
  });
  const [operands, setOperands] = createSignal<Envelope<string[]>>({
    status: "loading",
  });
  const [nodes, setNodes] = createSignal<CanvasNode[]>([]);
  const [edges, setEdges] = createSignal<CanvasEdge[]>([]);
  const [guardrails, setGuardrails] = createSignal<
    ReturnType<typeof guardrailsFromSpec>
  >([]);
  const [selectedNodeId, setSelectedNodeId] = createSignal<string>();
  const [workflowName, setWorkflowName] = createSignal("");
  const [goal, setGoal] = createSignal("");
  const [saving, setSaving] = createSignal(false);
  const [saveError, setSaveError] = createSignal<string>();
  const rows = () => {
    const state = definitions();
    return state.status === "ready" ? state.data : [];
  };
  const definitionError = () => {
    const state = definitions();
    return state.status === "failed" ? state.error.message : "";
  };
  const selectedDetail = () => {
    const state = detail();
    return state.status === "ready" ? state.data : undefined;
  };
  const paletteRows = () => {
    const state = palette();
    return state.status === "ready" ? state.data : [];
  };
  const presetRows = () => {
    const state = presets();
    return state.status === "ready" ? state.data : [];
  };
  const operandRows = () => {
    const state = operands();
    return state.status === "ready" ? state.data : [];
  };
  const selectedNode = createMemo(() =>
    nodes().find((node) => node.id === selectedNodeId()),
  );

  const applyDetail = (value: PersistedWorkflowDefinition) => {
    const canvas = canvasFromGraph(value.graph);
    setNodes(canvas.nodes);
    setEdges(canvas.edges);
    setSelectedNodeId(canvas.nodes[0]?.id);
    setWorkflowName(value.name);
    setGoal(text(value.spec.goal, "Complete the workflow goal."));
    setGuardrails(guardrailsFromSpec(value.spec));
    setSaveError(undefined);
  };

  createEffect(() => {
    const projectId = selectedProjectId();
    setSelectedId(undefined);
    setDetail({ status: "empty" });
    if (!projectId) {
      setDefinitions({
        status: "failed",
        error: {
          command: "workflow_definitions",
          message: "a project is required to read workflows",
        },
      });
      return;
    }
    setDefinitions({ status: "loading" });
    void fetchWorkflowDefinitions(projectId).then((result) => {
      setDefinitions(result);
      if (result.status === "ready" && result.data[0])
        setSelectedId(result.data[0].id);
    });
  });

  createEffect(() => {
    const projectId = selectedProjectId();
    const workflowId = selectedId();
    if (!projectId || !workflowId) return;
    setDetail({ status: "loading" });
    void fetchWorkflowDefinition(projectId, workflowId).then((result) => {
      setDetail(result);
      if (result.status === "ready") applyDetail(result.data);
    });
  });

  onMount(() => {
    void fetchWorkflowPalette().then(setPalette);
    void fetchWorkflowPresets().then(setPresets);
    void fetchWorkflowOperands().then(setOperands);
  });

  const persist = async (
    nextNodes = nodes(),
    nextEdges = edges(),
    nextGuardrails = guardrails(),
  ) => {
    const projectId = selectedProjectId();
    if (!projectId || !workflowName().trim()) return;
    const current = selectedDetail();
    const currentVersion =
      typeof current?.spec.version === "number"
        ? current.spec.version
        : (current?.version ?? 1);
    setSaving(true);
    setSaveError(undefined);
    const result = await saveWorkflowDefinition({
      projectId,
      name: workflowName().trim(),
      graph: graphFromCanvas(nextNodes, nextEdges),
      governance: {
        version: currentVersion,
        goal: goal().trim(),
        guardrails: nextGuardrails.map((guardrail) =>
          writeGuardrail(guardrail.key, guardrail.value),
        ),
        success_criteria: [
          { kind: "command", checker: current?.verifyCommand ?? "cargo test" },
        ],
      },
    });
    setSaving(false);
    if (result.status !== "ready") {
      setSaveError(
        result.status === "failed"
          ? result.error.message
          : "Workflow was not saved.",
      );
      return;
    }
    setDetail(result);
    applyDetail(result.data);
    setDefinitions({
      status: "ready",
      data: [
        ...rows().filter((row) => row.id !== result.data.id),
        {
          id: result.data.id,
          name: result.data.name,
          version: result.data.version,
        },
      ],
    });
    setSelectedId(result.data.id);
  };

  const createWorkflow = () => {
    const projectId = selectedProjectId();
    if (!projectId) return;
    const value = newWorkflowDetail(projectId);
    setDetail({ status: "ready", data: value });
    applyDetail(value);
    setSelectedId(undefined);
    const canvas = canvasFromGraph(value.graph);
    void persist(canvas.nodes, canvas.edges, []);
  };
  const addNode = (kind: string, position: { x: number; y: number }) => {
    const normalizedKind = kind.toLowerCase();
    if (
      normalizedKind === "verify" &&
      nodes().some((node) => node.kind === "verify")
    )
      return;
    const count = nodes().filter((node) => node.kind === normalizedKind).length;
    const newNode: CanvasNode = {
      id: `${normalizedKind}-${count + 1}`,
      kind: normalizedKind,
      label:
        paletteRows().find((node) => node.kind === normalizedKind)?.label ??
        kind,
      state: "draft",
      x: position.x,
      y: position.y,
      tone: normalizedKind === "condition" ? "condition" : "default",
    };
    const previous = nodes()[nodes().length - 1];
    const nextNodes = [...nodes(), newNode];
    const nextEdges = previous
      ? [
          ...edges(),
          { from: previous.id, to: newNode.id, label: null, dashed: false },
        ]
      : edges();
    setNodes(nextNodes);
    setEdges(nextEdges);
    setSelectedNodeId(newNode.id);
    void persist(nextNodes, nextEdges);
  };
  const moveNode = (id: string, position: { x: number; y: number }) => {
    const nextNodes = nodes().map((node) =>
      node.id === id ? { ...node, ...position } : node,
    );
    setNodes(nextNodes);
    void persist(nextNodes);
  };
  const connectNodes = (edge: CanvasEdge) => {
    if (
      edges().some(
        (candidate) => candidate.from === edge.from && candidate.to === edge.to,
      )
    )
      return;
    const nextEdges = [...edges(), edge];
    setEdges(nextEdges);
    void persist(nodes(), nextEdges);
  };
  const addGuardrail = () => {
    const nextGuardrails = [
      ...guardrails(),
      {
        key: `guardrail-${guardrails().length + 1}`,
        label: "new guardrail",
        kind: "value" as const,
        value: "",
      },
    ];
    setGuardrails(nextGuardrails);
    void persist(nodes(), edges(), nextGuardrails);
  };
  const updateGuardrail = (key: string, value: string) => {
    const nextGuardrails = guardrails().map((guardrail) =>
      guardrail.key === key ? { ...guardrail, value } : guardrail,
    );
    setGuardrails(nextGuardrails);
    void persist(nodes(), edges(), nextGuardrails);
  };

  return (
    <div class="wf wf-live" data-testid="workflow" data-live-state="ready">
      <header class="ws-head">
        <span class="ws-title">Workflow definitions</span>
        <PageProjectFilter
          value={selectedProjectId()}
          required
          onChange={setSelectedProjectId}
        />
        <Button
          variant="primary"
          data-testid="workflow-new"
          onClick={createWorkflow}
        >
          New workflow
        </Button>
        <span class="ws-note">
          Graph and Governance are persisted as immutable versions
        </span>
      </header>
      <Show when={definitions().status === "loading"}>
        <p data-testid="workflow-loading">Loading workflows…</p>
      </Show>
      <Show when={definitionError()}>
        <InlineError
          cause={definitionError()}
          next="Workflow definitions could not be loaded from the store."
        />
      </Show>
      <Show when={definitions().status === "empty"}>
        <p data-testid="workflow-empty">
          No workflow definitions are persisted for this project.
        </p>
      </Show>
      <div class="wf-live-layout">
        <aside class="wf-definition-list" data-testid="workflow-definitions">
          <For each={rows()}>
            {(definition) => (
              <button
                type="button"
                data-testid={`workflow-definition-${definition.id}`}
                aria-pressed={selectedId() === definition.id ? "true" : "false"}
                onClick={() => setSelectedId(definition.id)}
              >
                <strong>{definition.name}</strong>
                <span>v{definition.version}</span>
              </button>
            )}
          </For>
        </aside>
        <Show
          when={selectedDetail()}
          fallback={
            <main class="wf-live-empty">
              <p>Select a workflow or create a new one.</p>
            </main>
          }
        >
          <main class="wf-live-editor">
            <header class="wf-live-editor-head">
              <Input
                value={workflowName()}
                aria-label="Workflow name"
                data-testid="workflow-name"
                onInput={(event) => setWorkflowName(event.currentTarget.value)}
                onBlur={() => void persist()}
              />
              <span data-testid="wf-autosave">
                {saving() ? "saving…" : "saved to store"}
              </span>
            </header>
            <Show when={saveError()}>
              <InlineError
                cause={saveError()!}
                next="Correct the graph or governance and try again."
              />
            </Show>
            <aside class="wf-palette" data-testid="wf-palette">
              <span class="wf-section">Nodes</span>
              <For each={paletteRows()}>
                {(node) => (
                  <button
                    type="button"
                    class="wf-chip"
                    data-testid={`wf-chip-${node.kind}`}
                    draggable="true"
                    onDragStart={(event) =>
                      event.dataTransfer?.setData(
                        "application/x-locus-node",
                        node.kind,
                      )
                    }
                    /* Keyboard path: activating a chip places the node, the
                       same default position the presets use. */
                    onClick={() => addNode(node.kind, { x: 120, y: 260 })}
                  >
                    {node.label}
                  </button>
                )}
              </For>
              <span class="wf-section">Presets</span>
              <For each={presetRows()}>
                {(preset) => (
                  <button
                    type="button"
                    class="wf-preset"
                    data-testid={`wf-preset-${preset.name.replace(/\\W+/g, "-")}`}
                    onClick={() => addNode("task", { x: 120, y: 260 })}
                  >
                    {preset.name}
                    <small>{preset.note}</small>
                  </button>
                )}
              </For>
            </aside>
            <WorkflowCanvas
              nodes={nodes()}
              edges={edges()}
              loop={EMPTY_LOOP}
              events={[]}
              onSelect={setSelectedNodeId}
              onDrop={addNode}
              onConnect={connectNodes}
              onNodePositionChange={moveNode}
            />
            <aside class="wf-inspector" data-testid="wf-inspector">
              <span class="wf-section" data-testid="wf-inspector-title">
                {selectedNode()
                  ? `${selectedNode()!.kind} · ${selectedNode()!.label}`
                  : "Select a node"}
              </span>
              <Input
                value={goal()}
                aria-label="Workflow goal"
                data-testid="wf-goal"
                onInput={(event) => setGoal(event.currentTarget.value)}
                onBlur={() => void persist()}
              />
              <span class="wf-section">Condition operands</span>
              <div class="operands" data-testid="wf-operands">
                <For each={operandRows()}>
                  {(operand) => <span class="operand">{operand}</span>}
                </For>
              </div>
              <span class="wf-section" data-testid="wf-guardrails-title">
                Guardrails
              </span>
              <For each={guardrails()}>
                {(guardrail) => (
                  <div
                    class="guardrail-row"
                    data-testid={`guardrail-${guardrail.key}`}
                  >
                    <span>{guardrail.label}</span>
                    <Input
                      value={guardrail.value}
                      aria-label={guardrail.label}
                      onInput={(event) =>
                        updateGuardrail(
                          guardrail.key,
                          event.currentTarget.value,
                        )
                      }
                    />
                  </div>
                )}
              </For>
              <Button
                variant="ghost"
                data-testid="guardrail-add"
                onClick={addGuardrail}
              >
                + add guardrail
              </Button>
            </aside>
          </main>
        </Show>
      </div>
    </div>
  );
}

/**
 * The live provider owns the persisted workflow editor; the explicit demo provider
 * retains the fixture canvas used by isolated component tests.
 */
export function WorkflowView(props: { projectId?: string } = {}) {
  if (dataProvider().kind === "live")
    return <LiveWorkflowList projectId={props.projectId} />;
  const sourceCanvas = useCanvas();
  const palette = usePalette();
  const presets = usePresets();
  const operands = useOperands();
  const [nodes, setNodes] = createSignal<CanvasNode[]>([...sourceCanvas.nodes]);
  const [edges, setEdges] = createSignal<CanvasEdge[]>([...sourceCanvas.edges]);
  const [clauses, setClauses] = createSignal([...useExpression()]);
  const [guardrails, setGuardrails] = createSignal([...useGuardrails()]);
  const [selectedNodeId, setSelectedNodeId] = createSignal(
    sourceCanvas.nodes.find((node) => node.kind === "condition")?.id,
  );
  const [expandedPreset, setExpandedPreset] = createSignal<string>();
  const selectedNode = createMemo(() =>
    nodes().find((node) => node.id === selectedNodeId()),
  );
  const compiledExpression = createMemo(() =>
    clauses()
      .map((clause) => `${clause.operand} ${clause.operator} ${clause.value}`)
      .join(" and "),
  );
  const inspectorTitle = createMemo(() => {
    const node = selectedNode();
    if (!node) return "Select a node";
    return node.kind === "condition"
      ? `Condition · ${clauses()[0]?.operand ?? "new condition"}`
      : `${node.kind} · ${node.label}`;
  });

  const addNode = (kind: string, position: { x: number; y: number }) => {
    const normalizedKind = kind.toLowerCase();
    const count = nodes().filter(
      (node) => node.kind.toLowerCase() === normalizedKind,
    ).length;
    const label =
      palette.find((node) => node.kind.toLowerCase() === normalizedKind)
        ?.label ?? kind;
    const node: CanvasNode = {
      id: `n-${normalizedKind}-${count + 1}`,
      kind: normalizedKind,
      label,
      state: "draft",
      x: position.x,
      y: position.y,
      tone: normalizedKind === "condition" ? "condition" : "default",
    };
    setNodes((current) => [...current, node]);
    setSelectedNodeId(node.id);
  };
  const moveNode = (id: string, position: { x: number; y: number }) => {
    setNodes((current) =>
      current.map((node) =>
        node.id === id ? { ...node, x: position.x, y: position.y } : node,
      ),
    );
  };
  const connectNodes = (edge: CanvasEdge) => {
    setEdges((current) =>
      current.some(
        (candidate) => candidate.from === edge.from && candidate.to === edge.to,
      )
        ? current
        : [...current, edge],
    );
  };
  const expandPreset = (name: string) => {
    setExpandedPreset(name);
    if (name !== "Ralph loop") return;
    const ordinary: CanvasNode[] = [
      ["pick", "agent"],
      ["act", "task"],
      ["validate", "verify"],
      ["commit", "task"],
      ["reset", "loop"],
    ].map(([label, kind], index) => ({
      id: `n-ralph-${kind}-${index + 1}`,
      kind,
      label,
      state: "draft",
      x: 60 + index * 180,
      y: 360,
      tone: "default",
    }));
    setNodes((current) => [
      ...current,
      ...ordinary.filter(
        (candidate) => !current.some((node) => node.id === candidate.id),
      ),
    ]);
  };
  const addClause = () => {
    setClauses((current) => [
      ...current,
      {
        operand: operands[0] ?? "verify.passed",
        operator: "==",
        value: "true",
      },
    ]);
  };
  const updateGuardrail = (key: string, value: string) => {
    setGuardrails((current) =>
      current.map((guardrail) =>
        guardrail.key === key ? { ...guardrail, value } : guardrail,
      ),
    );
  };
  const stepGuardrail = (key: string, delta: number) => {
    const guardrail = guardrails().find((candidate) => candidate.key === key);
    if (!guardrail) return;
    const match = /^(\d+)(.*)$/.exec(guardrail.value);
    if (!match) return;
    updateGuardrail(key, `${Math.max(1, Number(match[1]) + delta)}${match[2]}`);
  };

  return (
    <div class="wf" data-testid="workflow">
      <aside class="wf-palette" data-testid="wf-palette">
        <span class="wf-section">Nodes</span>
        <For each={palette}>
          {(node) => (
            <button
              type="button"
              class={[
                "wf-chip",
                node.tone === "default" ? "" : `wf-chip-${node.tone}`,
              ]
                .filter(Boolean)
                .join(" ")}
              data-testid={`wf-chip-${node.kind}`}
              data-tone={node.tone}
              draggable="true"
              onDragStart={(event) => {
                event.dataTransfer?.setData(
                  "application/x-locus-node",
                  node.kind,
                );
                event.dataTransfer?.setData("text/plain", node.kind);
              }}
              onClick={() => addNode(node.kind, { x: 120, y: 260 })}
            >
              <Icon
                name="dots-six-vertical"
                size={10}
                style={{ color: "var(--text-muted)" }}
              />
              <Icon name={node.icon} size={11} />
              {node.label}
              <Show when={node.required}>
                <span
                  class="wf-chip-req"
                  data-testid={`wf-chip-req-${node.kind}`}
                >
                  req
                </span>
              </Show>
            </button>
          )}
        </For>

        <span class="wf-section" data-testid="wf-presets-title">
          Presets
        </span>
        <For each={presets}>
          {(preset) => (
            <button
              type="button"
              class="wf-preset"
              data-testid={`wf-preset-${preset.name.replace(/\W+/g, "-")}`}
              aria-expanded={expandedPreset() === preset.name}
              onClick={() => expandPreset(preset.name)}
            >
              {preset.name}
              <div class="wf-preset-note">{preset.note}</div>
            </button>
          )}
        </For>
        <Show when={expandedPreset()}>
          <div
            class="wf-preset-expanded"
            data-testid="wf-preset-expanded"
            data-preset={expandedPreset()}
          >
            <span>ordinary nodes</span>
            <span>pick</span>
            <span>act</span>
            <span>validate</span>
            <span>commit</span>
            <span>reset</span>
          </div>
        </Show>
        <span class="wf-preset-note" data-testid="wf-preset-note">
          {PRESET_NOTE}
        </span>
      </aside>

      <WorkflowCanvas
        nodes={nodes()}
        edges={edges()}
        loop={sourceCanvas.loop}
        events={sourceCanvas.events}
        onSelect={setSelectedNodeId}
        onDrop={addNode}
        onConnect={connectNodes}
        onNodePositionChange={moveNode}
      />

      <aside class="wf-inspector" data-testid="wf-inspector">
        <FixtureNotice
          surface="Workflow canvas"
          command='invoke("workflow_graph")'
        />
        <div class="wf-inspector-body">
          <span class="wf-section" data-testid="wf-inspector-title">
            {inspectorTitle()}
          </span>

          <For each={clauses()}>
            {(clause, i) => (
              <>
                <Show when={i() > 0}>
                  <span class="clause-joiner" data-testid="clause-joiner">
                    and
                  </span>
                </Show>
                <div class="clause" data-testid={`clause-${i()}`}>
                  <span class="clause-field">{clause.operand}</span>
                  <span class="clause-field">{clause.operator}</span>
                  <span class="clause-field">{clause.value}</span>
                </div>
              </>
            )}
          </For>
          <button
            type="button"
            class="clause-add"
            data-testid="clause-add"
            onClick={addClause}
          >
            + add clause
          </button>

          <div class="compiled" data-testid="wf-compiled">
            <div class="compiled-expr" data-testid="wf-compiled-expr">
              {compiledExpression()}
            </div>
            <div class="compiled-note" data-testid="wf-compiled-note">
              {COMPILED_NOTE}
            </div>
          </div>

          <span class="wf-section" data-testid="wf-operands-title">
            Operands — every one is a column
          </span>
          <div class="operands" data-testid="wf-operands">
            <For each={operands}>
              {(operand) => (
                <span
                  class="operand"
                  data-testid={`operand-${operand.replace(/\W+/g, "-")}`}
                >
                  {operand}
                </span>
              )}
            </For>
          </div>
          <span class="wf-preset-note" data-testid="wf-operand-note">
            {OPERAND_NOTE}
          </span>

          <span class="wf-section" data-testid="wf-guardrails-title">
            Guardrails
          </span>
          <For each={guardrails()}>
            {(guardrail) => (
              <div
                class="guardrail-row"
                data-testid={`guardrail-${guardrail.key}`}
              >
                {guardrail.label}
                <Show
                  when={guardrail.kind === "stepper"}
                  fallback={
                    <Show
                      when={guardrail.kind === "toggle"}
                      fallback={
                        <span
                          class="guardrail-value"
                          data-testid={`guardrail-value-${guardrail.key}`}
                        >
                          {guardrail.value}
                        </span>
                      }
                    >
                      <button
                        type="button"
                        class="toggle"
                        role="switch"
                        aria-checked={guardrail.value === "on"}
                        aria-label={guardrail.label}
                        data-testid={`guardrail-toggle-${guardrail.key}`}
                        data-on={guardrail.value === "on" ? "true" : "false"}
                        onClick={() =>
                          updateGuardrail(
                            guardrail.key,
                            guardrail.value === "on" ? "off" : "on",
                          )
                        }
                      />
                    </Show>
                  }
                >
                  <span
                    class="stepper"
                    data-testid={`guardrail-stepper-${guardrail.key}`}
                  >
                    <button
                      type="button"
                      aria-label={`Decrease ${guardrail.label}`}
                      onClick={() => stepGuardrail(guardrail.key, -1)}
                    >
                      −
                    </button>
                    <span
                      class="guardrail-value"
                      data-testid={`guardrail-value-${guardrail.key}`}
                    >
                      {guardrail.value}
                    </span>
                    <button
                      type="button"
                      aria-label={`Increase ${guardrail.label}`}
                      onClick={() => stepGuardrail(guardrail.key, 1)}
                    >
                      +
                    </button>
                  </span>
                </Show>
              </div>
            )}
          </For>
          <span class="wf-preset-note" data-testid="wf-budget-note">
            {BUDGET_NOTE}
          </span>
          <span class="wf-preset-note" data-testid="wf-waiting-note">
            {WAITING_NOTE}
          </span>
        </div>

        <footer class="wf-inspector-foot" data-testid="wf-inspector-foot">
          <span data-testid="wf-autosave">saved 2s ago</span>
        </footer>
      </aside>
    </div>
  );
}

/** Default export so the view can be code-split at the route boundary. */
export default WorkflowView;
