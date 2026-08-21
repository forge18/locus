// The four node kinds Q1 exercises, chosen because they span the shapes that
// differ: an approval state (Goal), a pinned version plus a permission
// narrowing (Agent), MULTIPLE NAMED OUTBOUND HANDLES (Condition), and a
// pass/fail state (Verify).
//
// Typed props and typed handles are the claim under test. `NodeProps<T>` gives
// the first; `HANDLES` in graph.ts gives the second, and the components render
// their handles FROM that table rather than hard-coding ids — so a handle can
// only exist here if the validator also knows about it.

import { Handle, Position, type NodeProps } from '@dschz/solid-flow';
import { For, Show } from 'solid-js';
import { HANDLES, type NodeKind } from '../graph';

type Shell = {
  kind: NodeKind;
  state?: string;
  selected?: boolean;
  children?: unknown;
};

function NodeShell(props: Shell & { id: string; children?: any }) {
  const spec = () => HANDLES[props.kind];
  // Handles are laid out along the edge they belong to, evenly spaced, so two
  // outbound handles are visually distinguishable as well as distinct in data.
  const at = (i: number, n: number) => `${((i + 1) / (n + 1)) * 100}%`;
  return (
    <div class="wf-node" data-kind={props.kind} data-node-id={props.id}
         data-selected={String(Boolean(props.selected))}>
      <For each={spec().in}>{(h, i) => (
        <Handle type="target" position={Position.Left} id={h}
                style={{ top: at(i(), spec().in.length) }} />
      )}</For>
      <div class="wf-strip">
        <span>{props.kind}</span>
        <Show when={props.state}><span class="state">{props.state}</span></Show>
      </div>
      <div class="wf-body">{props.children}</div>
      <For each={spec().out}>{(h, i) => (
        <Handle type="source" position={Position.Right} id={h}
                data-handle-id={h}
                style={{ top: at(i(), spec().out.length) }} />
      )}</For>
    </div>
  );
}

export type GoalData      = { label: string; approved: boolean };
export type AgentData     = { label: string; agent: string; version: string; role: string;
                              tools: string[]; network: 'none' | 'model' | 'packages' | 'open';
                              iteration?: string };
export type ConditionData = { label: string; expression: string };
export type VerifyData    = { label: string; command: string; result?: 'passed' | 'failed' };

export const GoalNode = (props: NodeProps<{ type: 'Goal'; data: GoalData }>) => (
  <NodeShell id={props.id} kind="Goal" selected={props.selected}
             state={props.data.approved ? 'approved' : 'awaiting approval'}>
    <div class="wf-title">{props.data.label}</div>
    <div class="wf-mono">also the termination condition</div>
  </NodeShell>
);

export const AgentNode = (props: NodeProps<{ type: 'Agent'; data: AgentData }>) => (
  <NodeShell id={props.id} kind="Agent" selected={props.selected} state={props.data.iteration}>
    <div class="wf-title">{props.data.label}</div>
    <div class="wf-mono">{props.data.agent}@{props.data.version}</div>
    <div class="wf-chips">
      <span class="wf-chip">role {props.data.role}</span>
      <span class="wf-chip">net {props.data.network}</span>
      <span class="wf-chip">tools {props.data.tools.length}</span>
    </div>
  </NodeShell>
);

export const ConditionNode = (props: NodeProps<{ type: 'Condition'; data: ConditionData }>) => (
  <NodeShell id={props.id} kind="Condition" selected={props.selected} state="deterministic">
    <div class="wf-title">{props.data.label}</div>
    <div class="wf-mono">{props.data.expression}</div>
  </NodeShell>
);

export const VerifyNode = (props: NodeProps<{ type: 'Verify'; data: VerifyData }>) => (
  <NodeShell id={props.id} kind="Verify" selected={props.selected} state={props.data.result ?? 'req'}>
    <div class="wf-title">{props.data.label}</div>
    <div class="wf-mono">{props.data.command}</div>
    <div class="wf-chips"><span class="wf-chip">fresh container · run branch</span></div>
  </NodeShell>
);

export const nodeTypes = {
  Goal: GoalNode,
  Agent: AgentNode,
  Condition: ConditionNode,
  Verify: VerifyNode,
};
