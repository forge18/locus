import { describe, expect, it } from 'vitest';
import { render } from '@solidjs/testing-library';
import { Canvas, MARKERS, markerFor, toFlow } from '../src/Canvas';
import { fixture } from '../src/fixture';

// The handoff draws an SVG edge layer with four arrow markers — neutral,
// accent, --ok, --bad — and a dashed loop-back edge.
describe('the edge layer', () => {
  it('uses all four markers, assigned from the handle an edge leaves', () => {
    // Deriving the marker from the handle means the drawing cannot disagree
    // with the routing.
    expect(markerFor('true')).toBe('ok');
    expect(markerFor('false')).toBe('bad');
    expect(markerFor('start')).toBe('accent');
    expect(markerFor('out')).toBe('neutral');
    expect([...new Set(toFlow(fixture()).edges.map((e) => e.marker))].sort())
      .toEqual(Object.keys(MARKERS).sort());
  });

  it('draws the loop-back edge dashed, and only that one', () => {
    const edges = toFlow(fixture()).edges;
    expect(edges.filter((e) => e.dashed).map((e) => e.id)).toEqual(['e-6']);
    expect((edges.find((e) => e.id === 'e-6')!.style as Record<string, string>)['stroke-dasharray'])
      .toBe('4 3');
  });

  it('mounts an edge layer with marker definitions', () => {
    // NOTE: jsdom lays nothing out, so solid-flow renders the layer and its
    // markers but no edge paths — a handle with a 0x0 box has no position to
    // route from. Whether the paths are right is answered by the screenshot,
    // not here. Recorded in FINDINGS.md.
    const { container } = render(() => <Canvas graph={fixture()} />);
    expect(container.querySelector('.solid-flow__edges')).toBeTruthy();
    expect(container.querySelectorAll('marker').length).toBeGreaterThan(0);
  });
});
