/**
 * The explicit demo/test provider. It answers only commands whose fixtures are
 * registered here — never imported by the production bootstrap and never selected
 * by a Tauri runtime. See .specs/desktop-data-integration/contract.md.
 */
import * as agentDefs from "../../fixtures/agent-defs";
import * as analytics from "../../fixtures/analytics";
import * as artifacts from "../../fixtures/artifacts";
import * as board from "../../fixtures/board";
import * as bots from "./fixtures/bots";
import * as core from "../../fixtures/core";
import * as extensions from "../../fixtures/extensions";
import * as guardrails from "../../fixtures/settings-guardrails";
import * as interact from "./fixtures/interact";
import * as harnesses from "../../fixtures/generated/harnesses";
import * as knowledge from "../../fixtures/knowledge";
import * as mail from "../../fixtures/mail";
import * as plan from "../../fixtures/plan";
import * as qa from "../../fixtures/qa";
import * as settings from "../../fixtures/settings";
import * as telemetry from "../../fixtures/telemetry";
import * as workflow from "../../fixtures/workflow";
import * as workflowEvents from "../../fixtures/workflow-events";
import type { Envelope } from "../envelope";
import { failed, ready, readyOne } from "../envelope";
import type { DataProvider } from "../provider";

type FixtureValue = (args?: Record<string, unknown>) => unknown;

const FIXTURES: Record<string, FixtureValue> = {
  projects_list: () => core.PROJECTS,
  repos_list: (args) => {
    const projectId = args?.projectId;
    return typeof projectId === "string"
      ? core.REPOS.filter((repo) => repo.projectId === projectId)
      : core.REPOS;
  },
  local_remotes_list: () => core.LOCAL_REMOTES,
  plans_list: () => plan.PLANS,
  plan_conversation: () => plan.CONVERSATION,
  plan_live_line: () => plan.LIVE_LINE,
  workflow_definitions: () => [
    {
      id: workflow.WORKFLOW.id,
      name: workflow.WORKFLOW.name,
      version: workflow.WORKFLOW.version,
    },
  ],
  board_tasks: () => board.TASKS,
  board_dependencies: () => board.DEPENDENCIES,
  interact_sessions_list: (args) => {
    const projectId = args?.projectId;
    return typeof projectId === "string"
      ? interact.SESSIONS.filter((session) => session.projectId === projectId)
      : interact.SESSIONS;
  },
  bots_list: (args) => {
    const projectId = args?.projectId;
    return typeof projectId === "string"
      ? bots.BOTS.filter((bot) => bot.projectId === projectId)
      : bots.BOTS;
  },
  bot_routines: (args) =>
    bots.ROUTINES.filter((routine) => routine.botId === args?.botId),
  bot_routine_executions: () => [],
  task_evidence: () => board.EVIDENCE,
  memory_facts: () => knowledge.LONG_TERM_FACTS,
  analytics_at_a_glance: () => analytics.AT_A_GLANCE_METRICS,
  analytics_stats: () => analytics.ANALYTICS_STATS,
  analytics_breakdown: () => analytics.ANALYTICS_BREAKDOWN,
  analytics_task_outcomes: () => analytics.TASK_OUTCOMES,
  analytics_workflow_timings: () => analytics.WORKFLOW_TIMINGS,
  analytics_retrieval_tiers: () => analytics.RETRIEVAL_TIERS,
  analytics_extension_usage: () => analytics.EXTENSION_USAGE,
  analytics_extension_kinds: () => analytics.EXTENSION_KINDS,
  analytics_breakdown_dimensions: () => analytics.BREAKDOWN_DIMENSIONS,
  analytics_telemetry_facets: () => analytics.TELEMETRY_FACETS,
  analytics_telemetry_actions: () => analytics.TELEMETRY_ACTIONS,
  analytics_telemetry_sessions: () => analytics.TELEMETRY_SESSIONS,
  analytics_telemetry_verbs: () => analytics.TELEMETRY_VERBS,
  telemetry_metrics: () => telemetry.METRICS,
  sessions_over_time: () => telemetry.SPARKLINE,
  telemetry_filters: () => telemetry.FILTER_CHIPS,
  telemetry_facets: () => telemetry.FACET_GROUPS,
  telemetry_actions: () => telemetry.ACTION_ROWS,
  telemetry_tools: () => telemetry.TOOL_ROWS,
  telemetry_sessions: () => telemetry.SESSION_ROWS,
  telemetry_sessions_page: (args) => {
    const offset = typeof args?.offset === "number" ? args.offset : 0;
    const limit = typeof args?.limit === "number" ? args.limit : 100;
    return telemetry.ALL_SESSION_ROWS.slice(offset, offset + limit);
  },
  telemetry_facets_flat: () => telemetry.FACETS,
  telemetry_verb_counts: () => telemetry.VERB_COUNTS,
  telemetry_spend: () => telemetry.SPEND,
  qa_snapshot: (args) =>
    typeof args?.projectId === "string"
      ? qa.QA_FINDINGS.filter((finding) => finding.project === args.projectId)
      : qa.QA_FINDINGS,
  qa_sources: () => qa.QA_CHECK_SOURCES,
  artifacts_list: () => artifacts.ARTIFACTS,
  external_work_item_providers: () => [
    {
      pluginId: "github",
      label: "GitHub",
      host: "github.com",
      project: "forge18/locus",
      resolutionSupported: true,
      syncSupported: true,
      syncIntervalSeconds: 60,
    },
  ],
  artifact_comments: (args) =>
    artifacts.ARTIFACT_COMMENTS.filter(
      (comment) => comment.artifactId === args?.artifactId,
    ),
  agent_defs_list: () => agentDefs.AGENT_DEFS,
  agent_frontmatter: () => agentDefs.FRONTMATTER,
  agent_prose: () => agentDefs.PROSE,
  extension_inventory: () => extensions.TYPE_CARDS,
  extension_counts: () => harnesses.EXTENSION_COUNTS,
  recently_edited: () => extensions.RECENTLY_EDITED,
  harness_registry_list: () => harnesses.HARNESSES,
  extension_types: () => harnesses.EXTENSION_TYPES,
  settings_guardrails: () => guardrails.GUARDRAIL_SECTIONS,
  mail_threads: () => mail.MAIL_THREADS,
  mail_messages: (args) =>
    mail.MAIL_MESSAGES.filter((message) => message.threadId === args?.threadId),
  mail_participants: () => mail.MAIL_PARTICIPANTS,
  workflow_node_vocabulary: () => workflow.PALETTE,
  workflow_presets: () => workflow.PRESETS,
  condition_expression: () => workflow.EXPRESSION,
  condition_operands: () => workflow.OPERANDS,
  workflow_guardrails: () => workflow.GUARDRAILS,
  guardrail_trips: () => workflow.GUARDRAIL_TRIPS,
  workflow_events: () => workflowEvents.WORKFLOW_EVENTS,
};

