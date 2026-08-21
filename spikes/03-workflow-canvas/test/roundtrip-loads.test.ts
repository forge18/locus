import { describe, expect, it } from 'vitest';
import { deserializeGraph, serializeGraph } from '../src/graph';
import { fixture } from '../src/fixture';

describe('canvas -> JSONB -> canvas', () => {
  it('reloads nodes with their positions, and edges with BOTH handle identities', () => {
    const back = deserializeGraph(serializeGraph(fixture()));

    for (const original of fixture().nodes) {
      const loaded = back.nodes.find((n) => n.id === original.id);
      expect(loaded, `node ${original.id}`).toBeDefined();
      expect(loaded!.position).toEqual(original.position);
      expect(loaded!.loop).toBe(original.loop);
    }

    // The part that matters: a Condition's `true` and `false` edges are not
    // interchangeable, so an edge reloading through the wrong handle is a
    // silent routing change.
    for (const original of fixture().edges) {
      const loaded = back.edges.find((e) => e.id === original.id)!;
      expect(loaded.sourceHandle).toBe(original.sourceHandle);
      expect(loaded.targetHandle).toBe(original.targetHandle);
    }
    expect(back.edges.filter((e) => e.source === 'c-3').map((e) => e.sourceHandle).sort())
      .toEqual(['false', 'true']);
    expect(back.edges.find((e) => e.id === 'e-6')!.loopBack).toBe(true);
  });
});
