# desktop-application-shell

> Superseded by `shell-revision` for the current shell contract; this file records M0.6 history.

**Milestone** M0.6 · **Depends on** `design-desktop`, `theme-system` · **Blocks** all desktop views.

## Purpose

Replace the v1 shell with the desktop title bar, project-scoped rail, project switcher, running-agent
popover, locator, keyboard navigation, and accessible shared primitives. The shell remains one Tauri
window and never ports the handoff HTML or `support.js`.

## Contract

The 42px title bar exposes product/view labels, locator, and a running-agent pill. The 212px rail has
global and selected-project regions, expandable Memory/Workshop groups, a visible dispatch state, and
Cmd-K. Selecting a project scopes only project views. Every global view declares its scope. Shell state
persists across restart; keyboard and screen-reader paths have the same resolver behavior.

## Acceptance

- Every desktop route renders through the shared shell in Dark and Light.
- Rail, locator, palette, history, selected project, expansion state, and running popover are keyboard
  operable and resolver-backed.
- No component can write a view or project scope directly.
- The shell announces action-required changes but suppresses continuous run-stream noise.
