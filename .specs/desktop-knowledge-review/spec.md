# desktop-knowledge-review

**Milestone** M0.6 · **Depends on** `desktop-application-shell`, `theme-system` · **Blocks** M2–M6 viewers.

## Purpose

Replace v1 Review/Wiki fixtures with desktop Develop, Memory, Review, Inbox, and Dashboard viewers. These
screens make agent work inspectable through scoped artifacts, diffs, run evidence, short-term context,
long-term facts, wiki provenance, and analytics instead of transcripts or undifferentiated metrics.

## Contract

Develop is a project-scoped diff/terminal workbench. Memory has four views: short-term prompt/context
budget, long-term facts and decay, artifacts, and wiki. Review exposes telemetry and run evidence.
Inbox and Dashboard remain global and show only action-relevant or aggregate data. Each viewer has
empty, loading, error, focus, keyboard, and Light/Dark fixture coverage.

## Acceptance

Run/project scope is unmissable, evidence links resolve through locators, data charts use the data
ramp, and the same artifact looks identical from every entry point.
