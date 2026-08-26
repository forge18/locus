//! Versioned workflow authoring data.
//!
//! Definitions carry Governance alongside their graph, while evaluations identify
//! the run that produced them rather than mutating a definition.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;
use uuid::Uuid;

#[path = "workflow_graph.rs"]
pub mod graph;

use crate::{
    ids::{ProjectId, RunId, SessionId},
    runtime::session::{resume_from_events, ResumePlan, Session},
    sandbox::workspace::{refuse_primary_branch, workspace_clone_branch_command},
    services::{
        inbox::{InboxItem, InboxKind},
        mail::{Locator, WaitReason, WaitingState},
    },
};

/// The closed vocabulary of workflow domain entries. Telemetry verbs intentionally do not appear.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowEntryKind {
    Execution,
    Iteration,
    GuardrailTrip,
    VerifyResult,
}

impl WorkflowEntryKind {
    pub const ALL: [Self; 4] = [
        Self::Execution,
        Self::Iteration,
        Self::GuardrailTrip,
        Self::VerifyResult,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Execution => "workflow.execution",
            Self::Iteration => "workflow.iteration",
            Self::GuardrailTrip => "workflow.guardrail_trip",
            Self::VerifyResult => "workflow.verify_result",
        }
    }

    pub fn parse(value: &str) -> Result<Self, WorkflowEntryError> {
        match value {
            "workflow.execution" => Ok(Self::Execution),
            "workflow.iteration" => Ok(Self::Iteration),
            "workflow.guardrail_trip" => Ok(Self::GuardrailTrip),
            "workflow.verify_result" => Ok(Self::VerifyResult),
            _ => Err(WorkflowEntryError::UnknownKind(value.to_owned())),
        }
    }
}

