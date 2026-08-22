import { createSignal, For, Show } from 'solid-js'
import { Button } from '../../ui/Button'
import { Input } from '../../ui/Input'
import { Segmented } from '../../ui/Segmented'
import { Tag } from '../../ui/Tag'
import './workshop-fixtures.css'

export const WORKSHOP_FIXTURES = [
  'agents',
  'cli',
  'commands',
  'harnesses',
  'hooks',
  'linters',
  'output-styles',
  'providers',
  'rules',
  'skills',
  'workflows-visual',
  'workflows-governance',
] as const

export type WorkshopFixture = (typeof WORKSHOP_FIXTURES)[number]

export interface WorkshopFixtureViewProps {
  fixture: WorkshopFixture
}

const EXTENSIONS = {
  commands: ['check-pr', 'handoff', 'release-notes'],
  hooks: ['session-start', 'before-tool', 'session-end'],
  linters: ['no-secrets', 'no-todo-comments', 'typed-boundaries'],
  'output-styles': ['brief-bright-gone', 'technical-review', 'release-notes'],
  rules: ['no-secrets', 'rust-style', 'desktop-patterns'],
  skills: ['verify-loop', 'incident-response', 'spec-decomposition'],
} as const

const CLI_GROUPS = [
  { id: 'source-control', label: 'Source control', tools: ['git', 'gh', 'delta'] },
  { id: 'search-files', label: 'Search & files', tools: ['rg', 'fd', 'jq'] },
  { id: 'rust', label: 'Rust', tools: ['cargo', 'rustfmt', 'clippy'] },
  { id: 'database', label: 'Database', tools: ['psql', 'sqlx'] },
  { id: 'network', label: 'Network', tools: ['curl', 'httpie'] },
] as const

function Toggle(props: { on: boolean; onClick: () => void; label: string; testId?: string }) {
  return (
    <button
      type="button"
      class="ws-toggle"
      aria-label={props.label}
      aria-pressed={props.on}
      data-testid={props.testId}
      data-on={props.on ? 'true' : 'false'}
      onClick={props.onClick}
    >
      <span />
    </button>
  )
}

function ExtensionFixture(props: { fixture: keyof typeof EXTENSIONS }) {
  const label = props.fixture === 'output-styles' ? 'Output styles' : props.fixture[0].toUpperCase() + props.fixture.slice(1)
  return (
    <div class="ws-fixture ws-extension" data-testid={`workshop-${props.fixture}`}>
      <header class="ws-fixture-head">
        <div>
          <h1>{label}</h1>
          <p>Authored once in Workshop and materialized fresh into every runtime at run start.</p>
        </div>
        <div class="ws-actions"><Input placeholder={`Search ${props.fixture}`} /><Button variant="primary">New {label.slice(0, -1)}</Button></div>
      </header>
      <section class="ws-list-card">
        <For each={EXTENSIONS[props.fixture]}>
          {(entry, index) => (
            <article class="ws-entry" data-testid={`workshop-${props.fixture}-${entry}`}>
              <div><strong>{entry}</strong><p>{index() === 0 ? 'Project default · versioned' : 'Shared extension · versioned'}</p></div>
              <Tag variant="neutral">v{4 - index()}</Tag>
              <Button variant="ghost">Edit</Button>
            </article>
          )}
        </For>
      </section>
      <p class="ws-footnote">Changes affect the next run only; the existing materialized trees remain immutable.</p>
    </div>
  )
}

function AgentsFixture() {
  return (
    <div class="ws-fixture" data-testid="workshop-agents">
      <header class="ws-fixture-head"><div><h1>Agents</h1><p>Markdown plus a tool list. Agent definitions are versioned documents, not a canvas.</p></div><Button variant="primary">New agent</Button></header>
      <section class="ws-list-card">
        <For each={['builder', 'reviewer', 'researcher', 'auditor']}>
          {(agent, index) => <article class="ws-entry"><div><strong>{agent}@{4 - index()}</strong><p>high tier · scoped tools · project memory</p></div><Tag variant="neutral">active</Tag><Button variant="ghost">Open</Button></article>}
        </For>
      </section>
    </div>
  )
}

