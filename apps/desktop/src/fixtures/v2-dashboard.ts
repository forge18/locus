// schema: telemetry aggregates + agents.sessions + inbox items
// replaced by: invoke("dashboard_fixture") + emit("inbox_changed")

export const V2_INBOX_ITEMS = [
  {
    kind: 'gate',
    title: 'Gate — approve plan before implementation',
    age: '4m',
    detail: 'texere · builder@3 · workflow/keyframes — waiting on the Gate node before it writes.',
  },
  {
    kind: 'ask',
    title: 'locus ask — which migration path?',
    age: '11m',
    detail: 'loom-db · builder@4 — “partition by month, or by project then month?”',
  },
  {
    kind: 'guardrail',
    title: 'Guardrail — kill & reassign, 3 stuck iterations',
    age: '26m',
    detail: 'weaver · builder@4 · Cmd-chord capture — handoff drafted, needs a successor.',
  },
] as const

export const V2_RUNNING_PROJECTS = [
  { project: 'tapestry', detail: '3 running · builder@4, reviewer@2, keeper@1', state: 'running' },
  { project: 'loom-db', detail: '2 running · interviewer@3, researcher@1', state: 'running' },
  { project: 'weaver', detail: '2 running · builder@4 ×2', state: 'running' },
  { project: 'texere', detail: '1 waiting — gate, 4m', state: 'waiting' },
] as const

export const V2_TOKEN_DAYS = [
  [4, 8, 14, 37], [4, 10, 18, 47], [2, 2, 4, 12], [0, 4, 6, 10],
  [6, 12, 22, 55], [4, 10, 20, 61], [6, 8, 26, 51], [6, 14, 24, 67],
  [4, 12, 16, 57], [2, 4, 6, 16], [2, 2, 4, 14], [8, 12, 28, 63],
  [6, 16, 22, 71], [4, 10, 18, 43],
] as const

export const V2_MODEL_SCORECARD = [
  { model: 'opus-4.6', runs: '412', cache: '91%', verify: '84%', iterations: '2.1', cost: '$4.60', good: true },
  { model: 'gpt-5.2-pro', runs: '188', cache: '84%', verify: '79%', iterations: '2.6', cost: '$9.10', good: true },
  { model: 'gemini-3-ultra', runs: '141', cache: '88%', verify: '71%', iterations: '3.4', cost: '$3.20', good: false },
  { model: 'composer-2', runs: '96', cache: '79%', verify: '66%', iterations: '4.1', cost: '$2.10', good: false },
] as const

export const V2_DASHBOARD_COUNTERS = [
  { label: 'Steer vs review', value: '31 : 4', note: 'Steering the agent, versus recording that you read what it wrote.' },
  { label: 'Review debt', value: '38%', note: 'of 61 gates · falling from 52% two weeks ago.' },
  { label: 'Median time to land', value: '3h 12m', note: 'Task reaching the board to branch merged.' },
  { label: 'Cache read', value: '88%', note: 'Across all projects · +3pt.' },
] as const
