import type { AgentEvent } from "../types/event";

export type AgentPermissionPosture = "bypass" | "gated";
export type AgentGateMode = "manual" | "auto";
export type AgentThinkingDisplay = "summary" | "full" | "hidden";
export type AgentToolDisplay = "expanded" | "collapsed" | "hidden";
export type AgentPanelStatus = "working" | "waiting" | "idle" | "done";
export type AgentFieldValue = string | number | boolean;

export interface AgentPaneViewModel {
  sessionId: string;
  runId: string;
  projectId: string;
  taskId?: string;
  workflowDefId?: string;
  permissionPosture: AgentPermissionPosture;
  liveStatus: AgentPanelStatus;
  context: { used: number; total: number };
  activePlan?: AgentPanePlan;
}

export interface AgentPaneSession {
  project: string;
  task?: string;
  workflow?: string;
  agent: string;
  model: string;
  harness: string;
  effort: string;
  name: string;
  context: { used: number; total: number };
  harnessOptions?: string[];
  modelOptions?: string[];
  effortOptions?: string[];
  cost?: string;
  permissionPosture: AgentPermissionPosture;
  status?: AgentPanelStatus;
}

export interface AgentPaneFinding {
  id: string;
  title: string;
  summary: string;
  source: string;
  provenance: "seed" | "this_run" | "session_close";
  reviewed?: boolean;
}

export interface AgentPaneCitation {
  id: string;
  label: string;
  source: string;
  summary?: string;
}

export interface AgentPanePlanStep {
  id: string;
  title: string;
  status:
    | "pending"
    | "in_progress"
    | "done"
    | "completed"
    | "failed"
    | "cancelled";
  outcome?: string;
}

export interface AgentPanePlan {
  id: string;
  title: string;
  steps: AgentPanePlanStep[];
  markdown?: string;
  file?: string;
  outcome?: string;
}

export interface AgentPaneBlocker {
  id: string;
  kind: "gate" | "elicitation";
  title: string;
  detail: string;
  event?: AgentEvent;
}

export interface AgentPaneElicitationField {
  id: string;
  label: string;
  type: "text" | "string" | "number" | "integer" | "boolean" | "enum";
  required?: boolean;
  defaultValue?: AgentFieldValue;
  options?: string[];
  format?: "email" | "uri";
  pattern?: string;
  minLength?: number;
  minimum?: number;
  maximum?: number;
  suggestions?: string[];
}

export interface AgentPaneElicitation {
  id: string;
  title: string;
  detail: string;
  mode?: "form" | "url";
  fields: AgentPaneElicitationField[];
  history?: Record<string, string>[];
}

export interface AgentPaneCheckpoint {
  id: string;
  label: string;
  file: string;
  state: "available" | "restored";
}

export interface AgentPaneProps {
  runId: string;
  session?: AgentPaneSession;
  events?: AgentEvent[];
  findings?: AgentPaneFinding[];
  plan?: AgentPanePlan;
  planUpdates?: AgentPanePlan[];
  viewModel?: AgentPaneViewModel;
  blockers?: AgentPaneBlocker[];
  elicitation?: AgentPaneElicitation;
  checkpoints?: AgentPaneCheckpoint[];
  permissionPosture?: AgentPermissionPosture;
  gateMode?: AgentGateMode;
  onGateModeChange?: (mode: AgentGateMode) => void;
  thinkingDisplay?: AgentThinkingDisplay;
  onThinkingDisplayChange?: (display: AgentThinkingDisplay) => void;
  toolCallsDisplay?: AgentToolDisplay;
  onToolCallsDisplayChange?: (display: AgentToolDisplay) => void;
  showCost?: boolean;
  onCostVisibilityChange?: (visible: boolean) => void;
  /** Disable the live channel for deterministic fixture previews. */
  live?: boolean;
  /** Use an owner-controlled research toggle when the pane is embedded. */
  researchOpen?: boolean;
  onResearchToggle?: (open: boolean) => void;
  showResearchControl?: boolean;
  mentionSuggestions?: string[];
  showResearchPane?: boolean;
  onSend?: (prompt: string) => void;
  onQueue?: (prompt: string) => void;
  onStop?: () => void;
  onNewSession?: () => void;
  onCompact?: () => void;
  onClearContext?: () => void;
  onViewContext?: () => void;
  onSessionRename?: (name: string) => void;
  onHarnessChange?: (harness: string) => void;
  onModelChange?: (model: string) => void;
  onEffortChange?: (effort: string) => void;
  onApprovePermission?: (event: AgentEvent) => void;
  onDeclinePermission?: (event: AgentEvent) => void;
  onApproveRemainingTurn?: (event: AgentEvent) => void;
  onResubmit?: (event: AgentEvent, prompt: string) => void;
  onCopyPrompt?: (prompt: string) => void;
  onOpenFile?: (path: string) => void;
  onPinCitation?: (citation: AgentPaneCitation) => void;
  onReviewFinding?: (finding: AgentPaneFinding) => void;
  onPromoteFinding?: (finding: AgentPaneFinding) => void;
  onAcceptElicitation?: (
    elicitation: AgentPaneElicitation,
    values: Record<string, AgentFieldValue>,
  ) => void;
  onDeclineElicitation?: (elicitation: AgentPaneElicitation) => void;
  onCancelElicitation?: (elicitation: AgentPaneElicitation) => void;
  onRestoreCheckpoint?: (checkpoint: AgentPaneCheckpoint) => void;
  onUndoCheckpoint?: (checkpoint: AgentPaneCheckpoint) => void;
}
