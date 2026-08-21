# locus-browse

**Milestone** M3.5 · **Depends on** `sandbox`, `artifacts` · **Blocks** `media-artifacts`, `agent-prs`

## Purpose

An agent that changed a UI can look at it. This is also what **makes screenshots free** — the agent's
app runs on `$LOCUS_PORT` in its container, the browser container reaches it over the project network,
and the screenshot lands as a run artifact on the board card.

The browser container is shared by a project's agents, which makes **isolation the whole problem**: two
agents driving one page is two agents fighting.

## Governed by

- PLAN.md §Browser testing — one container, one context per run
- PLAN.md §Agents need real tools — where each server runs
- PLAN.md §Artifacts — screenshots and recordings land automatically

## Contract

```
locus browse open URL              relative to the run's own app
locus browse click|fill|press SELECTOR [VALUE]
locus browse assert SELECTOR [--text S] [--visible] [--count N]
locus browse screenshot [SELECTOR]  → an image artifact
locus browse record start|stop      → a recording artifact
locus browse console|network        what the page logged, what it fetched
```

**Container per project, context per run.** Each run gets its own Playwright browser context — own
cookies, own storage, own pages, cheap to create — inside the one shared browser. Headless Chromium in
every agent image would be gratuitous; one per project on machinery `locus svc` already provides is not.

**The app is started by the container, not by the agent.** If the project declares a run script it
starts at container start, backgrounded, and `locus browse open` blocks until the readiness probe
passes. **An agent that forgets to start its app produces a screenshot of a connection error and reports
it as a UI bug.**

**`assert` exits non-zero and prints structured JSON**, so a workflow's `Verify` node can use it
directly. This is the difference between a browser an agent plays with and a browser that gates a merge.

**Auto-waiting, not sleeps.** Playwright waits for actionability by default. An agent writing `sleep 2`
is a flaky test being born, and the docs blob says so.

**The browser gets no egress by default.** It exists to reach the agent's app on the project network. A
test that genuinely needs a third-party origin is a project setting, named and audited — otherwise the
browser is a clean way around the egress policy the whole sandbox model rests on.

**`console` and `network` matter more than pixels.** A failing UI usually explains itself in a console
error, which is text; a screenshot of it costs tokens to say less. Same rule as the OCR path: text
first, pixels when appearance is the question.

## Acceptance

1. Two agents drive the shared browser at once **without seeing each other's cookies, storage or pages**.
2. `locus browse open` blocks until the readiness probe passes, and the app was started by the container.
3. `assert` exits non-zero on failure with structured JSON, and a `Verify` node can gate on it directly.
4. A screenshot lands as an artifact on the run and the board card, with no upload step by the agent.
5. The browser cannot reach an external origin by default; granting one is a named project setting and
   is audited.
6. `console` and `network` return text.
7. The docs blob advises against `sleep` and the auto-waiting behavior is real, not documented-only.
8. Killing the project's last run does not kill the browser container mid-use by another.

## Open

- Recording duration caps. PLAN.md says recordings are "capped by duration" without naming the cap, and
  it interacts with the 30-day media retention policy.
