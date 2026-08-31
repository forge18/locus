import {
  For,
  Match,
  Show,
  Switch,
  createEffect,
  createMemo,
  createSignal,
} from "solid-js";
import { dataProvider } from "../../data/provider";
import { InlineError } from "../../ui/InlineError";
import { Button } from "../../ui/Button";
import { FixtureNotice } from "../../ui/FixtureNotice";
import { Segmented } from "../../ui/Segmented";
import {
  QA_FOOTER,
  QA_LAST_RUN,
  fetchQaFindings,
  QA_SCHEDULE_OPTIONS,
  findingSummary,
  runQaCheck,
  sourceFindings,
  useQaFindings,
  useQaSources,
} from "../../data/qa";
import type { QaFinding, QaSchedule } from "../../data/qa";
import { failed, type Envelope } from "../../data/envelope";

import "./qa.css";

const schedules = new Map<string, QaSchedule>();

export interface QAViewProps {
  projectId?: string;
}

function QALive(props: { projectId?: string }) {
  const [findings, setFindings] = createSignal<Envelope<QaFinding[]>>({
    status: "loading",
  });
  const rows = createMemo(() => {
    const state = findings();
    return state.status === "ready" ? state.data : [];
  });
  const errorMessage = createMemo(() => {
    const state = findings();
    return state.status === "failed"
      ? `${state.error.command}: ${state.error.message}`
      : "";
  });

  let requestId = 0;
  createEffect(() => {
    const projectId = props.projectId;
    const request = ++requestId;
    setFindings({ status: "loading" });
    if (!projectId) {
      setFindings(
        failed(
          "qa_snapshot",
          "an active project is required to read QA findings",
        ),
      );
      return;
    }
    void fetchQaFindings(projectId)
      .then((result) => {
        if (request === requestId) setFindings(result);
      })
      .catch((cause) => {
        if (request === requestId) setFindings(failed("qa_snapshot", cause));
      });
  });

  return (
    <div class="qa-view" data-testid="qa" data-project={props.projectId}>
      <header class="qa-header">
        <div>
          <h1>QA</h1>
          <p>
            Persisted findings for {props.projectId ?? "the active project"}.
          </p>
        </div>
      </header>
      <main
        class="qa-groups"
        data-testid="qa-live-state"
        data-state={findings().status}
      >
        <Switch>
          <Match when={findings().status === "loading"}>
            <p class="qa-empty">Loading QA findings…</p>
          </Match>
          <Match when={findings().status === "failed"}>
            <InlineError
              cause={errorMessage()}
              next="Check the project connection and retry QA."
            />
          </Match>
          <Match when={findings().status === "empty"}>
            <p class="qa-empty">No persisted QA findings for this project.</p>
          </Match>
          <Match when={findings().status === "ready"}>
            <div class="qa-findings">
              <For each={rows()}>
                {(finding) => (
                  <article
                    class={`qa-finding qa-finding-${finding.severity}`}
                    data-testid={`qa-finding-${finding.id}`}
                  >
                    <span class="qa-severity">{finding.severity}</span>
                    <div class="qa-finding-copy">
                      <strong>{finding.title}</strong>
                      <span class="qa-location">
                        {finding.project} · {finding.location}
                      </span>
                      <p>{finding.explanation}</p>
                    </div>
                  </article>
                )}
              </For>
            </div>
          </Match>
        </Switch>
      </main>
    </div>
  );
}

