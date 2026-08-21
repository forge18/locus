import { describe, expect, it } from 'vitest';
import { deserializeGraph, serializeGraph } from '../src/graph';
import { fixture } from '../src/fixture';

// The acceptance criterion says the assertion is on the SERIALIZED FORM, not on
// how it looks. A screenshot comparison would pass on a graph whose handles had
// silently collapsed.
describe('the round trip is byte-identical', () => {
  it('serialize -> deserialize -> serialize produces the same bytes', () => {
    const once = serializeGraph(fixture());
    expect(serializeGraph(deserializeGraph(once))).toBe(once);
  });

  it('DOES change when a handle changes — the check has teeth', () => {
    const g = fixture();
    g.edges.find((e) => e.id === 'e-5')!.sourceHandle = 'false';
    expect(serializeGraph(g)).not.toBe(serializeGraph(fixture()));
  });
});
