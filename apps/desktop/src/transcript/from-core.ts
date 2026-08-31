import { Channel, invoke } from '@tauri-apps/api/core'
import type { AgentEvent } from '../types/event'

/** Replay a run's persisted events from `agents.events` (the durable record). */
export async function replayRunEvents(
  runId: string,
): Promise<AgentEvent[]> {
  return invoke<AgentEvent[]>('telemetry_events_replay', { runId });
}

/** Subscribe a transcript pane to the core's source-neutral normalized event stream. */
export async function streamFromCore(onEvent: (event: AgentEvent) => void): Promise<Channel<AgentEvent>> {
  const channel = new Channel<AgentEvent>()
  channel.onmessage = onEvent
  await invoke('telemetry_subscribe', { channel })
  return channel
}
