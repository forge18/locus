import { describe, expect, it } from 'vitest';
import { validateGraph, type WorkflowGraph } from '../src/graph';
import { fixture } from '../src/fixture';

// PLAN.md makes loop termination a GRAPH-VALIDATION requirement, refused when
// the graph is saved rather than when it runs.
describe('a loop with no termination condition is refused at save time', () => {
  const stripTermination = (): WorkflowGraph => {
    const g = fixture();
    delete (g.nodes.find((n) => n.id === 'l-1')!.data as Record<string, unknown>).max_iterations;
    g.edges = g.edges.filter((e) => !(e.source === 'l-1' && e.sourceHandle === 'exit'));
    g.edges = g.edges.filter((e) => e.id !== 'e-5');
    return g;
  };

  it('the fixture as drawn is valid', () => {
    expect(validateGraph(fixture())).toEqual([]);
  });

  it('is rejected, and the message NAMES the offending node and what is missing', () => {
    const error = validateGraph(stripTermination()).find((e) => e.rule === 'loop-no-termination');
    expect(error).toBeDefined();
    expect(error!.node).toBe('l-1');
    expect(error!.message).toContain('build loop');
    expect(error!.message).toMatch(/exit/);
    expect(error!.message).toMatch(/max_iterations/);
  });

  it('does NOT count the dashed loop-back edge as a way out', () => {
    // Counting it would make every non-terminating loop look terminated.
    const g = stripTermination();
    expect(g.edges.some((e) => e.loopBack)).toBe(true);
    expect(validateGraph(g).some((e) => e.rule === 'loop-no-termination')).toBe(true);
  });
});
