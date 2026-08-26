# desktop-workshop-runtime

> Superseded by `workshop-revision` for the Extensions contract and `workshop-plugins` for the Plugins subgroup; this file records M0.6 history.

**Milestone** M0.6 · **Depends on** `desktop-application-shell`, `theme-system` · **Blocks** M1 runtime
configuration and M4 workflow execution.

## Purpose

Make desktop Workshop configuration real: providers, Minisign-verified CLI tools, adapter-gated harnesses,
extensions, agent definitions, and Workflow Visual/Governance. It also defines the backend boundaries
those screens require.

## Contract

Provider secrets are OS-keychain references resolved only by the host broker. A provider owns model
aliases and verification metadata. Harness selection requires an adapter and configured provider;
six-band autorouting records its decision. Built-in and verified custom tools form image sets, then
project/role scopes subtract. Workflow Visual contains only graph structure; Governance owns goal,
named guardrails, and checked success criteria.

## Acceptance

No secret reaches persistence, events, logs, UI state, or containers. An unsigned/untrusted tool never
enters an image. Router choices, effective tools/extensions, and governance outcomes are reproducible
from stored state.
