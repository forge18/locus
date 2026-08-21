// The workflow graph — the minimum needed to answer Spike 3's questions.
//
// SCOPE NOTE. This is spike code, not `.specs/workflow-canvas`. It exists to
// make three claims checkable: that a graph round-trips unchanged, that handle
// identities survive it, and that a loop with no termination can be refused at
// save time. The M4 feature owns the rest — the full validation set PLAN.md
// lists (unresolved handles, unreachable goal, role contamination), the `spec`
// JSONB the supervisor reads, and anything about how this is stored.

export type NodeKind = 'Goal' | 'Agent' | 'Task' | 'Loop' | 'Condition' | 'Gate' | 'Verify';

/** Named handles, because a `Condition`'s two edges are not interchangeable. */
export const HANDLES: Record<NodeKind, { in: string[]; out: string[] }> = {
  Goal:      { in: ['approved'], out: ['start'] },
  Agent:     { in: ['in'],       out: ['out'] },
  Task:      { in: ['in'],       out: ['out'] },
  Loop:      { in: ['in'],       out: ['body', 'exit'] },
  Condition: { in: ['in'],       out: ['true', 'false'] },
  Gate:      { in: ['in'],       out: ['pass', 'reject'] },
  Verify:    { in: ['in'],       out: ['passed', 'failed'] },
};

export type WorkflowNode = {
  id: string;
  kind: NodeKind;
  position: { x: number; y: number };
  data: Record<string, unknown>;
  /** Set on nodes inside a Loop's body. The dashed grouping is derived from it. */
  loop?: string;
};

export type WorkflowEdge = {
  id: string;
  source: string;
  sourceHandle: string;
  target: string;
  targetHandle: string;
  /** A loop-back edge is drawn dashed and is exempt from the cycle check. */
  loopBack?: boolean;
};

export type WorkflowGraph = { version: 1; nodes: WorkflowNode[]; edges: WorkflowEdge[] };

// --- serialization -----------------------------------------------------------
// Both directions write the same fields in the same order, which is what makes
// deserialize -> serialize a fixed point. That is all Q2 needs; a canonical
// form that is also insensitive to node order and key order is M4's problem.

export const serializeGraph = (g: WorkflowGraph): string =>
  JSON.stringify({
    version: g.version,
    nodes: g.nodes.map((n) => ({
      id: n.id, kind: n.kind,
      position: { x: n.position.x, y: n.position.y },
      data: n.data,
      ...(n.loop !== undefined ? { loop: n.loop } : {}),
    })),
    edges: g.edges.map((e) => ({
      id: e.id,
      source: e.source, sourceHandle: e.sourceHandle,
      target: e.target, targetHandle: e.targetHandle,
      ...(e.loopBack !== undefined ? { loopBack: e.loopBack } : {}),
    })),
  });

export const deserializeGraph = (json: string): WorkflowGraph => {
  const raw = JSON.parse(json);
  if (raw.version !== 1) throw new Error(`unsupported graph version: ${raw.version}`);
  return {
    version: 1,
    nodes: raw.nodes.map((n: WorkflowNode) => ({
      id: n.id, kind: n.kind,
      position: { x: n.position.x, y: n.position.y },
      data: { ...n.data },
      ...(n.loop !== undefined ? { loop: n.loop } : {}),
    })),
    edges: raw.edges.map((e: WorkflowEdge) => ({
      id: e.id, source: e.source, sourceHandle: e.sourceHandle,
      target: e.target, targetHandle: e.targetHandle,
      ...(e.loopBack !== undefined ? { loopBack: e.loopBack } : {}),
    })),
  };
};

// --- validation --------------------------------------------------------------
// Three rules, because three are what the spike's tasks name: a loop with no
// termination, an undeclared cycle, and a missing verify. Each must name the
// offending node — a validator that says "invalid graph" on a canvas of forty
// nodes has not helped anyone.

export type ValidationError = { rule: string; node?: string; message: string };

export function validateGraph(g: WorkflowGraph): ValidationError[] {
  const errors: ValidationError[] = [];

  // -- a Verify is required, and it needs a command --------------------------
  const verifies = g.nodes.filter((n) => n.kind === 'Verify');
  if (verifies.length === 0) {
    errors.push({ rule: 'missing-verify',
      message: 'the workflow has no Verify node; a runnable success criterion is required' });
  }
  for (const v of verifies) {
    if (!String(v.data.command ?? '').trim()) {
      errors.push({ rule: 'missing-verify', node: v.id,
        message: `Verify node '${v.id}' has no command` });
    }
  }

  // -- loop termination ------------------------------------------------------
  // A Loop terminates when something can leave it: its `exit` handle is wired,
  // a node in its body routes out, or it sets max_iterations.
  for (const loop of g.nodes.filter((n) => n.kind === 'Loop')) {
    const members = new Set(g.nodes.filter((n) => n.loop === loop.id).map((n) => n.id));
    const exitWired = g.edges.some((e) => e.source === loop.id && e.sourceHandle === 'exit');
    const routesOut = g.edges.some((e) =>
      members.has(e.source) && !members.has(e.target) && e.target !== loop.id && !e.loopBack);
    if (!exitWired && !routesOut && !loop.data.max_iterations) {
      errors.push({ rule: 'loop-no-termination', node: loop.id,
        message: `Loop '${String(loop.data.label ?? loop.id)}' (${loop.id}) has no termination condition: its 'exit' handle is unwired, no node in its body routes out of it, and it sets no max_iterations` });
    }
  }

  // -- cycles, excluding declared loop-backs ---------------------------------
  // A workflow IS a loop toward a goal, so a cycle is not per se an error — an
  // UNDECLARED one is. It is the case that hangs a run.
  const adjacency = new Map(g.nodes.map((n) => [n.id, [] as string[]]));
  for (const e of g.edges) if (!e.loopBack) adjacency.get(e.source)?.push(e.target);
  const state = new Map(g.nodes.map((n) => [n.id, 'white']));
  const path: string[] = [];
  const visit = (id: string): boolean => {
    state.set(id, 'grey');
    path.push(id);
    for (const next of adjacency.get(id) ?? []) {
      if (state.get(next) === 'grey') {
        errors.push({ rule: 'cycle', node: next,
          message: `undeclared cycle through '${next}': ${path.slice(path.indexOf(next)).concat(next).join(' -> ')}. Mark the back edge loopBack, or route it through a Loop` });
        return true;
      }
      if (state.get(next) === 'white' && visit(next)) return true;
    }
    path.pop();
    state.set(id, 'black');
    return false;
  };
  for (const n of g.nodes) if (state.get(n.id) === 'white') visit(n.id);

  return errors;
}