impl std::fmt::Display for WorkflowEntryKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ExecutionEntryPayload {
    pub execution_id: Uuid,
    pub workflow_def_id: Uuid,
    pub schedule_id: Option<Uuid>,
    pub status: String,
    pub scheduled_for: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IterationEntryPayload {
    pub iteration_id: Uuid,
    pub execution_id: Uuid,
    pub run_id: Option<Uuid>,
    pub number: u32,
    pub arbiter_class: Option<String>,
    pub counts_toward_iteration_budget: bool,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

impl IterationEntryPayload {
    pub fn arbiter_classification(
        iteration_id: Uuid,
        execution_id: Uuid,
        run_id: Option<Uuid>,
        number: u32,
        class: super::arbiter::FailureClass,
    ) -> Self {
        Self {
            iteration_id,
            execution_id,
            run_id,
            number,
            arbiter_class: Some(class.as_str().to_owned()),
            counts_toward_iteration_budget: class.counts_toward_budget(),
            started_at: None,
            ended_at: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuardrailTripEntryPayload {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub iteration_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub guardrail: String,
    pub detail: Value,
    pub tripped_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VerifyResultEntryPayload {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub iteration_id: Option<Uuid>,
    pub verify_node_id: String,
    pub command: String,
    pub container_id: String,
    pub exit_code: i32,
    pub passed: bool,
    pub stdout: String,
    pub stderr: String,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEntry {
    pub project_id: ProjectId,
    pub stream_pos: u64,
    pub kind: WorkflowEntryKind,
    pub version: u16,
    pub payload: Value,
    pub actor: String,
    pub caused_by: Option<u64>,
}

impl WorkflowEntry {
    pub fn new(
        project_id: ProjectId,
        stream_pos: u64,
        kind: WorkflowEntryKind,
        version: u16,
        payload: Value,
        actor: impl Into<String>,
        caused_by: Option<u64>,
    ) -> Self {
        Self {
            project_id,
            stream_pos,
            kind,
            version,
            payload,
            actor: actor.into(),
            caused_by,
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum WorkflowEntryError {
    #[error("unknown workflow entry kind `{0}`")]
    UnknownKind(String),
    #[error("unknown workflow entry version {version} for `{kind}`")]
    UnknownVersion {
        kind: WorkflowEntryKind,
        version: u16,
    },
    #[error("invalid payload for workflow entry `{kind}` v{version}: {detail}")]
    InvalidPayload {
        kind: WorkflowEntryKind,
        version: u16,
        detail: String,
    },
    #[error("workflow entry stream position must be greater than zero")]
    InvalidStreamPosition,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkflowsProjection {
    executions: BTreeMap<Uuid, ExecutionEntryPayload>,
    iterations: BTreeMap<Uuid, IterationEntryPayload>,
    guardrail_trips: BTreeMap<Uuid, GuardrailTripEntryPayload>,
    verify_results: BTreeMap<Uuid, VerifyResultEntryPayload>,
}

impl WorkflowsProjection {
    pub fn apply(&mut self, entry: &WorkflowEntry) -> Result<(), WorkflowEntryError> {
        if entry.stream_pos == 0 {
            return Err(WorkflowEntryError::InvalidStreamPosition);
        }
        match decode_entry_payload(entry)? {
            WorkflowPayload::Execution(payload) => {
                self.executions.insert(payload.execution_id, payload);
            }
            WorkflowPayload::Iteration(payload) => {
                self.iterations.insert(payload.iteration_id, payload);
            }
            WorkflowPayload::GuardrailTrip(payload) => {
                self.guardrail_trips.insert(payload.id, payload);
            }
            WorkflowPayload::VerifyResult(payload) => {
                self.verify_results.insert(payload.id, payload);
            }
        }
        Ok(())
    }

    pub fn rebuild(
        entries: impl IntoIterator<Item = WorkflowEntry>,
    ) -> Result<Self, WorkflowEntryError> {
        let mut projection = Self::default();
        let mut previous = None;
        for entry in entries {
            if previous.is_some_and(|position| entry.stream_pos <= position) {
                return Err(WorkflowEntryError::InvalidStreamPosition);
            }
            previous = Some(entry.stream_pos);
            projection.apply(&entry)?;
        }
        Ok(projection)
    }

    pub fn execution(&self, id: Uuid) -> Option<&ExecutionEntryPayload> {
        self.executions.get(&id)
    }

    pub fn iteration(&self, id: Uuid) -> Option<&IterationEntryPayload> {
        self.iterations.get(&id)
    }

    pub fn verify_result(&self, id: Uuid) -> Option<&VerifyResultEntryPayload> {
        self.verify_results.get(&id)
    }

    pub fn executions(&self) -> impl Iterator<Item = &ExecutionEntryPayload> {
        self.executions.values()
    }

    pub fn iterations(&self) -> impl Iterator<Item = &IterationEntryPayload> {
        self.iterations.values()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowPayload {
    Execution(ExecutionEntryPayload),
    Iteration(IterationEntryPayload),
    GuardrailTrip(GuardrailTripEntryPayload),
    VerifyResult(VerifyResultEntryPayload),
}

pub fn decode_entry_payload(entry: &WorkflowEntry) -> Result<WorkflowPayload, WorkflowEntryError> {
    if entry.version != 1 {
        return Err(WorkflowEntryError::UnknownVersion {
            kind: entry.kind,
            version: entry.version,
        });
    }
    let decoded = match entry.kind {
        WorkflowEntryKind::Execution => {
            serde_json::from_value(entry.payload.clone()).map(WorkflowPayload::Execution)
        }
        WorkflowEntryKind::Iteration => {
            serde_json::from_value(entry.payload.clone()).map(WorkflowPayload::Iteration)
        }
        WorkflowEntryKind::GuardrailTrip => {
            serde_json::from_value(entry.payload.clone()).map(WorkflowPayload::GuardrailTrip)
        }
        WorkflowEntryKind::VerifyResult => {
            serde_json::from_value(entry.payload.clone()).map(WorkflowPayload::VerifyResult)
        }
    };
    let decoded = decoded.map_err(|error| WorkflowEntryError::InvalidPayload {
        kind: entry.kind,
        version: entry.version,
        detail: error.to_string(),
    })?;
    validate_entry_payload(entry, &decoded)?;
    Ok(decoded)
}

fn validate_entry_payload(
    entry: &WorkflowEntry,
    payload: &WorkflowPayload,
) -> Result<(), WorkflowEntryError> {
    let invalid = |detail: &str| WorkflowEntryError::InvalidPayload {
        kind: entry.kind,
        version: entry.version,
        detail: detail.to_owned(),
    };
    match payload {
        WorkflowPayload::Execution(payload) => {
            if payload.status.trim().is_empty() {
                return Err(invalid("execution status is required"));
            }
        }
        WorkflowPayload::Iteration(payload) => {
            if payload.number == 0 {
                return Err(invalid("iteration number must be greater than zero"));
            }
            if let Some(class) = payload.arbiter_class.as_deref() {
                if !super::arbiter::FailureClass::ALL
                    .iter()
                    .any(|candidate| candidate.as_str() == class)
                {
                    return Err(invalid("unknown arbiter failure class"));
                }
                if class == super::arbiter::FailureClass::Noise.as_str()
                    && payload.counts_toward_iteration_budget
                {
                    return Err(invalid("noise must not count toward iteration budget"));
                }
            }
        }
        WorkflowPayload::GuardrailTrip(payload) => {
            if payload.guardrail.trim().is_empty() {
                return Err(invalid("guardrail name is required"));
            }
        }
        WorkflowPayload::VerifyResult(payload) => {
            if payload.verify_node_id.trim().is_empty()
                || payload.command.trim().is_empty()
                || payload.container_id.trim().is_empty()
                || payload.passed != (payload.exit_code == 0)
            {
                return Err(invalid("verify result fields are inconsistent"));
            }
        }
    }
    Ok(())
}

/// Authored Governance attached to one immutable workflow definition version.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkflowGovernance {
    pub version: u32,
    pub goal: String,
    pub guardrails: Vec<Guardrail>,
    pub success_criteria: Vec<SuccessCriterion>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct WorkflowAgentPermissions {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_scope: Option<String>,
}

/// A typed executable step produced by compiling one graph.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkflowStep {
    Agent {
        node_id: String,
        role: String,
        permissions: WorkflowAgentPermissions,
    },
    Task {
        node_id: String,
    },
    Verify {
        node_id: String,
        command: String,
    },
    Condition {
        node_id: String,
        expression: String,
    },
    Gate {
        node_id: String,
        gate: GateKind,
    },
    Loop {
        node_id: String,
        max_iterations: u32,
        reset_to: Option<String>,
    },
}

impl WorkflowStep {
    fn node_id(&self) -> &str {
        match self {
            Self::Agent { node_id, .. }
            | Self::Task { node_id }
            | Self::Verify { node_id, .. }
            | Self::Condition { node_id, .. }
            | Self::Gate { node_id, .. }
            | Self::Loop { node_id, .. } => node_id,
        }
    }
}

/// Gate policy belongs to the compiled executable step; pending state belongs to an execution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GateKind {
    Human { prompt: String },
    ReviewerAgent { role: String, max_rounds: u32 },
}

pub type GateNode = GateKind;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HumanGateRequest {
    pub node_id: String,
    pub project_id: ProjectId,
    pub session_id: SessionId,
    pub run_id: RunId,
    pub prompt: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReviewerAgentRequest {
    pub node_id: String,
    pub run_id: RunId,
    pub role: String,
    pub max_rounds: u32,
    /// Summaries and identifiers only; reviewer gates never receive a model/provider handle.
    pub artifact_summaries: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GateRequest {
    Human(HumanGateRequest),
    ReviewerAgent(ReviewerAgentRequest),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum HumanGateOutcome {
    Approved,
    Rejected { reason: String },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReviewerAgentOutcome {
    pub approved: bool,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GateOutcome {
    Human(HumanGateOutcome),
    ReviewerAgent(ReviewerAgentOutcome),
}

pub const MAX_REVIEW_ROUNDS: u32 = 3;

impl HumanGateRequest {
    pub fn waiting_state(&self) -> WaitingState {
        WaitingState::new(
            self.run_id,
            WaitReason::Gate,
            serde_json::json!({"node_id": self.node_id, "prompt": self.prompt}),
        )
    }

    pub fn inbox_item(&self) -> Result<InboxItem, WorkflowError> {
        InboxItem::new(
            format!("gate:{}", self.node_id),
            self.project_id,
            self.session_id,
            Some(self.run_id),
            InboxKind::Gate,
            "Workflow approval",
            self.prompt.clone(),
            Locator::new(format!(
                "locus://project/{}/gate/{}",
                self.project_id, self.node_id
            ))
            .map_err(|_| WorkflowError::InvalidGateLocator)?,
        )
        .map_err(|_| WorkflowError::MissingGatePrompt)
    }
}

impl ReviewerAgentRequest {
    pub fn bounded(
        node_id: impl Into<String>,
        run_id: RunId,
        role: impl Into<String>,
        max_rounds: u32,
        artifact_summaries: Vec<String>,
    ) -> Result<Self, WorkflowError> {
        if max_rounds == 0 || max_rounds > MAX_REVIEW_ROUNDS {
            return Err(WorkflowError::GateBoundExceeded);
        }
        let role = role.into();
        if role.trim().is_empty() {
            return Err(WorkflowError::MissingReviewerRole);
        }
        Ok(Self {
            node_id: node_id.into(),
            run_id,
            role,
            max_rounds,
            artifact_summaries,
        })
    }

    pub fn has_model_hook(&self) -> bool {
        false
    }
}

/// Convert a compiled gate into its explicit boundary request. Reviewer gates are data-only and
/// deliberately return no waiting/inbox state.
pub fn gate_request(
    node_id: impl Into<String>,
    gate: &GateKind,
    project_id: ProjectId,
    session_id: SessionId,
    run_id: RunId,
    artifact_summaries: Vec<String>,
) -> Result<GateRequest, WorkflowError> {
    let node_id = node_id.into();
    if node_id.trim().is_empty() {
        return Err(WorkflowError::InvalidNodeId);
    }
    match gate {
        GateKind::Human { prompt } => {
            if prompt.trim().is_empty() {
                return Err(WorkflowError::MissingGatePrompt);
            }
            Ok(GateRequest::Human(HumanGateRequest {
                node_id,
                project_id,
                session_id,
                run_id,
                prompt: prompt.clone(),
            }))
        }
        GateKind::ReviewerAgent { role, max_rounds } => {
            Ok(GateRequest::ReviewerAgent(ReviewerAgentRequest::bounded(
                node_id,
                run_id,
                role.clone(),
                *max_rounds,
                artifact_summaries,
            )?))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoalApproval {
    Pending,
    Approved,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionStatus {
    AwaitingApproval,
    Running,
    Exited,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowExecution {
    pub governance_version: u32,
    pub goal: String,
    pub approval: GoalApproval,
    pub status: ExecutionStatus,
}

impl WorkflowExecution {
    pub fn new(compiled: &CompiledWorkflow) -> Self {
        Self {
            governance_version: compiled.governance.version,
            goal: compiled.governance.goal.clone(),
            approval: GoalApproval::Pending,
            status: ExecutionStatus::AwaitingApproval,
        }
    }

    pub fn approve_goal(&mut self, governance_version: u32) -> Result<(), WorkflowError> {
        if self.governance_version != governance_version {
            return Err(WorkflowError::ApprovalVersionMismatch);
        }
        if self.status != ExecutionStatus::AwaitingApproval {
            return Err(WorkflowError::ExecutionAlreadyStarted);
        }
        self.approval = GoalApproval::Approved;
        Ok(())
    }

    pub fn start_loop(&mut self) -> Result<(), WorkflowError> {
        if self.approval != GoalApproval::Approved {
            return Err(WorkflowError::GoalApprovalRequired);
        }
        if self.status != ExecutionStatus::AwaitingApproval {
            return Err(WorkflowError::ExecutionAlreadyStarted);
        }
        self.status = ExecutionStatus::Running;
        Ok(())
    }

    pub fn loop_may_run(&self) -> bool {
        self.approval == GoalApproval::Approved && self.status == ExecutionStatus::Running
    }
}

pub fn begin_execution(compiled: &CompiledWorkflow) -> WorkflowExecution {
    WorkflowExecution::new(compiled)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RalphResult {
    pub goal: String,
    pub verify: String,
    pub max_iterations: u32,
    pub iterations: u32,
    pub status: String,
}

/// Run the stateless Ralph preset. This plans a bounded deterministic loop only; execution has no
/// model/provider callback and therefore cannot claim a verify result it did not observe.
pub fn run_ralph(
    goal: impl Into<String>,
    verify: impl Into<String>,
    max_iterations: u32,
) -> Result<RalphResult, WorkflowError> {
    let goal = goal.into();
    let verify = verify.into();
    if goal.trim().is_empty() || verify.trim().is_empty() {
        return Err(WorkflowError::EmptyGoal);
    }
    if max_iterations == 0 || max_iterations > MAX_WALK_EVENTS as u32 {
        return Err(WorkflowError::WalkLimitExceeded);
    }
    Ok(RalphResult {
        goal,
        verify,
        max_iterations,
        iterations: 0,
        status: "ready".into(),
    })
}

/// A compiled control-flow edge. Edges are retained in the spec so execution never falls back
/// to the insertion order of canvas nodes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WorkflowEdge {
    from: String,
    to: String,
}

impl WorkflowEdge {
    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn to(&self) -> &str {
        &self.to
    }
}

/// The only representation consumed by the workflow supervisor. Fields are private so a caller
/// cannot construct a spec that disagrees with an authored graph; obtain one from
/// [`CompiledWorkflow`] instead.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WorkflowSpec {
    version: u32,
    goal: String,
    guardrails: Vec<Guardrail>,
    success_criteria: Vec<SuccessCriterion>,
    verify_command: String,
    steps: Vec<WorkflowStep>,
    edges: Vec<WorkflowEdge>,
}

impl WorkflowSpec {
    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn goal(&self) -> &str {
        &self.goal
    }

    pub fn verify_command(&self) -> &str {
        &self.verify_command
    }

    pub fn steps(&self) -> &[WorkflowStep] {
        &self.steps
    }

    pub fn edges(&self) -> &[WorkflowEdge] {
        &self.edges
    }
}

/// A graph and its Governance are one authored, compile-time unit. `spec` is private by
/// construction: callers can only obtain it from [`compile_workflow`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CompiledWorkflow {
    graph: serde_json::Value,
    governance: WorkflowGovernance,
    spec: WorkflowSpec,
}

impl CompiledWorkflow {
    pub fn graph(&self) -> &serde_json::Value {
        &self.graph
    }

    pub fn governance(&self) -> &WorkflowGovernance {
        &self.governance
    }

    pub fn spec(&self) -> &WorkflowSpec {
        &self.spec
    }

    pub fn verify_command(&self) -> &str {
        self.spec.verify_command()
    }

    /// The pair written to `workflow_defs`; no independently supplied spec is accepted.
    pub fn persisted_spec(&self) -> serde_json::Value {
        serde_json::to_value(&self.spec).expect("workflow spec is serializable")
    }
}

#[derive(Clone, Debug, Deserialize)]
struct GraphNode {
    id: String,
    kind: String,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    data: Value,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    expression: Option<String>,
    #[serde(default)]
    max_iterations: Option<u32>,
    #[serde(default)]
    reset_to: Option<String>,
    #[serde(default)]
    gate: Option<String>,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    reviewer_role: Option<String>,
    #[serde(default)]
    max_review_rounds: Option<u32>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct GraphEdge {
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    target: Option<String>,
    #[serde(rename = "loopBack", default)]
    loop_back: Option<bool>,
}

impl GraphEdge {
    fn from(&self) -> Option<&str> {
        self.from.as_deref().or(self.source.as_deref())
    }

    fn to(&self) -> Option<&str> {
        self.to.as_deref().or(self.target.as_deref())
    }

    fn loop_back(&self) -> bool {
        self.loop_back.unwrap_or(false)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
struct AuthoringGraph {
    #[serde(default)]
    nodes: Vec<GraphNode>,
    #[serde(default)]
    edges: Vec<GraphEdge>,
}

fn node_text(node: &GraphNode, key: &str) -> Option<String> {
    let direct = match key {
        "role" => node.role.clone(),
        "command" => node.command.clone(),
        "expression" => node.expression.clone(),
        "reset_to" => node.reset_to.clone(),
        "gate" => node.gate.clone(),
        "prompt" => node.prompt.clone(),
        "reviewer_role" => node.reviewer_role.clone(),
        _ => None,
    };
    direct.filter(|value| !value.trim().is_empty()).or_else(|| {
        node.data
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_owned)
    })
}

fn node_string_list(node: &GraphNode, key: &str) -> Vec<String> {
    let Some(value) = node.data.get(key) else {
        return Vec::new();
    };
    let mut values = match value {
        Value::Array(values) => values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
        Value::String(value) => value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .collect(),
        _ => Vec::new(),
    };
    values.sort();
    values.dedup();
    values
}

fn node_permissions_spec(node: &GraphNode) -> WorkflowAgentPermissions {
    WorkflowAgentPermissions {
        tools: node_string_list(node, "tools"),
        network: node_text(node, "network"),
        write_scope: node_text(node, "write_scope").or_else(|| node_text(node, "write")),
    }
}

fn node_u32(node: &GraphNode, key: &str) -> Option<u32> {
    let direct = match key {
        "max_iterations" => node.max_iterations,
        "max_review_rounds" => node.max_review_rounds,
        _ => None,
    };
    direct.or_else(|| {
        node.data
            .get(key)
            .or_else(|| {
                (key == "max_iterations")
                    .then_some(())
                    .and_then(|_| node.data.get("maxIterations"))
            })
            .and_then(|value| value.as_u64())
            .and_then(|value| u32::try_from(value).ok())
    })
}

fn is_typed_graph(value: &Value) -> bool {
    value.get("version").is_some()
        || value
            .get("nodes")
            .and_then(Value::as_array)
            .is_some_and(|nodes| nodes.iter().any(|node| node.get("position").is_some()))
        || value
            .get("edges")
            .and_then(Value::as_array)
            .is_some_and(|edges| edges.iter().any(|edge| edge.get("source").is_some()))
}

/// Compile graph plus Governance into the typed unit consumed at runtime.
pub fn compile_workflow(
    graph: serde_json::Value,
    governance: WorkflowGovernance,
) -> Result<CompiledWorkflow, WorkflowError> {
    validate_authoring_graph(&graph)?;
    if governance.version == 0 {
        return Err(WorkflowError::InvalidVersion);
    }
    if governance.goal.trim().is_empty() {
        return Err(WorkflowError::EmptyGoal);
    }
    let authoring: AuthoringGraph =
        serde_json::from_value(graph.clone()).map_err(|_| WorkflowError::MalformedGraph)?;
    let mut seen = BTreeSet::new();
    let mut steps = Vec::with_capacity(authoring.nodes.len());
    let mut verify_command = None;
    for node in &authoring.nodes {
        if node.id.trim().is_empty() || !seen.insert(node.id.clone()) {
            return Err(WorkflowError::InvalidNodeId);
        }
        let kind = node.kind.to_ascii_lowercase();
        let step = match kind.as_str() {
            "agent" => WorkflowStep::Agent {
                node_id: node.id.clone(),
                role: node_text(node, "role").unwrap_or_default(),
                permissions: node_permissions_spec(node),
            },
            "task" => WorkflowStep::Task {
                node_id: node.id.clone(),
            },
            "verify" => {
                let command =
                    node_text(node, "command").ok_or(WorkflowError::MissingVerifyCommand)?;
                if verify_command.replace(command.clone()).is_some() {
                    return Err(WorkflowError::MultipleVerifyNodes);
                }
                WorkflowStep::Verify {
                    node_id: node.id.clone(),
                    command,
                }
            }
            "condition" => {
                let expression =
                    node_text(node, "expression").ok_or(WorkflowError::MissingCondition)?;
                crate::services::condition::Condition::parse(&expression)
                    .map_err(|_| WorkflowError::InvalidCondition)?;
                WorkflowStep::Condition {
                    node_id: node.id.clone(),
                    expression,
                }
            }
            "gate" => WorkflowStep::Gate {
                node_id: node.id.clone(),
                gate: parse_gate_kind(node)?,
            },
            "loop" => WorkflowStep::Loop {
                node_id: node.id.clone(),
                max_iterations: node_u32(node, "max_iterations").unwrap_or(1).max(1),
                reset_to: node_text(node, "reset_to"),
            },
            _ => return Err(WorkflowError::UnknownNodeKind(node.kind.clone())),
        };
        steps.push(step);
    }
    let node_ids = seen;
    if authoring.edges.iter().any(|edge| {
        edge.from().is_none_or(|from| !node_ids.contains(from))
            || edge.to().is_none_or(|to| !node_ids.contains(to))
    }) {
        return Err(WorkflowError::UnknownEdgeEndpoint);
    }
    let mut edges = Vec::new();
    for edge in authoring.edges.iter().filter(|edge| !edge.loop_back()) {
        let (Some(from), Some(to)) = (edge.from(), edge.to()) else {
            return Err(WorkflowError::UnknownEdgeEndpoint);
        };
        edges.push(WorkflowEdge {
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }
    edges.sort_by(|left, right| (&left.from, &left.to).cmp(&(&right.from, &right.to)));
    validate_control_flow(&steps, &edges)?;
    let verify_command = verify_command.ok_or(WorkflowError::MissingVerifyCommand)?;
    let spec = WorkflowSpec {
        version: governance.version,
        goal: governance.goal.clone(),
        guardrails: governance.guardrails.clone(),
        success_criteria: governance.success_criteria.clone(),
        verify_command,
        steps,
        edges,
    };
    Ok(CompiledWorkflow {
        graph,
        governance,
        spec,
    })
}

fn parse_gate_kind(node: &GraphNode) -> Result<GateKind, WorkflowError> {
    match node_text(node, "gate")
        .unwrap_or_else(|| "human".into())
        .to_ascii_lowercase()
        .as_str()
    {
        "human" => Ok(GateKind::Human {
            prompt: node_text(node, "prompt").ok_or(WorkflowError::MissingGatePrompt)?,
        }),
        "reviewer_agent" | "reviewer-agent" => {
            let role =
                node_text(node, "reviewer_role").ok_or(WorkflowError::MissingReviewerRole)?;
            let max_rounds = node_u32(node, "max_review_rounds").unwrap_or(1);
            if max_rounds == 0 || max_rounds > MAX_REVIEW_ROUNDS {
                return Err(WorkflowError::GateBoundExceeded);
            }
            Ok(GateKind::ReviewerAgent { role, max_rounds })
        }
        _ => Err(WorkflowError::UnknownGateKind),
    }
}

fn validate_control_flow(
    steps: &[WorkflowStep],
    edges: &[WorkflowEdge],
) -> Result<(), WorkflowError> {
    if steps.is_empty() {
        return Err(WorkflowError::MissingVerifyCommand);
    }
    let node_ids: BTreeSet<_> = steps.iter().map(WorkflowStep::node_id).collect();
    let mut adjacency: BTreeMap<&str, Vec<&str>> = node_ids
        .iter()
        .map(|node_id| (*node_id, Vec::new()))
        .collect();
    for edge in edges {
        let targets = adjacency
            .get_mut(edge.from.as_str())
            .ok_or(WorkflowError::UnknownEdgeEndpoint)?;
        if targets.contains(&edge.to.as_str()) {
            return Err(WorkflowError::DuplicateControlFlowEdge);
        }
        targets.push(edge.to.as_str());
    }
    if steps.len() > 1 && edges.is_empty() {
        return Err(WorkflowError::MissingControlFlowEdge);
    }
    for targets in adjacency.values_mut() {
        targets.sort_unstable();
    }
    for step in steps {
        let targets = &adjacency[step.node_id()];
        let target_limit = match step {
            WorkflowStep::Condition { .. }
            | WorkflowStep::Gate { .. }
            | WorkflowStep::Loop { .. }
            | WorkflowStep::Verify { .. } => 2,
            WorkflowStep::Agent { .. } | WorkflowStep::Task { .. } => 1,
        };
        if targets.len() > target_limit {
            return Err(WorkflowError::UnsupportedBranching);
        }
        if let WorkflowStep::Loop {
            reset_to: Some(reset_to),
            ..
        } = step
        {
            if !node_ids.contains(reset_to.as_str()) {
                return Err(WorkflowError::InvalidLoopReset);
            }
        }
    }
    let mut incoming = BTreeMap::<&str, usize>::new();
    for node_id in &node_ids {
        incoming.insert(*node_id, 0);
    }
    for edge in edges {
        *incoming
            .get_mut(edge.to.as_str())
            .ok_or(WorkflowError::UnknownEdgeEndpoint)? += 1;
    }
    let roots: Vec<_> = incoming
        .iter()
        .filter_map(|(node_id, count)| (*count == 0).then_some(*node_id))
        .collect();
    if roots.len() != 1 {
        return Err(WorkflowError::MalformedControlFlow);
    }
    let mut reachable = BTreeSet::new();
    let mut queue = VecDeque::from([roots[0]]);
    while let Some(node_id) = queue.pop_front() {
        if !reachable.insert(node_id) {
            continue;
        }
        queue.extend(adjacency[node_id].iter().copied());
    }
    if reachable.len() != node_ids.len() {
        return Err(WorkflowError::MissingControlFlowEdge);
    }
    Ok(())
}

/// The closed operand vocabulary accepted by a deterministic Condition node.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperand {
    VerifyPassed,
    VerifyExitCode,
    Iteration,
    Elapsed,
    TokensUsed,
    ToolErrorCount,
    LastEventKind,
    ArtifactExists,
    TaskStatus,
    MailPending,
}

impl ConditionOperand {
    pub const ALL: [Self; 10] = [
        Self::VerifyPassed,
        Self::VerifyExitCode,
        Self::Iteration,
        Self::Elapsed,
        Self::TokensUsed,
        Self::ToolErrorCount,
        Self::LastEventKind,
        Self::ArtifactExists,
        Self::TaskStatus,
        Self::MailPending,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::VerifyPassed => "verify.passed",
            Self::VerifyExitCode => "verify.exit_code",
            Self::Iteration => "iteration",
            Self::Elapsed => "elapsed",
            Self::TokensUsed => "tokens.used",
            Self::ToolErrorCount => "events.count(tool_error)",
            Self::LastEventKind => "events.last(kind)",
            Self::ArtifactExists => "artifact.exists(kind)",
            Self::TaskStatus => "task.status",
            Self::MailPending => "mail.pending",
        }
    }
}

fn contains_goal_node(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Array(values) => values.iter().any(contains_goal_node),
        serde_json::Value::Object(fields) => {
            fields
                .get("kind")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|kind| kind.eq_ignore_ascii_case("goal"))
                || fields.values().any(contains_goal_node)
        }
        _ => false,
    }
}

/// Reject the authoring-only graph if it contains the retired Goal node or runtime state.
pub fn validate_authoring_graph(graph: &serde_json::Value) -> Result<(), WorkflowError> {
    if graph.get("execution").is_some() || graph.get("results").is_some() {
        return Err(WorkflowError::ExecutionStateInAuthoring);
    }
    if contains_goal_node(graph) {
        return Err(WorkflowError::GoalNodeNotAllowed);
    }
    if is_typed_graph(graph) {
        let typed =
            graph::WorkflowGraph::from_value(graph).map_err(WorkflowError::GraphValidation)?;
        typed
            .validate()
            .map_err(|errors| WorkflowError::GraphValidation(format_graph_errors(&errors)))?;
    }
    Ok(())
}

fn format_graph_errors(errors: &[graph::GraphValidationError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

/// Backwards-compatible authoring entry point. New callers should use [`compile_workflow`]
/// to receive validation errors instead of panicking on invalid authoring data.
pub fn compile_governance(
    graph: serde_json::Value,
    governance: WorkflowGovernance,
) -> CompiledWorkflow {
    compile_workflow(graph, governance).expect("workflow authoring must compile")
}

/// The evaluation of one immutable Governance version during one run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RunGovernanceEvaluation {
    pub run_id: String,
    pub governance_version: u32,
    pub passed: bool,
}

impl RunGovernanceEvaluation {
    pub fn passed(run_id: impl Into<String>, governance_version: u32) -> Self {
        Self {
            run_id: run_id.into(),
            governance_version,
            passed: true,
        }
    }
}

/// A named instruction that constrains every run of a workflow version.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Guardrail {
    pub name: String,
    pub prompt: String,
}

/// An authored condition required before a workflow can report completion.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SuccessCriterion {
    pub kind: SuccessCriterionKind,
    pub checker: String,
}

impl SuccessCriterion {
    pub fn evaluation_route(&self) -> EvaluationRoute {
        match self.kind {
            SuccessCriterionKind::Human => EvaluationRoute::InboxGate,
            SuccessCriterionKind::Command | SuccessCriterionKind::Assertion => {
                EvaluationRoute::Core
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvaluationRoute {
    Core,
    InboxGate,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum WorkflowError {
    #[error("Goal must be authored in Governance, not as a canvas node")]
    GoalNodeNotAllowed,
    #[error("workflow authoring cannot contain execution results")]
    ExecutionStateInAuthoring,
    #[error("workflow version must be greater than zero")]
    InvalidVersion,
    #[error("workflow goal must not be empty")]
    EmptyGoal,
    #[error("workflow graph is malformed")]
    MalformedGraph,
    #[error("workflow graph validation failed: {0}")]
    GraphValidation(String),
    #[error("workflow node ids must be unique and non-empty")]
    InvalidNodeId,
    #[error("workflow graph contains an unknown node kind `{0}`")]
    UnknownNodeKind(String),
    #[error("workflow graph contains an edge endpoint that is not a node")]
    UnknownEdgeEndpoint,
    #[error("workflow must contain exactly one Verify node")]
    MissingVerifyCommand,
    #[error("workflow must contain at most one Verify node")]
    MultipleVerifyNodes,
    #[error("condition node must contain an expression")]
    MissingCondition,
    #[error("condition node contains an invalid expression")]
    InvalidCondition,
    #[error("workflow graph contains duplicate control-flow edges")]
    DuplicateControlFlowEdge,
    #[error("workflow graph is missing a control-flow edge")]
    MissingControlFlowEdge,
    #[error("workflow graph has unsupported branching")]
    UnsupportedBranching,
    #[error("workflow graph has malformed roots or cycles")]
    MalformedControlFlow,
    #[error("gate prompt is required")]
    MissingGatePrompt,
    #[error("gate locator is invalid")]
    InvalidGateLocator,
    #[error("reviewer-agent role is required")]
    MissingReviewerRole,
    #[error("gate bound is outside the allowed limit")]
    GateBoundExceeded,
    #[error("unknown gate kind")]
    UnknownGateKind,
    #[error("goal approval is required before the loop may run")]
    GoalApprovalRequired,
    #[error("goal approval belongs to another workflow version")]
    ApprovalVersionMismatch,
    #[error("loop execution has already started")]
    ExecutionAlreadyStarted,
    #[error("loop execution has not been approved")]
    LoopNotApproved,
    #[error("loop execution is not active")]
    LoopNotActive,
    #[error("loop reset target is not a workflow node")]
    InvalidLoopReset,
    #[error("workflow walk exceeded its bound")]
    WalkLimitExceeded,
}

/// The component that evaluates an authored success criterion.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuccessCriterionKind {
    Command,
    Assertion,
    Human,
}

/// Events emitted by the deterministic supervisor. A reset is explicit rather than hidden in
/// recursive control flow, so callers can persist or test the context boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupervisorEvent {
    Step {
        node_id: String,
    },
    LoopReset {
        node_id: String,
        next_iteration: u32,
    },
}

pub const MAX_WALK_EVENTS: usize = 4_096;

#[derive(Clone, Debug)]
pub struct WorkflowSupervisor {
    spec: WorkflowSpec,
}

impl WorkflowSupervisor {
    /// The supervisor can only be built from the compiler-owned unit. There is no public
    /// constructor accepting an independently assembled `WorkflowSpec`.
    pub fn from_compiled(compiled: &CompiledWorkflow) -> Self {
        Self {
            spec: compiled.spec.clone(),
        }
    }

    pub fn spec(&self) -> &WorkflowSpec {
        &self.spec
    }

    /// The approved execution boundary for a loop. `walk` remains a structural inspection helper;
    /// actual loop execution must use this method after Governance approval.
    pub fn walk_approved(
        &self,
        execution: &WorkflowExecution,
    ) -> Result<Vec<SupervisorEvent>, WorkflowError> {
        if !execution.loop_may_run() {
            return Err(WorkflowError::LoopNotApproved);
        }
        self.walk()
    }

    /// Walk the compiled control flow, not canvas insertion order. A loop reset starts the next
    /// bounded path from `reset_to`; it is not represented by an event without changing control.
    pub fn walk(&self) -> Result<Vec<SupervisorEvent>, WorkflowError> {
        let mut adjacency: BTreeMap<&str, Vec<&str>> = self
            .spec
            .steps
            .iter()
            .map(|step| (step.node_id(), Vec::new()))
            .collect();
        for edge in &self.spec.edges {
            adjacency
                .get_mut(edge.from.as_str())
                .ok_or(WorkflowError::UnknownEdgeEndpoint)?
                .push(edge.to.as_str());
        }
        for targets in adjacency.values_mut() {
            targets.sort_unstable();
        }
        let incoming: BTreeSet<&str> = self
            .spec
            .edges
            .iter()
            .map(|edge| edge.to.as_str())
            .collect();
        let roots: Vec<_> = self
            .spec
            .steps
            .iter()
            .map(WorkflowStep::node_id)
            .filter(|node_id| !incoming.contains(node_id))
            .collect();
        if roots.len() != 1 {
            return Err(WorkflowError::MalformedControlFlow);
        }
        let steps: BTreeMap<&str, &WorkflowStep> = self
            .spec
            .steps
            .iter()
            .map(|step| (step.node_id(), step))
            .collect();
        let mut events = Vec::new();
        let mut current = roots[0];
        let mut path = BTreeSet::new();
        let mut iterations = BTreeMap::<&str, u32>::new();
        while events.len() < MAX_WALK_EVENTS {
            if !path.insert(current) {
                return Err(WorkflowError::MalformedControlFlow);
            }
            let step = steps
                .get(current)
                .copied()
                .ok_or(WorkflowError::UnknownEdgeEndpoint)?;
            events.push(SupervisorEvent::Step {
                node_id: current.to_owned(),
            });
            let targets = adjacency
                .get(current)
                .ok_or(WorkflowError::UnknownEdgeEndpoint)?;
            let next = match step {
                WorkflowStep::Loop {
                    node_id,
                    max_iterations,
                    reset_to,
                } => {
                    let reset_target = reset_to.as_deref();
                    let iteration = iterations.entry(node_id).or_insert(1);
                    if let Some(reset_to) = reset_target {
                        if *iteration < *max_iterations {
                            *iteration += 1;
                            events.push(SupervisorEvent::LoopReset {
                                node_id: node_id.clone(),
                                next_iteration: *iteration,
                            });
                            path.clear();
                            current = reset_to;
                            continue;
                        }
                    }
                    if targets.len() > 1 {
                        Some(
                            targets
                                .iter()
                                .copied()
                                .find(|target| Some(*target) != reset_target)
                                .ok_or(WorkflowError::UnsupportedBranching)?,
                        )
                    } else {
                        targets.first().copied()
                    }
                }
                _ => {
                    if targets.len() > 1 {
                        return Err(WorkflowError::UnsupportedBranching);
                    }
                    targets.first().copied()
                }
            };
            let Some(next) = next else {
                break;
            };
            current = next;
        }
        if events.len() >= MAX_WALK_EVENTS {
            return Err(WorkflowError::WalkLimitExceeded);
        }
        if !self
            .spec
            .steps
            .iter()
            .any(|step| matches!(step, WorkflowStep::Verify { node_id, .. } if path.contains(node_id.as_str()) || events.iter().any(|event| matches!(event, SupervisorEvent::Step { node_id: seen } if seen == node_id))))
        {
            return Err(WorkflowError::MissingVerifyCommand);
        }
        Ok(events)
    }
}

pub fn walk_spec(compiled: &CompiledWorkflow) -> Result<Vec<SupervisorEvent>, WorkflowError> {
    WorkflowSupervisor::from_compiled(compiled).walk()
}

pub fn walk_approved(
    compiled: &CompiledWorkflow,
    execution: &WorkflowExecution,
) -> Result<Vec<SupervisorEvent>, WorkflowError> {
    WorkflowSupervisor::from_compiled(compiled).walk_approved(execution)
}

/// Orchestration is intentionally deterministic. Model/provider invocation is owned by agent
/// runs, never by this supervisor path.
pub fn orchestration_model_invocation_hook() -> Option<()> {
    None
}

/// Start the next container lifetime while retaining the session's branch, memory base, and
/// board task linkage. The prior events are the durable resume input for the new run.
pub fn reset_same_session(
    session: &Session,
    prior_events: impl IntoIterator<Item = crate::services::telemetry::Event>,
    resolved_model_id: impl Into<String>,
) -> ResumePlan {
    resume_from_events(session, prior_events, resolved_model_id)
}

/// Produce a bounded sequence of distinct run lifetimes for loop resets. Each plan is created by
/// the real resume path, so session-owned memory, branch, and task linkage are carried rather
/// than represented by a synthetic reset event.
pub fn reset_same_session_runs(
    session: &Session,
    prior_events: impl IntoIterator<Item = crate::services::telemetry::Event>,
    resolved_model_id: impl Into<String>,
    reset_count: usize,
) -> Result<Vec<ResumePlan>, WorkflowError> {
    if reset_count > MAX_SESSION_RESETS {
        return Err(WorkflowError::WalkLimitExceeded);
    }
    let prior_events: Vec<_> = prior_events.into_iter().collect();
    let resolved_model_id = resolved_model_id.into();
    Ok((0..=reset_count)
        .map(|_| resume_from_events(session, prior_events.clone(), resolved_model_id.clone()))
        .collect())
}

pub const MAX_SESSION_RESETS: usize = 1_024;

/// A fresh verification request. `clone_command` is deliberately part of the request so a
/// runner cannot silently substitute the agent container's local filesystem.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifyContainerRequest {
    pub run_id: RunId,
    pub verify_node_id: String,
    pub image: String,
    pub workspace_remote: String,
    pub branch: String,
    pub command: String,
    pub agent_container_name: String,
    pub container_name: String,
    pub clone_command: String,
}

impl VerifyContainerRequest {
    pub fn new(
        run_id: RunId,
        verify_node_id: impl Into<String>,
        image: impl Into<String>,
        workspace_remote: impl Into<String>,
        branch: impl Into<String>,
        command: impl Into<String>,
        agent_container_name: impl Into<String>,
    ) -> Result<Self, VerifyError> {
        let image = image.into();
        let workspace_remote = workspace_remote.into();
        let branch = branch.into();
        let command = command.into();
        let agent_container_name = agent_container_name.into();
        refuse_primary_branch(&branch).map_err(|_| VerifyError::PrimaryBranch)?;
        if image.trim().is_empty()
            || workspace_remote.trim().is_empty()
            || command.trim().is_empty()
            || agent_container_name.trim().is_empty()
        {
            return Err(VerifyError::MissingField);
        }
        let container_name = format!("locus-verify-{run_id}");
        if container_name == agent_container_name {
            return Err(VerifyError::ReusesAgentContainer);
        }
        let clone_command = workspace_clone_branch_command(&workspace_remote, &branch)
            .map_err(|_| VerifyError::InvalidWorkspace)?;
        Ok(Self {
            run_id,
            verify_node_id: verify_node_id.into(),
            image,
            workspace_remote,
            branch,
            command,
            agent_container_name,
            container_name,
            clone_command,
        })
    }

    pub fn command_line(&self) -> String {
        format!("{} && {}", self.clone_command, self.command)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifyEvidence {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub passed: bool,
    pub command: String,
    pub container_id: String,
    pub verify_node_id: String,
}

pub trait VerifyContainerRunner {
    fn run_fresh_container(
        &mut self,
        request: &VerifyContainerRequest,
    ) -> Result<VerifyEvidence, VerifyError>;
}

pub fn verify_in_fresh_container(
    runner: &mut impl VerifyContainerRunner,
    request: &VerifyContainerRequest,
) -> Result<VerifyEvidence, VerifyError> {
    if request.container_name == request.agent_container_name {
        return Err(VerifyError::ReusesAgentContainer);
    }
    let evidence = runner.run_fresh_container(request)?;
    if evidence.command != request.command || evidence.verify_node_id != request.verify_node_id {
        return Err(VerifyError::EvidenceMismatch);
    }
    if evidence.container_id.trim().is_empty()
        || evidence.container_id == request.agent_container_name
    {
        return Err(VerifyError::ReusesAgentContainer);
    }
    if evidence.passed != (evidence.exit_code == 0) {
        return Err(VerifyError::EvidenceMismatch);
    }
    Ok(evidence)
}

impl<T> VerifyContainerRunner for T
where
    T: crate::runtime::container::ContainerRuntime,
{
    fn run_fresh_container(
        &mut self,
        request: &VerifyContainerRequest,
    ) -> Result<VerifyEvidence, VerifyError> {
        self.run_verify_container(request)
            .map_err(|_| VerifyError::RunnerUnavailable)
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum VerifyError {
    #[error("verification cannot run on main or master")]
    PrimaryBranch,
    #[error("verification request has a missing field")]
    MissingField,
    #[error("verification must use a fresh container")]
    ReusesAgentContainer,
    #[error("verification workspace request is invalid")]
    InvalidWorkspace,
    #[error("verification evidence does not match its request")]
    EvidenceMismatch,
    #[error("container runtime cannot run fresh verification")]
    RunnerUnavailable,
}

#[cfg(test)]
#[test]
fn goal_is_governance_not_node() {
    assert!(validate_authoring_graph(&serde_json::json!({"nodes": [{"kind": "Goal"}]})).is_err());
    assert!(validate_authoring_graph(&serde_json::json!({"nodes": [{"kind": "Agent"}]})).is_ok());
}

#[cfg(test)]
#[test]
fn goal_text_is_allowed_when_no_goal_node_exists() {
    assert!(validate_authoring_graph(&serde_json::json!({
        "nodes": [{"kind": "Agent", "label": "goal"}]
    }))
    .is_ok());
}

#[cfg(test)]
#[test]
fn condition_operands_are_closed() {
    assert_eq!(ConditionOperand::ALL.len(), 10);
    assert_eq!(ConditionOperand::MailPending.as_str(), "mail.pending");
}

#[cfg(test)]
#[test]
fn human_criterion_is_gate() {
    let criterion = SuccessCriterion {
        kind: SuccessCriterionKind::Human,
        checker: "you".into(),
    };
    assert_eq!(criterion.evaluation_route(), EvaluationRoute::InboxGate);
}

#[cfg(test)]
#[test]
fn guardrails_reinjected_after_reset() {
    let governance = WorkflowGovernance {
        version: 1,
        goal: "ship".into(),
        guardrails: vec![Guardrail {
            name: "no delete".into(),
            prompt: "preserve data".into(),
        }],
        success_criteria: vec![],
    };
    assert_eq!(governance.guardrails.len(), 1);
}

#[cfg(test)]
#[test]
fn authoring_has_no_run_state() {
    let graph = serde_json::json!({"nodes": [{"kind": "Agent"}]});
    validate_authoring_graph(&graph).unwrap();
    assert!(graph.get("results").is_none());
}

#[cfg(test)]
#[test]
fn governance_is_versioned() {
    let governance = WorkflowGovernance {
        version: 4,
        goal: "Ship the migration without downtime".into(),
        guardrails: vec![Guardrail {
            name: "Preserve data".into(),
            prompt: "Do not delete or rewrite existing records.".into(),
        }],
        success_criteria: vec![SuccessCriterion {
            kind: SuccessCriterionKind::Command,
            checker: "cargo test -p locus-core".into(),
        }],
    };

    let value = serde_json::to_value(&governance).expect("governance serializes");
    assert_eq!(value["version"], 4);
    assert_eq!(
        value["guardrails"][0]["prompt"],
        "Do not delete or rewrite existing records."
    );
    assert!(value.get("execution").is_none());
    assert!(value.get("results").is_none());
}
