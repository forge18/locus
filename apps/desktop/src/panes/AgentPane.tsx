import { createSignal, onCleanup, onMount } from 'solid-js'
import { coalesce } from './coalesce'
import { streamFromCore } from '../transcript/from-core'
import type { AgentEvent } from '../types/event'

export interface AgentPaneProps { runId: string }
export const agentPaneTransport = 'event-channel' as const

/** A typed telemetry transcript. This component never opens a PTY. */
export function AgentPane(props: AgentPaneProps) {
  const [events, setEvents] = createSignal<AgentEvent[]>([])
  let stopped = false
  onMount(async () => {
    const frames = coalesce<AgentEvent>((items) => setEvents((current) => [...current, ...items]))
    await streamFromCore((event) => { if (!stopped && event.runId === props.runId) frames.push(event) })
    onCleanup(() => { stopped = true; frames.stop() })
  })
  return <section class="agent-pane" data-testid="agent-pane" data-run-id={props.runId} data-pty="false"><ol>{events().map((event) => <li>{event.verb}</li>)}</ol></section>
}
