# @locus/desktop

The Locus desktop application: Tauri 2 + SolidJS + Vite + TypeScript.

## Run

```bash
pnpm install     # from this directory
pnpm tauri dev   # dev server + native window
pnpm tauri build # release bundle
```

`cargo build` from the repo root builds the whole workspace, including this app's
Rust half at `src-tauri/`.

## Layout

| Path | Holds |
| --- | --- |
| `src/panes/` | The pane manager — Agent Panes, Shell Panes, the editor pane, tiles |
| `src/workflow-canvas/` | `solid-flow` node editor for Workflows |
| `src/ui/` | shadcn-solid components, copied in and owned here |
| `src-tauri/` | The `locus-desktop` crate — a workspace member, not its own workspace |

## Constraints

Two rules from PLAN.md that are cheap now and expensive to retrofit:

- **Stream over `tauri::ipc::Channel`, not `emit`.** The event system is documented as
  unsuited to high throughput. PTY bytes, agent events, and LSP diagnostics are Channels;
  `emit` is for low-frequency notifications. Coalesce per pane on a frame tick.
- **One webview per window.** Never two webviews in a single window. Detachable panels
  are additional Tauri windows running this same app in detached mode.
