# acp-client — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Add `agent-client-protocol`; wire a stdio transport | — | `cargo test -p locus-core acp::transport` |
| 2 | `session/new` with `cwd` and an empty `mcpServers` | 1 | `cargo test -p locus-core acp::session_new` |
| 3 | Assert `mcpServers` is empty on every call | 2 | `cargo test -p locus-core acp::mcp_always_empty` |
| 4 | `session/prompt` and the streamed update subscription | 2 | `cargo test -p locus-core acp::prompt_streams` |
| 5 | Attach stdio to a container process rather than a host one | 2 | `cargo test -p locus-core acp::runs_in_container` |
| 6 | Assert the ACP agent's process is not on the host | 5 | `cargo test -p locus-core acp::not_on_host` |
| 7 | Shared `session/update` → verb mapping | 4 | `cargo test -p locus-core acp::update_mapping` |
| 8 | `ToolCallUpdate.status` → `tool_result` or `tool_error` | 7 | `cargo test -p locus-core acp::tool_status_split` |
| 9 | `RequestPermission` → `permission_request` and the alarm | 7 | `cargo test -p locus-core acp::permission_request` |
| 10 | A second ACP harness needs no new mapping code | 7 | `cargo test -p locus-core acp::mapping_is_shared` |
| 11 | ACP events indistinguishable downstream from hooks events | 7 | `cargo test -p locus-core acp::indistinguishable` |
| 12 | Assert no code path starts an ordinary agent session over ACP | 4 | `cargo test -p locus-core acp::planning_only` |
| 13 | Expose the conversation to the Plan screen over IPC | 11 | `pnpm -C apps/desktop test -- plan/conversation-from-core` |