export function QAView(props: QAViewProps) {
  if (dataProvider().kind === "live")
    return <QALive projectId={props.projectId} />;

  const project = () => props.projectId ?? "tapestry";
  const [schedule, setSchedule] = createSignal<QaSchedule>(
    schedules.get(project()) ?? "manual",
  );
  const [findings, setFindings] = createSignal(useQaFindings(project()));
  // The last recorded run comes from the fixture seam; nothing can update it
  // until a real QA command exists, so the signal is read-only for now.
  const [lastRun] = createSignal(QA_LAST_RUN);
  // Manual Refresh cannot start a real check until the backend registers a QA
  // command, so `refresh` records that honestly instead of reporting success.
  const [refreshUnsupported, setRefreshUnsupported] = createSignal(false);
  const [sentIds, setSentIds] = createSignal<string[]>(
    findings()
      .filter((finding) => finding.sentToInbox)
      .map((finding) => finding.id),
  );

  const setProjectSchedule = (value: string) => {
    const next = value as QaSchedule;
    schedules.set(project(), next);
    setSchedule(next);
  };

  const refresh = () => {
    // All sources use the same run entry point for manual and scheduled checks.
    // Every attempt comes back `unsupported` (no QA command exists yet), so the
    // checks did not run: no fresh findings, and the last-run stamp stays as it
    // was. The view says so rather than claiming a successful new run.
    const attempts = useQaSources().map((source) =>
      runQaCheck(project(), source.id),
    );
    setRefreshUnsupported(
      attempts.every((attempt) => attempt.status === "unsupported"),
    );
  };

  const send = (id: string) => {
    setSentIds((current) =>
      current.includes(id) ? current : [...current, id],
    );
    setFindings((current) =>
      current.map((finding) =>
        finding.id === id ? { ...finding, sentToInbox: true } : finding,
      ),
    );
  };

  return (
    <div class="qa-view" data-testid="qa" data-project={project()}>
      <FixtureNotice surface="QA" command='invoke("qa_checks")' />
      <header class="qa-header">
        <div>
          <h1>QA</h1>
          <p>
            Tests, linters, LSP diagnostics and agent reviews for #{project()} ·
            last run {lastRun()}
          </p>
        </div>
        <Segmented
          options={QA_SCHEDULE_OPTIONS.map((option) => ({
            value: option.value,
            label: option.label,
          }))}
          value={schedule()}
          onChange={setProjectSchedule}
          label="QA schedule"
        />
        <Button onClick={refresh} data-testid="qa-refresh">
          Refresh
        </Button>
      </header>

      <Show when={refreshUnsupported()}>
        <p class="qa-empty" role="status" data-testid="qa-refresh-unsupported">
          Refresh can't run checks — the desktop backend registers no QA command
          yet, so findings stay as of the last recorded run.
        </p>
      </Show>

      <main class="qa-groups">
        <For each={useQaSources()}>
          {(source) => {
            const rows = createMemo(() =>
              sourceFindings(findings(), source.id),
            );
            const summary = createMemo(() => findingSummary(rows()));
            return (
              <section class="qa-group" data-testid={`qa-group-${source.id}`}>
                <header class="qa-group-header">
                  <div>
                    <h2>{source.name}</h2>
                    <span>{source.attribution}</span>
                  </div>
                  <span
                    class="qa-summary"
                    data-testid={`qa-summary-${source.id}`}
                  >
                    {summary().fail} fail · {summary().warn} warn
                  </span>
                </header>
                <Show
                  when={rows().length > 0}
                  fallback={
                    <p class="qa-empty">No findings from this source.</p>
                  }
                >
                  <div class="qa-findings">
                    <For each={rows()}>
                      {(finding) => (
                        <article
                          class={`qa-finding qa-finding-${finding.severity}`}
                          data-testid={`qa-finding-${finding.id}`}
                        >
                          <span class="qa-severity">{finding.severity}</span>
                          <div class="qa-finding-copy">
                            <strong>{finding.title}</strong>
                            <span class="qa-location">
                              {finding.project} · {finding.location}
                            </span>
                            <p>{finding.explanation}</p>
                          </div>
                          <Button
                            variant="ghost"
                            onClick={() => send(finding.id)}
                            disabled={sentIds().includes(finding.id)}
                          >
                            {sentIds().includes(finding.id)
                              ? "Sent to Inbox"
                              : "Send to Inbox"}
                          </Button>
                        </article>
                      )}
                    </For>
                  </div>
                </Show>
              </section>
            );
          }}
        </For>
      </main>
      <footer class="qa-footer">{QA_FOOTER}</footer>
    </div>
  );
}

export default QAView;