const SINGLE_FIXTURES: Record<string, FixtureValue> = {
  artifact: (args) =>
    artifacts.ARTIFACTS.find((artifact) => artifact.id === args?.id) ?? null,
  artifact_diff: () => artifacts.UNIFIED_DIFF,
  artifact_default_id: () => artifacts.SELECTED_ARTIFACT_ID,
  artifact_kinds: () => ({
    review: artifacts.REVIEW_KINDS,
    reference: artifacts.REFERENCE_KINDS,
  }),
  memory_short_term: () => knowledge.RESIDENT_LAYERS,
  memory_compacted_artifacts: () => knowledge.COMPACTED_CONTEXT,
  workflow_def: () => workflow.WORKFLOW,
  workflow_graph: () => ({
    nodes: workflow.CANVAS_NODES,
    edges: workflow.CANVAS_EDGES,
    loop: workflow.LOOP_GROUP,
    markers: workflow.ARROW_MARKERS,
    events: workflowEvents.WORKFLOW_EVENTS,
  }),
  harness_registry_summary: () => ({
    harnesses: harnesses.HARNESS_COUNT,
    entries: harnesses.ENTRY_COUNT,
    downgrades: harnesses.DOWNGRADE_COUNT,
  }),
  telemetry_sessions_count: () => telemetry.ALL_SESSION_ROWS.length,
  settings_model_tiers: () => settings.MODEL_TIERS,
  settings_tier_fallback: () => settings.TIER_FALLBACK,
  harness_tier_grid: () =>
    harnesses.HARNESSES.map((harness) => ({
      name: harness.name,
      models: harness.canEnumerateModels
        ? (settings.DISCOVERED_MODEL_IDS[harness.name] ?? [])
        : null,
      tiers: settings.TIERS.map((tier) => ({
        harness: harness.name,
        tier,
        model:
          settings.MODEL_TIERS.find(
            (setting) =>
              setting.harness === harness.name && setting.tier === tier,
          )?.model ?? null,
      })),
    })),
  agent_default_id: () => agentDefs.SELECTED_DEF,
  agent_materialization: () => ({
    harnesses: harnesses.HARNESS_COUNT,
    downgraded:
      harnesses.EXTENSION_COUNTS.find((count) => count.type === "agents")
        ?.downgraded ?? 0,
  }),
  plan_default_id: () => plan.SELECTED_PLAN_ID,
  plan_scope_decision: () => plan.SCOPE_DECISION,
  plan_outputs: () => plan.DRAFT_OUTPUTS,
  plan_recommendation: () => plan.RECOMMENDATION,
  agent_def: () => ({
    name: agentDefs.SELECTED_DEF,
    version: agentDefs.NEXT_VERSION,
    frontmatter: {},
    body: agentDefs.PROSE.join("\n"),
    warnings: [],
  }),
};

function demoEnvelope<T>(
  command: string,
  args: Record<string, unknown> | undefined,
): Envelope<T[]> {
  const fixture = FIXTURES[command];
  if (!fixture) {
    return failed(command, `demo provider has no fixture for ${command}`);
  }
  const value = fixture(args);
  if (!Array.isArray(value)) {
    return failed(command, `demo fixture for ${command} is not a list`);
  }
  return ready(value as T[]);
}

export const demoProvider: DataProvider = {
  kind: "demo",
  read<T>(command: string, args?: Record<string, unknown>) {
    const fixture = FIXTURES[command] ?? SINGLE_FIXTURES[command];
    return fixture?.(args) as T | undefined;
  },
  async query<T>(command: string, args?: Record<string, unknown>) {
    return demoEnvelope<T>(command, args);
  },
  async queryOne<T>(command: string, args?: Record<string, unknown>) {
    const fixture = SINGLE_FIXTURES[command];
    if (!fixture) {
      return failed(
        command,
        `demo provider has no single-value fixture for ${command}`,
      );
    }
    return readyOne(fixture(args) as T | null);
  },
};