function CliFixture() {
  const [enabled, setEnabled] = createSignal<Record<string, boolean>>({ git: true, gh: false, delta: true, rg: true, fd: true, jq: true, cargo: true, rustfmt: true, clippy: true, psql: false, sqlx: true, curl: true, httpie: false })
  const toggleGroup = (group: (typeof CLI_GROUPS)[number]) => {
    const all = group.tools.every((tool) => enabled()[tool])
    setEnabled((current) => ({ ...current, ...Object.fromEntries(group.tools.map((tool) => [tool, !all])) }))
  }
  const groupState = (group: (typeof CLI_GROUPS)[number]) => {
    const count = group.tools.filter((tool) => enabled()[tool]).length
    return count === 0 ? 'off' : count === group.tools.length ? 'on' : 'mixed'
  }
  return (
    <div class="ws-fixture ws-cli" data-testid="workshop-cli">
      <header class="ws-fixture-head"><div><h1>CLI</h1><p>Enabled tools are baked into the image once, never installed per run.</p></div><div class="ws-actions"><Input placeholder="Filter tools" /><Button variant="primary">Upload a CLI</Button></div></header>
      <div class="ws-fixture-columns">
        <main>
          <p class="ws-intro">A tool switched on here can be added to any project. Off means it is not in the image at all.</p>
          <For each={CLI_GROUPS}>{(group) => <section class="ws-tool-group" data-testid={`cli-group-${group.id}`} data-state={groupState(group)}><header><span>{group.label}</span><code>{group.tools.filter((tool) => enabled()[tool]).length} / {group.tools.length}</code><Toggle label={`Toggle ${group.label}`} testId={`cli-group-toggle-${group.id}`} on={groupState(group) === 'on'} onClick={() => toggleGroup(group)} /></header><For each={group.tools}>{(tool) => <div class="ws-tool"><code>{tool}</code><span>{tool === 'git' ? 'version control' : 'available in enabled images'}</span><Toggle label={`Toggle ${tool}`} on={enabled()[tool]} onClick={() => setEnabled((current) => ({ ...current, [tool]: !current[tool] }))} /></div>}</For></section>}</For>
          <section class="ws-tool-group"><header><span>Your own</span><span /></header><div class="ws-tool"><code>repo-audit</code><span>unsigned · inspect repository policy</span><Tag variant="outline">unsigned</Tag></div><div class="ws-drop-zone">Drop a binary, a script, or a <code>tool.toml</code><small>Install and verify fields are recorded before an image build.</small></div><div class="ws-install-fields"><Input mono placeholder="install" value="cargo install --git …" readOnly /><Input mono placeholder="verify it landed" value="<tool> --version" readOnly /></div><p id="cli-unsigned-note" data-testid="cli-unsigned-note">An unsigned tool is available to read-only roles only.</p></section>
        </main>
        <aside class="ws-side-cards"><section><h2>Image</h2><p>Enabled tools are baked into the base image, not installed per run.</p><strong class="mono">1.9 GB</strong><small>rebuilt 2h ago</small></section><section><h2>Most reached for</h2><For each={['git · 2,481', 'rg · 1,920', 'cargo · 864']}>
          {(tool) => <div class="ws-usage"><span>{tool}</span><i /></div>}
        </For></section></aside>
      </div>
    </div>
  )
}

