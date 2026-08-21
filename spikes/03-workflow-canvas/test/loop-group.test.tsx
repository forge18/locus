import { describe, expect, it } from 'vitest';
import { render } from '@solidjs/testing-library';
import { Canvas } from '../src/Canvas';
import { loopGroups, NODE_H, NODE_W, PAD } from '../src/loopGroup';
import { fixture } from '../src/fixture';

// A dashed rounded rect around the loop, POSITIONED FROM ITS MEMBER NODES.
// Derived rather than stored: a stored rectangle is a second source of truth
// for which nodes are in the loop, and the two disagree the moment one moves.
describe('the loop grouping', () => {
  it('encloses every member and excludes every non-member', () => {
    const [g] = loopGroups(fixture());
    const inside = (x: number, y: number) =>
      x >= g.x && y >= g.y && x + NODE_W <= g.x + g.width && y + NODE_H <= g.y + g.height;
    for (const n of fixture().nodes) {
      expect(inside(n.position.x, n.position.y), `${n.id} (loop=${n.loop})`).toBe(n.loop === 'l-1');
    }
    expect(g.x).toBe(Math.min(...fixture().nodes.filter((n) => n.loop).map((n) => n.position.x)) - PAD);
  });

  it('MOVES when a member moves — the derivation is live', () => {
    const before = loopGroups(fixture())[0];
    const g = fixture();
    g.nodes.find((n) => n.id === 'c-3')!.position.x += 300;
    expect(loopGroups(g)[0].width).toBe(before.width + 300);
  });

  // NOTE: the group goes through solid-flow's ViewportPortal so it shares the
  // pan/zoom transform. The portal only mounts once the renderer has a DOM node,
  // which jsdom never gives it — so this asserts against a direct render, and
  // the screenshot is what shows it landing inside the viewport.
  it('renders as a dashed rounded rect labelled with the loop', () => {
    const { container } = render(() => <Canvas graph={fixture()} portalGroups={false} />);
    const rect = container.querySelector('.wf-loop-group') as HTMLElement;
    expect(rect.getAttribute('data-loop')).toBe('l-1');
    expect(rect.querySelector('.label')!.textContent).toBe('build loop');
    expect(rect.style.width).toBe(`${loopGroups(fixture())[0].width}px`);
  });
});
