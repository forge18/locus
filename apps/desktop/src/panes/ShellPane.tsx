import { Terminal } from '@xterm/xterm'
import { Channel, invoke } from '@tauri-apps/api/core'
import { onCleanup, onMount } from 'solid-js'
import { coalesce } from './coalesce'
import { reachesTerminal, terminalOptions } from './shell-config'

export interface ShellPaneProps { runId: string; title?: string }

/** A PTY-backed pane. It owns an xterm instance; AgentPane deliberately does not. */
export function ShellPane(props: ShellPaneProps) {
  let host: HTMLDivElement | undefined
  let terminal: Terminal | undefined

  onMount(async () => {
    terminal = new Terminal(terminalOptions)
    terminal.attachCustomKeyEventHandler(reachesTerminal)
    terminal.open(host!)
    const frames = coalesce<Uint8Array>((chunks) => chunks.forEach((chunk) => terminal?.write(chunk)))
    const channel = new Channel<number[]>()
    channel.onmessage = (bytes) => frames.push(Uint8Array.from(bytes))
    await invoke('pty_subscribe', { runId: props.runId, channel })
    onCleanup(() => { frames.stop(); terminal?.dispose() })
  })

  return <section class="shell-pane" data-testid="shell-pane" data-run-id={props.runId} data-pty="true"><div class="shell-pane-terminal" ref={host} /></section>
}