function ProvidersFixture() {
  const [selected, setSelected] = createSignal('Anthropic')
  const providers = ['Anthropic', 'OpenRouter', 'OpenAI', 'Gemini', 'Local']
  return (
    <div class="ws-providers" data-testid="workshop-providers">
      <aside class="ws-provider-list"><header><h1>Providers</h1><p>Credentials and the short list of models you actually use.</p><Button variant="primary">Add provider</Button></header><For each={providers}>{(provider, index) => <button type="button" class="ws-provider-row" aria-selected={selected() === provider} onClick={() => setSelected(provider)}><i data-state={index() < 3 ? 'ok' : 'unconfigured'} />{provider}<small>{index() === 0 ? '3 preferred' : 'not configured'}</small></button>}</For><footer id="provider-keychain-note" data-testid="provider-keychain-note">Secrets live in the OS keychain. Locus stores the reference and the model list, never the key.</footer></aside>
      <main class="ws-provider-main"><header class="ws-fixture-head"><div><h1>{selected()}</h1><p class="mono">provider/{selected().toLowerCase()}</p></div><div class="ws-actions"><Button variant="secondary">Test connection</Button><Button variant="primary">Save</Button></div></header><section><h2>Authentication</h2><div class="ws-settings-card"><div><span>method</span><Segmented label="Authentication method" value="api-key" onChange={() => undefined} options={[{ value: 'oauth', label: 'OAuth' }, { value: 'api-key', label: 'API key' }, { value: 'none', label: 'None' }]} /></div><div><span>API key</span><code id="provider-secret" data-testid="provider-secret">••••••••••••••••</code><Button variant="ghost">Reveal</Button><Button variant="ghost">Replace</Button><small>keychain</small></div><div><span>base_url</span><Input mono value="https://api.anthropic.com" readOnly /><small>optional override</small></div><p id="provider-verification" data-testid="provider-verification">verified 11m ago · 327 models listed</p></div></section><section><h2>Preferred models</h2><p>Aliases are what every model selector shows from then on, for every harness using this provider.</p><div class="ws-model-table"><div class="ws-model-head"><span>Model</span><span>Alias</span><span>Context</span><span>In / out per M</span><span>In selector</span></div><For each={[['claude-opus-4-6', 'opus', '200k', '$15 / $75'], ['claude-sonnet-4-5', 'sonnet', '200k', '$3 / $15'], ['claude-haiku-4-5', 'haiku', '200k', '$1 / $5']]}>{([id, alias, context, price]) => <div class="ws-model-row"><code>{id}</code><strong id={`provider-model-alias-${alias}`} data-testid={`provider-model-alias-${alias}`}>{alias}</strong><code>{context}</code><code>{price}</code><Toggle label={`Include ${alias}`} on={true} onClick={() => undefined} /></div>}</For><Input class="ws-catalog-search" placeholder="Search catalogue — 4 of 327 match" /></div></section></main>
      <aside class="ws-provider-preview"><h2>Selector preview</h2><div id="provider-selector-preview" data-testid="provider-selector-preview"><strong>opus</strong><span>Anthropic · 200k context</span></div><h2>Used by</h2><Tag variant="neutral">claude</Tag><Tag variant="neutral">cursor</Tag><h2>30-day spend</h2><strong class="mono">$418.20</strong></aside>
    </div>
  )
}

function HarnessesFixture() {
  const bands = ['xtra-low', 'low', 'medium', 'high', 'xtra-high', 'max']
  const [autorouting, setAutorouting] = createSignal(true)
  return (
    <div class="ws-fixture" data-testid="workshop-harnesses">
      <header class="ws-fixture-head"><div><h1>Harnesses</h1><p>One record per harness: providers, defaults, and adapter configuration.</p></div><Button variant="primary">Register a harness</Button></header>
      <div class="ws-fixture-columns"><main><section class="ws-settings-card"><div><span>identifier</span><code>claude</code></div><div id="harness-adapter-gate" data-testid="harness-adapter-gate"><span>adapter</span><strong>built-in · v3</strong><small>no adapter, no selection — anywhere</small></div><div><span>providers</span><Tag variant="neutral">Anthropic</Tag><Tag variant="neutral">OpenRouter</Tag></div><div><span>default model</span><strong>opus · Anthropic</strong></div><div><span>default effort</span><strong>high</strong></div></section><section class="ws-routing"><header><h2>Autorouting</h2><Toggle label="Enable autorouting" on={autorouting()} onClick={() => setAutorouting(!autorouting())} /></header><Show when={autorouting()}><div class="ws-band-table"><div><span>Band</span><span>Model</span><span>Effort</span><span>Approval</span><span>When to use</span></div><For each={bands}>{(band, index) => <div data-testid={`autoroute-band-${band}`}><strong>{band}</strong><span>{index() === 4 ? '—' : index() > 3 ? 'opus' : 'sonnet'}</span><span>{index() > 3 ? 'high' : 'medium'}</span><span>{index() === 5 ? '✓' : '—'}</span><small>{index() === 5 ? 'irreversible or production work' : 'routine implementation'}</small></div>}</For></div><p id="autoroute-fallback" data-testid="autoroute-fallback">A missing model falls upward to the next configured band.</p></Show></section></main><aside class="ws-side-cards"><section><h2>Adapter config</h2><div class="ws-kv"><code>permission-mode</code><span>bypass</span><Tag variant="neutral">string</Tag></div><Button variant="ghost">Add config key</Button></section></aside></div>
    </div>
  )
}

