// The dashed rounded-rect around a Loop, positioned from its member nodes.
//
// Derived, never stored: a stored rectangle is a second source of truth for
// "which nodes are in this loop", and the two disagree the moment a node moves.
import type { WorkflowGraph } from './graph';

/** Node card size from the handoff: 188px wide, ~86px tall. */
export const NODE_W = 188;
export const NODE_H = 86;
export const PAD = 22;

export type LoopGroup = { id: string; label: string; x: number; y: number; width: number; height: number };

export function loopGroups(g: WorkflowGraph): LoopGroup[] {
  const groups: LoopGroup[] = [];
  for (const loop of g.nodes.filter((n) => n.kind === 'Loop')) {
    const members = g.nodes.filter((n) => n.loop === loop.id);
    if (members.length === 0) continue;
    const xs = members.map((n) => n.position.x);
    const ys = members.map((n) => n.position.y);
    const x = Math.min(...xs) - PAD;
    const y = Math.min(...ys) - PAD;
    groups.push({
      id: loop.id,
      label: String(loop.data.label ?? 'loop'),
      x, y,
      width:  (Math.max(...xs) + NODE_W + PAD) - x,
      height: (Math.max(...ys) + NODE_H + PAD) - y,
    });
  }
  return groups;
}
