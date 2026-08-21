import { describe, expect, it } from 'vitest';
import { validateGraph } from '../src/graph';
import { fixture } from '../src/fixture';

describe('cycles', () => {
  it('a DECLARED loop-back is not an error — a workflow is a loop toward a goal', () => {
    expect(validateGraph(fixture()).filter((e) => e.rule === 'cycle')).toEqual([]);
  });

  it('an UNDECLARED back edge is rejected, naming the node and tracing the path', () => {
    const g = fixture();
    g.edges.find((e) => e.id === 'e-6')!.loopBack = undefined;
    const error = validateGraph(g).find((e) => e.rule === 'cycle')!;
    expect(error.node).toBe('a-1');
    expect(error.message).toMatch(/a-1 -> v-1 -> c-3 -> a-1/);
  });
});

describe('a missing Verify', () => {
  it('is rejected when there is no Verify node at all', () => {
    const g = fixture();
    g.nodes = g.nodes.filter((n) => n.kind !== 'Verify');
    g.edges = g.edges.filter((e) => e.source !== 'v-1' && e.target !== 'v-1');
    expect(validateGraph(g).some((e) => e.rule === 'missing-verify')).toBe(true);
  });

  it('is rejected when a Verify node carries no command', () => {
    // A loop iterating against a weak check converges confidently on the wrong
    // thing, which is why `verify` is NOT NULL.
    const g = fixture();
    g.nodes.find((n) => n.id === 'v-1')!.data.command = '   ';
    const error = validateGraph(g).find((e) => e.rule === 'missing-verify')!;
    expect(error.node).toBe('v-1');
  });
});