function WorkflowsFixture(props: { governance: boolean }) {
  const [tab, setTab] = createSignal(props.governance ? 'governance' : 'visual')
  return (
    <div class="ws-fixture ws-workflows" data-testid={`workshop-workflows-${props.governance ? 'governance' : 'visual'}`}>
      <header class="ws-fixture-head"><div><h1>Release verification</h1><p>saved 2s ago · authoring only</p></div><Segmented label="Workflow authoring view" value={tab()} onChange={setTab} options={[{ value: 'visual', label: 'Visual' }, { value: 'governance', label: 'Governance' }]} /></header>
      <Show when={tab() === 'visual'} fallback={<Governance />}>{<div class="ws-workflow-visual"><aside><h2>Nodes</h2><For each={['Agent', 'Task', 'Loop', 'Condition', 'Gate', 'Verify']}>
        {(node) => <button type="button" class="ws-node-chip">{node}</button>}
      </For><h2>Presets</h2><p>Plan → build → verify</p></aside><main><div class="ws-dot-grid"><div class="ws-flow-node">Agent<br /><strong>Build release</strong></div><div class="ws-flow-node">Verify<br /><strong>pnpm test</strong></div><svg aria-label="Workflow edges"><line x1="165" y1="92" x2="295" y2="92" /></svg></div><p>No model in the orchestration path — the graph decides.</p></main><aside><h2>Inspector</h2><p>Condition · verify.passed</p><code>verify.passed == true</code><p>Goal lives in Governance.</p></aside></div>}</Show>
    </div>
  )
}

function Governance() {
  return <div class="ws-governance"><section><h2>Goal</h2><p id="workflow-governance-goal" data-testid="workflow-governance-goal">Ship a release only when every required verification command passes.</p></section><section><h2>Guardrails</h2><article><strong>Preserve branches</strong><p>Never work in main or master; stop at a PR boundary.</p></article><Button variant="ghost">Add a guardrail</Button></section><section id="workflow-success-criteria" data-testid="workflow-success-criteria"><h2>Success criteria</h2><div><Tag variant="neutral">command</Tag><code>pnpm test</code><span>checked by core</span></div><div><Tag variant="neutral">human</Tag><span>Release notes approved</span><span>checked by a gate</span></div></section></div>
}

export function WorkshopFixtureView(props: WorkshopFixtureViewProps) {
  if (props.fixture === 'agents') return <AgentsFixture />
  if (props.fixture === 'cli') return <CliFixture />
  if (props.fixture === 'providers') return <ProvidersFixture />
  if (props.fixture === 'harnesses') return <HarnessesFixture />
  if (props.fixture === 'workflows-visual') return <WorkflowsFixture governance={false} />
  if (props.fixture === 'workflows-governance') return <WorkflowsFixture governance />
  return <ExtensionFixture fixture={props.fixture} />
}

export default WorkshopFixtureView
