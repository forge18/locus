export type GuardrailControl =
  | { kind: 'stepper'; value: string }
  | { kind: 'toggle'; value: boolean }
  | { kind: 'select'; value: string }

export interface GuardrailSetting {
  id: string
  label: string
  description: string
  control: GuardrailControl
}

export interface GuardrailSection {
  id: string
  label: string
  settings: readonly GuardrailSetting[]
}

export const SETTINGS_NAVIGATION = Object.freeze([
  'Guardrails',
  'Inbox & notifications',
  'Harnesses',
  'Repositories',
  'Store',
  'Appearance',
  'Account',
])

export const GUARDRAIL_SECTIONS: readonly GuardrailSection[] = Object.freeze([
  {
    id: 'stopping',
    label: 'Stopping conditions',
    settings: [
      {
        id: 'max-iterations',
        label: 'Max iterations',
        description:
          'The loop stops here whatever state it is in. Reaching it is a result, not a failure — the run hands off rather than dying.',
        control: { kind: 'stepper', value: '8' },
      },
      {
        id: 'token-budget',
        label: 'Token budget per run',
        description:
          'Counted across the whole session, not per iteration. At 80% the run is told, so it can spend what is left on a handoff instead of on work it cannot finish.',
        control: { kind: 'stepper', value: '120k' },
      },
      {
        id: 'stuck-detection',
        label: 'Stuck detection',
        description:
          'Consecutive iterations with no file write before the run is flagged and, if kill & reassign is on, replaced.',
        control: { kind: 'stepper', value: '3' },
      },
      {
        id: 'kill-reassign',
        label: 'Kill & reassign on stuck',
        description:
          'The successor starts from the handoff payload, not the transcript. Off means the run is only flagged and waits for you.',
        control: { kind: 'toggle', value: true },
      },
    ],
  },
  {
    id: 'parallelism',
    label: 'Parallelism',
    settings: [
      {
        id: 'max-parallel-agents',
        label: 'Max parallel agents',
        description:
          'Across every project. A run that would exceed the cap waits in the dispatch queue rather than starting thin — twelve agents at a third of the tokens each finish nothing.',
        control: { kind: 'stepper', value: '6' },
      },
      {
        id: 'max-per-project',
        label: 'Max per project',
        description:
          'So one busy project cannot take the whole budget. Counted against the same pool as the global cap, never in addition to it.',
        control: { kind: 'stepper', value: '3' },
      },
      {
        id: 'priority-method',
        label: 'Priority method',
        description: 'How the queue decides who starts next when the cap is full.',
        control: { kind: 'select', value: 'plan order' },
      },
      {
        id: 'tie-break',
        label: 'Tie-break',
        description: 'Applied when two cards rank equally under the method above.',
        control: { kind: 'select', value: 'longest waiting' },
      },
      {
        id: 'preempt',
        label: 'Preempt a running agent',
        description:
          'Off means a higher-priority card waits for a slot. On, it pauses the lowest-priority run at its next iteration boundary and takes the slot — the paused run keeps its handoff, not its context.',
        control: { kind: 'toggle', value: false },
      },
    ],
  },
  {
    id: 'change-size',
    label: 'Change size',
    settings: [
      {
        id: 'lines-changed',
        label: 'Lines changed ceiling',
        description:
          'An agent approaching this stops and splits the work. Review effectiveness falls off with patch size, and past a reviewer’s capacity it silently degrades to syntactic checking.',
        control: { kind: 'stepper', value: '400' },
      },
      {
        id: 'files-touched',
        label: 'Files touched ceiling',
        description:
          'The same guardrail on the other axis. Ten new files of a thousand lines each is the failure this exists to prevent.',
        control: { kind: 'stepper', value: '12' },
      },
      {
        id: 'on-breach',
        label: 'On breach',
        description: 'What happens when a ceiling is hit mid-iteration.',
        control: { kind: 'select', value: 'stop and split' },
      },
    ],
  },
  {
    id: 'permissions',
    label: 'Permissions',
    settings: [
      {
        id: 'network-tier',
        label: 'Network tier for new agents',
        description: 'The default sandbox tier. Individual agent definitions may request less, never more.',
        control: { kind: 'select', value: 'allowlist' },
      },
      {
        id: 'block-system-changes',
        label: 'Block unapproved system changes',
        description: 'Writes outside the working tree require a gate. Sixty percent of developers run with this on.',
        control: { kind: 'toggle', value: true },
      },
      {
        id: 'autopilot',
        label: 'Autopilot',
        description: 'Off means every irreversible step is gated. Sixty-three percent of developers rarely or never turn this on.',
        control: { kind: 'toggle', value: false },
      },
    ],
  },
])

export const PRIORITY_METHODS = Object.freeze([
  ['plan order', 'the order the plan approved its cards; dependencies first'],
  ['manual', 'the board’s own card order, dragged by you'],
  ['unblocks-most', 'whichever card the most other cards are waiting on'],
  ['shortest first', 'smallest estimate, for draining a backlog'],
] as const)
