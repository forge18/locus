# pane-manager — tasks

Tasks 14-16 are the ones PLAN.md warns against scheduling as an afternoon. Budget them.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Pane manager: split, resize, focus, close, with panes as a tree | — | `pnpm -C apps/desktop test -- panes/manager` |
| 2 | `Channel<&[u8]>` for PTY bytes, host to webview | — | `cargo test -p locus-core ipc::pty_channel` |
| 3 | `Channel<Event>` for normalized events | — | `cargo test -p locus-core ipc::event_channel` |
| 4 | `emit` for low-frequency notifications only | 3 | `cargo test -p locus-core ipc::emit_is_low_frequency` |
| 5 | Assert no high-frequency path uses `emit` | 4 | `bash apps/desktop/scripts/check-channel-not-emit.sh` |
| 6 | Frame-tick coalescing per pane | 2,3 | `pnpm -C apps/desktop test -- panes/coalesces` |
| 7 | Drive 1000 sends/sec and assert render count stays at frame rate | 6 | `pnpm -C apps/desktop test -- panes/coalesce-under-load` |
| 8 | Shell Pane on xterm.js with a real PTY | 2 | `pnpm -C apps/desktop test -- panes/shell-pty` |
| 9 | Agent Pane as a typed event stream with no PTY | 3 | `pnpm -C apps/desktop test -- panes/agent-no-pty` |
| 10 | Assert the two are distinct components, not one with a flag | 8,9 | `pnpm -C apps/desktop test -- panes/types-are-distinct` |
| 11 | Minimize to strip tile | 1 | `pnpm -C apps/desktop test -- panes/minimize` |
| 12 | Promote from strip, demoting the least recently focused pane | 11 | `pnpm -C apps/desktop test -- panes/promote-demotes` |
| 13 | Assert promotion closes nothing | 12 | `pnpm -C apps/desktop test -- panes/promotion-closes-nothing` |
| 14 | `macOptionIsMeta` and a custom key handler; `vim` survives | 8 | `pnpm -C apps/desktop test -- panes/vim-survives` |
| 15 | Ship no default menu; register accelerators in Rust | — | `cargo test -p locus-tauri menu::no_default_key_equivalents` |
| 16 | Cmd chords reach the app, not the menu bar | 15 | `pnpm -C apps/desktop test -- panes/cmd-chords` |
| 17 | IME composition and dead keys in a Shell Pane | 8 | `pnpm -C apps/desktop test -- panes/ime` |
| 18 | Detach a pane into a second Tauri window running the app in detached mode | 1 | `pnpm -C apps/desktop test -- panes/detach-window` |
| 19 | Assert webview count per window never exceeds one | 18 | `cargo test -p locus-tauri window::one_webview_each` |
| 20 | Detached window subscribes to the same bus, with no JS state sync | 18 | `pnpm -C apps/desktop test -- panes/detached-shares-bus` |
| 21 | Cap focused panes at four; the rest go to the strip | 12 | `pnpm -C apps/desktop test -- panes/four-focused-max` |
