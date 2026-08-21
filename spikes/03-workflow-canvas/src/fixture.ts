// One workflow, drawn once, used by every test and by the screenshot.
//
// It is the Ralph loop PLAN.md describes — pick, act, validate, commit, reset —
// expressed in ordinary nodes rather than as a preset, because the preset
// "expands into the ordinary nodes so it can be edited rather than configured".
import type { WorkflowGraph } from './graph';

export const fixture = (): WorkflowGraph => ({
  version: 1,
  nodes: [
    { id: 'g-1', kind: 'Goal', position: { x: 40, y: 180 },
      data: { label: 'Retry policy on PaymentClient', approved: true } },
    { id: 'l-1', kind: 'Loop', position: { x: 260, y: 180 },
      data: { label: 'build loop', max_iterations: 8, reset: 'fresh run, same session' } },
    { id: 'a-1', kind: 'Agent', position: { x: 460, y: 96 }, loop: 'l-1',
      data: { label: 'builder', agent: 'builder', version: '3', role: 'implement',
              tools: ['edit', 'shell', 'lsp'], network: 'packages', iteration: 'iter 3/8' } },
    { id: 'v-1', kind: 'Verify', position: { x: 700, y: 96 }, loop: 'l-1',
      data: { label: 'test suite', command: 'cargo test -p locus-core' } },
    { id: 'c-3', kind: 'Condition', position: { x: 940, y: 96 }, loop: 'l-1',
      data: { label: 'passed?', expression: 'verify.passed and iteration < 8' } },
    { id: 'a-2', kind: 'Agent', position: { x: 1180, y: 220 },
      data: { label: 'reviewer', agent: 'reviewer', version: '2', role: 'review',
              tools: ['read'], network: 'none' } },
  ],
  edges: [
    { id: 'e-1', source: 'g-1', sourceHandle: 'start', target: 'l-1', targetHandle: 'in' },
    { id: 'e-2', source: 'l-1', sourceHandle: 'body',  target: 'a-1', targetHandle: 'in' },
    { id: 'e-3', source: 'a-1', sourceHandle: 'out',   target: 'v-1', targetHandle: 'in' },
    { id: 'e-4', source: 'v-1', sourceHandle: 'passed', target: 'c-3', targetHandle: 'in' },
    // The two branches of a Condition. Not interchangeable, which is why the
    // handle id has to survive serialization.
    { id: 'e-5', source: 'c-3', sourceHandle: 'true',  target: 'a-2', targetHandle: 'in' },
    { id: 'e-6', source: 'c-3', sourceHandle: 'false', target: 'a-1', targetHandle: 'in', loopBack: true },
    { id: 'e-7', source: 'l-1', sourceHandle: 'exit',  target: 'a-2', targetHandle: 'in' },
  ],
});
