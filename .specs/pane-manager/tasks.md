# pane-manager — tasks

Tasks 14-16 are the ones PLAN.md warns against scheduling as an afternoon. Budget them.

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Pane manager: split, resize, focus, close, with panes as a tree | — | `pnpm -C apps/desktop test -- panes/manager` |
| 2 | ACP events flow host → webview over `Channel<Event>` | — | `cargo test -p locus-core ipc::event_channel` |
| 3 | `Channel<Event>` carries normalized events and token deltas | — | `cargo test -p locus-core ipc::event_channel` |
| 4 | `emit` for low-frequency notifications only | 3 | `cargo test -p locus-core ipc::emit_is_low_frequency` |
| 5 | Assert no high-frequency path uses `emit` | 4 | `bash apps/desktop/scripts/check-channel-not-emit.sh` |
| 6 | Frame-tick coalescing per pane | 2,3 | `pnpm -C apps/desktop test -- panes/coalesces` |
| 7 | Drive 1000 sends/sec and assert render count stays at frame rate | 6 | `pnpm -C apps/desktop test -- panes/coalesce-under-load` |
| 8 | Agent Pane renders only normalized ACP events — no PTY, no xterm | 2 | `pnpm -C apps/desktop test -- panes/agent-only` |
| 9 | Assert no terminal/PTY is attached to any run's pane | 3 | `pnpm -C apps/desktop test -- panes/no-terminal` |
| 10 | Minimize to strip tile | 1 | `pnpm -C apps/desktop test -- panes/minimize` |
| 11 | Promote from strip, demoting the least recently focused pane | 10 | `pnpm -C apps/desktop test -- panes/promote-demotes` |
| 12 | Assert promotion closes nothing | 11 | `pnpm -C apps/desktop test -- panes/promotion-closes-nothing` |
| 13 | Register accelerators in Rust (no default menu equivalents) | — | `cargo test -p locus-tauri menu::no_default_key_equivalents` |
| 14 | Detach a pane into a second Tauri window running the app in detached mode | 1 | `pnpm -C apps/desktop test -- panes/detach-window` |
| 15 | Assert webview count per window never exceeds one | 14 | `cargo test -p locus-tauri window::one_webview_each` |
| 16 | Detached window subscribes to the same bus, with no JS state sync | 14 | `pnpm -C apps/desktop test -- panes/detached-shares-bus` |
| 17 | Cap focused panes at four; the rest go to the strip | 12 | `pnpm -C apps/desktop test -- panes/four-focused-max` |
