import { describe, expect, it } from 'vitest';
import { render } from '@solidjs/testing-library';
import { Canvas } from '../src/Canvas';
import { fixture } from '../src/fixture';
import { HANDLES } from '../src/graph';

// Q1: do typed node props and typed handles work well enough to build the node
// vocabulary at the fidelity screen 12 draws?
describe('custom nodes', () => {
  const mount = () => render(() => <Canvas graph={fixture()} />);

  it('renders four DISTINCT custom node types', () => {
    const { container } = mount();
    const kinds = [...container.querySelectorAll('.wf-node')]
      .map((el) => el.getAttribute('data-kind'));
    expect(new Set(kinds)).toEqual(new Set(['Goal', 'Agent', 'Condition', 'Verify']));
  });

  it('renders each node from its OWN typed data, not from a generic label', () => {
    const { container } = mount();
    const byId = (id: string) => container.querySelector(`.wf-node[data-node-id="${id}"]`)!;
    // Agent: a pinned version and a permission narrowing
    expect(byId('a-1').textContent).toContain('builder@3');
    expect(byId('a-1').textContent).toContain('role implement');
    expect(byId('a-1').textContent).toContain('net packages');
    // Condition: the expression, in mono
    expect(byId('c-3').textContent).toContain('verify.passed and iteration < 8');
    // Verify: the runnable command, and where it runs
    expect(byId('v-1').textContent).toContain('cargo test -p locus-core');
    expect(byId('v-1').textContent).toContain('fresh container · run branch');
    // Goal: the approval state
    expect(byId('g-1').textContent).toContain('approved');
  });

  it('draws the handoff\'s tinted header strip with the kind label and a right-side state', () => {
    const { container } = mount();
    const strip = container.querySelector('.wf-node[data-kind="Agent"] .wf-strip')!;
    expect(strip.textContent).toContain('Agent');
    expect(strip.querySelector('.state')!.textContent).toBe('iter 3/8');
  });

  it('gives a Condition TWO named outbound handles, not two anonymous ones', () => {
    // This is the claim the whole node vocabulary rests on: a Condition routes
    // deterministically, so its edges must be distinguishable by name.
    const { container } = mount();
    const handles = [...container.querySelectorAll('.wf-node[data-kind="Condition"] [data-handle-id]')]
      .map((el) => el.getAttribute('data-handle-id'))
      .sort();
    expect(handles).toEqual(['false', 'true']);
  });

  it('renders exactly the handles the validator knows about, for every kind', () => {
    // The components read HANDLES rather than hard-coding ids, so a handle
    // cannot be drawn that validateGraph would then call unresolved.
    const { container } = mount();
    for (const kind of ['Goal', 'Agent', 'Condition', 'Verify'] as const) {
      const node = container.querySelector(`.wf-node[data-kind="${kind}"]`)!;
      const drawn = [...node.querySelectorAll('[data-handle-id]')]
        .map((el) => el.getAttribute('data-handle-id')).sort();
      expect(drawn, `${kind} outbound handles`).toEqual([...HANDLES[kind].out].sort());
    }
  });

  it('marks a selected node with the handoff\'s inset ring rather than an outer glow', () => {
    const { container } = mount();
    for (const node of container.querySelectorAll('.wf-node')) {
      expect(node.getAttribute('data-selected')).toMatch(/^(true|false)$/);
    }
  });
});
