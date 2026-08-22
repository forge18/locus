# pane-manager

**Milestone** M1 · **Depends on** `app-shell`, `run-supervisor`, `telemetry`

## Purpose

The pane manager and the IPC discipline behind it. PLAN.md calls panes and tiles product rather than
chrome, which is why Kobalte ships no split panes and correctly so.

Two constraints here are cheap to follow from the start and expensive to retrofit: **channels for
streams, events for notifications**, and **never two webviews in one window**.

## Governed by

- PLAN.md §Frontend and IPC constraints — the transport table, multiwebview, the component split
- PLAN.md §Sessions do not all fit, so most are strips
- `apps/desktop/src/panes/README.md`

## Contract

**Two pane types** (the Shell/PTY pane is retired):

| Pane | Is |
| --- | --- |
| Agent Pane | a typed event stream — the ACP conversation from `telemetry`, rendered as events |
| Editor Pane | CodeMirror at side-pane zoom (`editor`, M2) |

**One session per run, always.** Every agent is one ACP conversation over stdio in one container; the
Agent Pane is how you watch it. There is no terminal memory to render, no xterm.js, no second surface.

**Transport, by frequency:**

| Path | Mechanism |
| --- | --- |
| Normalized events, including token deltas | `Channel<Event>` |
| LSP diagnostics and semantic tokens | `Channel<T>` |
| "a run finished", "a task moved", "a guardrail tripped" | `emit` — low frequency, many listeners |

Tauri's own documentation says the event system "is not designed for low latency or high throughput
situations" and works by evaluating JavaScript. **Coalesce per pane on an animation-frame tick
regardless of mechanism** — a thousand small sends per second will drop frames whatever the transport.

**Never two webviews in one window; any number of windows.** Multiwebview is behind an unstable flag
and less mature than Electron's renderer-per-window model. Multi-window is ordinary. Detachable panels
are **additional windows, one webview each**, running the same Solid app in detached mode.

This costs almost nothing here: panes are views over runs, the Rust core and its bus are the source of
truth, and a detached window subscribes to the same bus. **There is no shared JS state to synchronize
because there is no shared JS state.**

**One to four focused panes; everything else is a strip entry.** Promoting an entry demotes the least
recently focused pane. **Nothing is closed by promotion** — a session you stopped watching is not a
session you ended.

**Keyboard is not a terminal problem here**, because there is no terminal on the agent path. The agent
renders as events, so there is no xterm.js, no `macOptionIsMeta`, no `attachCustomKeyEventHandler`,
 no IME/dead-key axis to solve on the agent pane.

## Acceptance

1. Normalized events arrive over `Channel<Event>`; nothing high-frequency uses `emit`.
2. Every pane coalesces on an animation-frame tick — asserted by driving 1000 sends/sec and counting
   renders, not by reading the code.
3. A detached pane is a **second window with one webview**; a test asserts the webview count per window
   is never above one.
4. A detached window receives the same events as the original with no JS-side state sync.
5. Promoting a strip entry demotes the least recently focused pane and **closes nothing**.
6. An Agent Pane renders only normalized events; **no terminal, no PTY, no xterm are attached to any
   run** — a test asserts the agent process has no terminal surface.
7. The Editor Pane renders side-pane CodeMirror alongside an Agent Pane.

## Open

- Whether pane layout persists per project or globally. PLAN.md puts pane state on the session, which
  suggests per session, but says nothing about the arrangement itself.
