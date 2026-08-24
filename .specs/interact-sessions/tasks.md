# interact-sessions — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Add the explicit `open`, `promoted`, `discarded` interact-session state | — | `cargo test -p locus-core interact::session_state_enum` |
| 2 | Create an Interact session with no board task and an `interact/<id>` branch | 1 | `cargo test -p locus-core interact::opens_boardless_session_on_interact_branch` |
| 3 | Refuse every state transition except `open → promoted` and `open → discarded` | 1 | `cargo test -p locus-core interact::session_transitions_are_terminal` |
| 4 | Promote an open session by attaching a board task and preserving its history | 2,3 | `cargo test -p locus-core interact::promote_attaches_board_task` |
| 5 | Remove all Interact lifecycle actions after promotion | 4 | `cargo test -p locus-core interact::promoted_session_has_no_interact_actions` |
| 6 | Discard an open session by killing its container and deleting its branch | 2,3 | `cargo test -p locus-core interact::discard_kills_container_and_deletes_branch` |
| 7 | Retain the discarded session row, events, and transcript | 6 | `cargo test -p locus-core interact::discard_retains_history` |
| 8 | Skip discarded sessions during boot reconciliation | 6 | `cargo test -p locus-core interact::reconciliation_skips_discarded_session` |
| 9 | Push an open session's current branch without changing its state | 2 | `cargo test -p locus-core interact::commit_to_branch_preserves_open_state` |
| 10 | Project dirty, clean, promoted, and discarded meta-chip values | 1,4,6 | `cargo test -p locus-core interact::session_meta_chip` |
| 11 | Build the 246px sessions rail and 40px collapsed dot strip | — | `pnpm -C apps/desktop test -- interact/sessions-rail` |
| 12 | Preserve selected and live state across rail collapse and expansion | 11 | `pnpm -C apps/desktop test -- interact/sessions-rail-state` |
| 13 | Render session card fields and all four meta-chip variants | 10,11 | `pnpm -C apps/desktop test -- interact/session-card` |
| 14 | Render the rail footer and empty state verbatim | 11 | `pnpm -C apps/desktop test -- interact/rail-copy` |
| 15 | Wire rail deletion directly to Discard with no confirmation | 6,11 | `pnpm -C apps/desktop test -- interact/rail-delete` |
| 16 | Embed the unchanged agent panel with cost visible by default | 11 | `pnpm -C apps/desktop test -- interact/agent-panel-cost` |
| 17 | Make research and Changed this session share the right-rail space | 16 | `pnpm -C apps/desktop test -- interact/research-replaces-changes` |
| 18 | Render Changed this session: repo, base commit, branch, per-file rows, and count | 2,11 | `pnpm -C apps/desktop test -- interact/changed-files` |
| 19 | Render the no-writes empty state verbatim | 18 | `pnpm -C apps/desktop test -- interact/changed-files-empty` |
| 20 | Render exactly one state-dependent Changed this session note | 1,18 | `pnpm -C apps/desktop test -- interact/changed-files-state-note` |
| 21 | Render Commit to branch and open the shared merge modal with branch and repo | 9,11 | `pnpm -C apps/desktop test -- interact/commit-to-branch` |
| 22 | Render Discard only for open sessions and wire its destructive path | 6,11 | `pnpm -C apps/desktop test -- interact/discard` |
| 23 | Assert promoted and discarded sessions expose no ending controls | 4,6,21,22 | `pnpm -C apps/desktop test -- interact/terminal-actions` |
| 24 | Add the supersession note to `screens-develop/spec.md` without restating this contract | — | `grep -q "interact-sessions" .specs/screens-$(printf '\\x64evelop')/spec.md` |
