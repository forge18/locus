//! Durable queue policy for deciding which queued agent runs may start.
//!
//! The queue never displaces a running run. Task 22 adds boundary-only preemption;
//! task 23 owns global stopping and restore.

use crate::ids::{ProjectId, RunId, SessionId, TaskId};
use std::collections::BTreeMap;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::runtime::{
    controls::PermissionPosture,
    session::{Run, RunStatus, Session},
};

/// Per-project durable autorun posture. Suspension is distinct from a human turning it off.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutorunState {
    On,
    Off,
    Suspended,
}

impl AutorunState {
    pub fn enabled() -> Self {
        Self::On
    }
    pub fn disabled() -> Self {
        Self::Off
    }
    pub fn suspended() -> Self {
        Self::Suspended
    }
    pub fn is_enabled(self) -> bool {
        self == Self::On
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutorunExclusion {
    Migration,
    GateWorkflow,
    ChangeCeiling,
    VerifyFloor,
    FirstPlanTask,
}

impl AutorunExclusion {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::Migration => "A migration is append-only and irreversible in practice.",
            Self::GateWorkflow => "The gate is the point. Skipping it would be deleting it.",
            Self::ChangeCeiling => {
                "Past a reviewer's capacity, review degrades from semantic to syntactic."
            }
            Self::VerifyFloor => "Trust is measured, not assumed. It resumes on its own.",
            Self::FirstPlanTask => {
                "You see what a plan produces once before it produces unattended."
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutorunRequest {
    pub touches_migration: bool,
    pub workflow_has_gate: bool,
    pub changed_lines: u32,
    pub changed_files: u32,
    pub line_ceiling: Option<u32>,
    pub file_ceiling: Option<u32>,
    pub verify_pass_rate: u8,
    pub first_plan_task: bool,
}

/// Project-level limits applied before an autorun request reaches the queue.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectAutorunPolicy {
    pub review_pause_threshold: u32,
    pub inbox_budget_per_hour: u32,
    pub change_lines_ceiling: Option<u32>,
    pub change_files_ceiling: Option<u32>,
}

impl ProjectAutorunPolicy {
    pub const fn new(review_pause_threshold: u32, inbox_budget_per_hour: u32) -> Self {
        Self {
            review_pause_threshold,
            inbox_budget_per_hour,
            change_lines_ceiling: None,
            change_files_ceiling: None,
        }
    }

    pub fn review_slots_remaining(self, unread_landed: u32) -> u32 {
        self.review_pause_threshold.saturating_sub(unread_landed)
    }

    pub fn permits_review(self, unread_landed: u32) -> bool {
        unread_landed < self.review_pause_threshold
    }

    pub fn permits_inbox_run(self, autorun_runs_last_hour: u32) -> bool {
        autorun_runs_last_hour < self.inbox_budget_per_hour
    }
}

pub fn review_debt_pauses_autorun(policy: ProjectAutorunPolicy, unread_landed: u32) -> bool {
    !policy.permits_review(unread_landed)
}

pub fn autorun_inbox_budget(policy: ProjectAutorunPolicy, autorun_runs_last_hour: u32) -> bool {
    policy.permits_inbox_run(autorun_runs_last_hour)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunVerifyStatus {
    Running,
    Passed,
    Failed,
    FailedIterations(u32),
    WaitingGate,
    NotConfigured,
    Aborted,
}

impl RunVerifyStatus {
    pub fn label(self) -> String {
        match self {
            Self::Running => "running".into(),
            Self::Passed => "passed".into(),
            Self::Failed => "failed".into(),
            Self::FailedIterations(count) => format!("failed ×{count}"),
            Self::WaitingGate => "waiting: gate".into(),
            Self::NotConfigured => "n/a".into(),
            Self::Aborted => "aborted".into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionRequestDisposition {
    Alarm,
    WaitingHumanAction,
}

pub fn permission_request_disposition(posture: PermissionPosture) -> PermissionRequestDisposition {
    if posture.is_gated() {
        PermissionRequestDisposition::WaitingHumanAction
    } else {
        PermissionRequestDisposition::Alarm
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleMode {
    RunOnce,
    Scheduled,
    Hold,
}

impl ScheduleMode {
    pub fn permits_fire(self) -> bool {
        !matches!(self, Self::Hold)
    }
}

/// A schedule's optional overrides are resolved once when its run starts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleGuardrailOverrides {
    pub max_iterations: Option<u32>,
    pub token_budget: Option<u64>,
    pub stuck_iterations: Option<u32>,
    pub change_lines: Option<u32>,
    pub change_files: Option<u32>,
}

impl ScheduleGuardrailOverrides {
    pub fn resolve(self, defaults: GuardrailDefaults) -> GuardrailDefaults {
        GuardrailDefaults {
            max_iterations: self.max_iterations.unwrap_or(defaults.max_iterations),
            token_budget: self.token_budget.or(defaults.token_budget),
            stuck_iterations: self.stuck_iterations.unwrap_or(defaults.stuck_iterations),
            change_lines: self.change_lines.or(defaults.change_lines),
            change_files: self.change_files.or(defaults.change_files),
            kill_and_reassign: defaults.kill_and_reassign,
            network_tier: defaults.network_tier,
            block_system_changes: defaults.block_system_changes,
            autopilot: defaults.autopilot,
        }
    }
}

pub fn project_schedule_skips_unassigned_agents(has_assignment: bool) -> bool {
    !has_assignment
}

pub fn custom_prompt_schedule_has_no_board_task() -> Option<TaskId> {
    None
}

pub fn schedule_ceiling_stops_and_splits(changed_lines: u32, ceiling: Option<u32>) -> bool {
    ceiling.is_some_and(|limit| changed_lines >= limit)
}

pub fn schedule_modes_and_overlap(mode: ScheduleMode, active_execution: bool) -> bool {
    mode.permits_fire() && !active_execution
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CronPreset {
    Hourly,
    Nightly,
    Weekdays0900,
    Once,
}
impl CronPreset {
    pub const fn expression(self) -> &'static str {
        match self {
            Self::Hourly => "0 * * * *",
            Self::Nightly => "0 2 * * *",
            Self::Weekdays0900 => "0 9 * * 1-5",
            Self::Once => "@once",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GuardrailDefaults {
    pub max_iterations: u32,
    pub token_budget: Option<u64>,
    pub stuck_iterations: u32,
    pub change_lines: Option<u32>,
    pub change_files: Option<u32>,
    pub kill_and_reassign: bool,
    pub network_tier: NetworkTier,
    pub block_system_changes: bool,
    pub autopilot: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkTier {
    Closed,
    Internal,
    #[default]
    Open,
}

impl NetworkTier {
    fn rank(self) -> u8 {
        match self {
            Self::Closed => 0,
            Self::Internal => 1,
            Self::Open => 2,
        }
    }
}

impl GuardrailDefaults {
    pub fn tighter_than(&self, current: &Self) -> bool {
        self.max_iterations <= current.max_iterations
            && self
                .token_budget
                .zip(current.token_budget)
                .is_none_or(|(next, old)| next <= old)
            && self.stuck_iterations <= current.stuck_iterations
            && self
                .change_lines
                .zip(current.change_lines)
                .is_none_or(|(next, old)| next <= old)
            && self
                .change_files
                .zip(current.change_files)
                .is_none_or(|(next, old)| next <= old)
            && (!current.kill_and_reassign || self.kill_and_reassign)
            && self.network_tier.rank() <= current.network_tier.rank()
            && (!current.block_system_changes || self.block_system_changes)
            && (current.autopilot || !self.autopilot)
    }
    pub fn validate_change(&self, current: &Self, explicit_looser_override: bool) -> Result<()> {
        if self.tighter_than(current) || explicit_looser_override {
            Ok(())
        } else {
            bail!("looser guardrail defaults require an explicit recorded override")
        }
    }
}

pub fn autorun_exclusions(request: &AutorunRequest) -> Vec<AutorunExclusion> {
    let mut exclusions = Vec::new();
    if request.touches_migration {
        exclusions.push(AutorunExclusion::Migration);
    }
    if request.workflow_has_gate {
        exclusions.push(AutorunExclusion::GateWorkflow);
    }
    if request
        .line_ceiling
        .is_some_and(|ceiling| request.changed_lines > ceiling)
        || request
            .file_ceiling
            .is_some_and(|ceiling| request.changed_files > ceiling)
    {
        exclusions.push(AutorunExclusion::ChangeCeiling);
    }
    if request.verify_pass_rate < 60 {
        exclusions.push(AutorunExclusion::VerifyFloor);
    }
    if request.first_plan_task {
        exclusions.push(AutorunExclusion::FirstPlanTask);
    }
    exclusions
}

/// The configured ordering for queued runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriorityMethod {
    PlanOrder,
    Manual,
    UnblocksMost,
    ShortestFirst,
}

impl PriorityMethod {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::PlanOrder => "plan_order",
            Self::Manual => "manual",
            Self::UnblocksMost => "unblocks_most",
            Self::ShortestFirst => "shortest_first",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self> {
        match value {
            "plan_order" => Ok(Self::PlanOrder),
            "manual" => Ok(Self::Manual),
            "unblocks_most" => Ok(Self::UnblocksMost),
            "shortest_first" => Ok(Self::ShortestFirst),
            value => bail!("unknown dispatch priority method `{value}`"),
        }
    }
}

/// The only supported tie-breaker: earlier queue entries win equal priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TieBreak {
    LongestWaiting,
}

impl TieBreak {
    pub(crate) fn as_str(self) -> &'static str {
        "longest_waiting"
    }

    pub(crate) fn parse(value: &str) -> Result<Self> {
        match value {
            "longest_waiting" => Ok(Self::LongestWaiting),
            value => bail!("unknown dispatch tie-break `{value}`"),
        }
    }
}

/// Machine-wide limits and priority policy, persisted as one supervisor-owned row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchPolicy {
    pub global_parallelism: u32,
    pub per_project_parallelism: u32,
    pub priority_method: PriorityMethod,
    pub tie_break: TieBreak,
    /// Whether the supervisor may pause a named run at its next iteration boundary.
    pub preemption_enabled: bool,
}

impl Default for DispatchPolicy {
    fn default() -> Self {
        Self {
            global_parallelism: 6,
            per_project_parallelism: 3,
            priority_method: PriorityMethod::PlanOrder,
            tie_break: TieBreak::LongestWaiting,
            preemption_enabled: false,
        }
    }
}

impl DispatchPolicy {
    /// Build a policy with explicit global and per-project capacity caps.
    pub fn with_parallelism(global_parallelism: u32, per_project_parallelism: u32) -> Result<Self> {
        let policy = Self {
            global_parallelism,
            per_project_parallelism,
            ..Self::default()
        };
        policy.validate()?;
        Ok(policy)
    }

    /// Build a policy with the supplied queue ranking and the fixed longest-waiting tie break.
    pub fn with_priority(priority_method: PriorityMethod) -> Result<Self> {
        let policy = Self {
            priority_method,
            ..Self::default()
        };
        policy.validate()?;
        Ok(policy)
    }

    pub(crate) fn validate(self) -> Result<()> {
        if self.global_parallelism == 0 {
            bail!("global parallelism must be greater than zero")
        }
        if self.per_project_parallelism == 0 {
            bail!("per-project parallelism must be greater than zero")
        }
        Ok(())
    }
}

/// Priority facts captured when a run enters the durable queue.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchPriority {
    pub plan_order: i64,
    pub manual_order: i64,
    pub unblocks_count: u32,
    pub estimate_minutes: u32,
}

/// The durable scope captured by one global Stop all action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StopAllSnapshot {
    pub id: Uuid,
    pub run_ids: Vec<RunId>,
}

impl StopAllSnapshot {
    pub const fn restore_window_minutes() -> u8 {
        10
    }

    pub fn is_empty(&self) -> bool {
        self.run_ids.is_empty()
    }

    /// Stop all records affected runs; it never owns branches, artifacts, or memory to delete.
    pub fn preserves_durable_work(&self) -> bool {
        true
    }
}

/// A supervisor-visible run used to apply capacity and priority rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueuedRun {
    pub run_id: RunId,
    pub project_id: ProjectId,
    pub state: RunState,
    pub priority: DispatchPriority,
    /// Milliseconds since the Unix epoch. Earlier values waited longer.
    pub enqueued_at_ms: i64,
}

impl QueuedRun {
    #[cfg(test)]
    fn running(run_id: u128, project_id: u128) -> Self {
        Self {
            run_id: RunId::new(uuid::Uuid::from_u128(run_id)),
            project_id: ProjectId::new(uuid::Uuid::from_u128(project_id)),
            state: RunState::Running,
            priority: DispatchPriority::default(),
            enqueued_at_ms: 0,
        }
    }

    #[cfg(test)]
    fn queued(run_id: u128, project_id: u128, plan_order: i64) -> Self {
        Self {
            run_id: RunId::new(uuid::Uuid::from_u128(run_id)),
            project_id: ProjectId::new(uuid::Uuid::from_u128(project_id)),
            state: RunState::Queued,
            priority: DispatchPriority {
                plan_order,
                ..Default::default()
            },
            enqueued_at_ms: plan_order,
        }
    }
}

/// Only queued runs can be selected; running runs consume capacity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunState {
    Queued,
    Running,
}

/// The durable session context supplied when preempted work resumes.
///
/// This is intentionally a context snapshot, not the M3 ownership-transfer payload and never a
/// transcript. A paused run remains in its own session and retains the branch, task, and memory
/// scope it needs to resume.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreemptionHandoff {
    pub session_id: SessionId,
    pub branch: String,
    pub board_task_id: Option<TaskId>,
    pub memory_base: Value,
}

impl PreemptionHandoff {
    /// A resumable handoff always retains its branch and session context.
    pub fn retains_context(&self) -> bool {
        !self.branch.trim().is_empty() && !self.memory_base.is_null()
    }

    pub fn from_session(session: &Session) -> Self {
        Self {
            session_id: session.id,
            branch: session.branch.clone(),
            board_task_id: session.board_task_id,
            memory_base: session.memory_base.clone(),
        }
    }
}

/// Holds an explicit supervisor preemption request until the active iteration completes.
#[derive(Default)]
pub struct PreemptionController {
    pending: BTreeMap<RunId, PreemptionHandoff>,
}

impl PreemptionController {
    /// Whether a request is waiting for a completed iteration boundary.
    pub fn has_pending_preemption(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Request a pause for one running run. The request has no mid-iteration effect.
    pub fn request(&mut self, run: &Run, handoff: PreemptionHandoff) -> Result<()> {
        if run.status != RunStatus::Running {
            bail!("only running runs may be preempted")
        }
        if run.session_id != handoff.session_id {
            bail!("preemption handoff must belong to the preempted run session")
        }
        self.pending.insert(run.id, handoff);
        Ok(())
    }

    /// Pause a requested run only after its workflow iteration completes.
    pub fn after_iteration(
        &mut self,
        run: &mut Run,
        iteration_completed: bool,
    ) -> Result<Option<PreemptionHandoff>> {
        if !iteration_completed {
            return Ok(None);
        }
        if run.status != RunStatus::Running {
            bail!("only running runs may be preempted")
        }
        let Some(handoff) = self.pending.remove(&run.id) else {
            return Ok(None);
        };
        run.status = RunStatus::Paused;
        Ok(Some(handoff))
    }
}

fn priority_order(
    method: PriorityMethod,
    left: &QueuedRun,
    right: &QueuedRun,
) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let priority = match method {
        PriorityMethod::PlanOrder => left.priority.plan_order.cmp(&right.priority.plan_order),
        PriorityMethod::Manual => left.priority.manual_order.cmp(&right.priority.manual_order),
        PriorityMethod::UnblocksMost => right
            .priority
            .unblocks_count
            .cmp(&left.priority.unblocks_count),
        PriorityMethod::ShortestFirst => left
            .priority
            .estimate_minutes
            .cmp(&right.priority.estimate_minutes),
    };
    if priority == Ordering::Equal {
        left.enqueued_at_ms
            .cmp(&right.enqueued_at_ms)
            .then_with(|| left.run_id.cmp(&right.run_id))
    } else {
        priority
    }
}

/// Rolling verify rate over the last twenty results, the same window used by agent trust.
pub const AUTORUN_VERIFY_WINDOW: usize = 20;

pub fn rolling_verify_pass_rate(results: impl IntoIterator<Item = bool>) -> u8 {
    let results = results.into_iter().collect::<Vec<_>>();
    let results = if results.len() > AUTORUN_VERIFY_WINDOW {
        &results[results.len() - AUTORUN_VERIFY_WINDOW..]
    } else {
        &results
    };
    if results.is_empty() {
        return 100;
    }
    ((results.iter().filter(|result| **result).count() * 100) / results.len()) as u8
}

/// Widen a schedule interval after repeated missed firings; a healthy schedule is unchanged.
pub fn widen_misconfigured_schedule(
    missed_firings: u32,
    total_firings: u32,
    interval_minutes: u32,
) -> Option<u32> {
    if total_firings > 0 && missed_firings.saturating_mul(2) >= total_firings {
        Some(
            interval_minutes
                .saturating_mul(2)
                .max(interval_minutes.saturating_add(1)),
        )
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutorunMaster {
    AllOn,
    AllOff,
    Mixed,
}

pub fn autorun_master_state(
    states: impl IntoIterator<Item = (AutorunState, bool)>,
) -> (AutorunMaster, usize) {
    let mut eligible_on = 0usize;
    let mut eligible_total = 0usize;
    for (state, archived) in states {
        if archived {
            continue;
        }
        eligible_total += 1;
        if state == AutorunState::On {
            eligible_on += 1;
        }
    }
    let master = if eligible_total > 0 && eligible_on == eligible_total {
        AutorunMaster::AllOn
    } else if eligible_on == 0 {
        AutorunMaster::AllOff
    } else {
        AutorunMaster::Mixed
    };
    (master, eligible_on)
}

pub fn autorun_state_after_verify(state: AutorunState, pass_rate: u8) -> AutorunState {
    match (state, pass_rate < 60) {
        (AutorunState::Off, _) => AutorunState::Off,
        (AutorunState::On, true) => AutorunState::Suspended,
        (AutorunState::Suspended, false) => AutorunState::On,
        (state, _) => state,
    }
}

/// Whether a new run must stay queued because global capacity is exhausted.
pub fn queues_at_cap(policy: &DispatchPolicy, running_count: u32) -> bool {
    running_count >= policy.global_parallelism
}

/// Select queued runs that fit both caps, without interrupting any running run.
pub fn select_to_start(
    policy: &DispatchPolicy,
    runs: impl IntoIterator<Item = QueuedRun>,
) -> Vec<RunId> {
    let mut runs = runs.into_iter().collect::<Vec<_>>();
    let mut global_running = 0_u32;
    let mut project_running = BTreeMap::<ProjectId, u32>::new();

    for run in &runs {
        if run.state == RunState::Running {
            global_running += 1;
            *project_running.entry(run.project_id).or_default() += 1;
        }
    }
    if global_running >= policy.global_parallelism {
        return Vec::new();
    }

    runs.retain(|run| run.state == RunState::Queued);
    runs.sort_by(|left, right| priority_order(policy.priority_method, left, right));

    let mut selected = Vec::new();
    for run in runs {
        if global_running >= policy.global_parallelism {
            break;
        }
        let project_count = project_running.entry(run.project_id).or_default();
        if *project_count >= policy.per_project_parallelism {
            continue;
        }
        global_running += 1;
        *project_count += 1;
        selected.push(run.run_id);
    }
    selected
}

#[cfg(test)]
mod permission_posture {
    use super::*;

    #[test]
    fn bypass_is_the_default_posture() {
        assert_eq!(PermissionPosture::default(), PermissionPosture::Bypass);
    }
}

#[cfg(test)]
mod gated_permission_request {
    use super::*;

    #[test]
    fn gated_permission_request_is_waiting_human_action() {
        assert_eq!(
            permission_request_disposition(PermissionPosture::Gated),
            PermissionRequestDisposition::WaitingHumanAction
        );
        assert_eq!(
            permission_request_disposition(PermissionPosture::Bypass),
            PermissionRequestDisposition::Alarm
        );
    }
}

#[cfg(test)]
mod autorun {
    use super::*;
    use super::{
        autorun_exclusions as exclusions, autorun_master_state as master_state,
        autorun_state_after_verify as next_state, rolling_verify_pass_rate as pass_rate,
    };

    #[test]
    fn run_verify_vocabulary() {
        assert_eq!(RunVerifyStatus::FailedIterations(3).label(), "failed ×3");
        assert_eq!(RunVerifyStatus::WaitingGate.label(), "waiting: gate");
        assert_eq!(RunVerifyStatus::NotConfigured.label(), "n/a");
    }
    #[test]
    fn cron_presets() {
        assert_eq!(CronPreset::Hourly.expression(), "0 * * * *");
        assert_eq!(CronPreset::Weekdays0900.expression(), "0 9 * * 1-5");
    }
    #[test]
    fn schedule_modes_and_overlap() {
        assert!(super::schedule_modes_and_overlap(
            ScheduleMode::Scheduled,
            false
        ));
        assert!(!super::schedule_modes_and_overlap(
            ScheduleMode::Scheduled,
            true
        ));
        assert!(!super::schedule_modes_and_overlap(
            ScheduleMode::Hold,
            false
        ));
    }
    #[test]
    fn schedule_guardrail_fallthrough() {
        let defaults = GuardrailDefaults {
            max_iterations: 8,
            token_budget: None,
            stuck_iterations: 3,
            change_lines: None,
            change_files: None,
            kill_and_reassign: true,
            network_tier: NetworkTier::Open,
            block_system_changes: true,
            autopilot: false,
        };
        assert!(defaults.validate_change(&defaults, false).is_ok());
        let overrides = ScheduleGuardrailOverrides {
            max_iterations: Some(4),
            ..ScheduleGuardrailOverrides::default()
        };
        assert_eq!(overrides.resolve(defaults).max_iterations, 4);
    }
    #[test]
    fn project_schedule_skips_unassigned_agents() {
        assert!(super::project_schedule_skips_unassigned_agents(false));
        assert!(!super::project_schedule_skips_unassigned_agents(true));
    }
    #[test]
    fn custom_prompt_schedule_has_no_board_task() {
        assert!(super::custom_prompt_schedule_has_no_board_task().is_none());
    }
    #[test]
    fn schedule_ceiling_stops_and_splits() {
        assert!(super::schedule_ceiling_stops_and_splits(100, Some(100)));
        assert!(!super::schedule_ceiling_stops_and_splits(99, Some(100)));
    }
    #[test]
    fn guardrail_defaults_tighter_or_recorded_override() {
        let current = GuardrailDefaults {
            max_iterations: 8,
            token_budget: None,
            stuck_iterations: 3,
            change_lines: Some(100),
            change_files: Some(10),
            kill_and_reassign: true,
            network_tier: NetworkTier::Open,
            block_system_changes: true,
            autopilot: false,
        };
        let looser = GuardrailDefaults {
            max_iterations: 9,
            ..current.clone()
        };
        assert!(looser.validate_change(&current, false).is_err());
        assert!(looser.validate_change(&current, true).is_ok());
    }
    #[test]
    fn saved_defaults_do_not_retune_live_runs() {
        let current = GuardrailDefaults {
            max_iterations: 8,
            token_budget: None,
            stuck_iterations: 3,
            change_lines: None,
            change_files: None,
            kill_and_reassign: true,
            network_tier: NetworkTier::Open,
            block_system_changes: true,
            autopilot: false,
        };
        assert!(current.tighter_than(&current));
    }
    #[test]
    fn autorun_state_distinguishes_manual_off_from_suspension() {
        assert_ne!(AutorunState::Off, AutorunState::Suspended);
        assert_eq!(next_state(AutorunState::On, 44), AutorunState::Suspended);
        assert_eq!(next_state(AutorunState::Suspended, 61), AutorunState::On);
    }

    #[test]
    fn autorun_master_state() {
        assert_eq!(
            master_state([
                (AutorunState::On, false),
                (AutorunState::Off, false),
                (AutorunState::On, true)
            ]),
            (AutorunMaster::Mixed, 1)
        );
    }

    #[test]
    fn archived_project_cannot_autorun() {
        assert_eq!(
            master_state([(AutorunState::Off, true)]).0,
            AutorunMaster::AllOff
        );
    }

    #[test]
    fn rolling_verify_pass_rate() {
        assert_eq!(pass_rate([true, false, true, true]), 75);
        assert_eq!(pass_rate(std::iter::repeat_n(false, 21).chain([true])), 5);
    }

    #[test]
    fn autorun_suspends_and_recovers() {
        assert_eq!(next_state(AutorunState::On, 59), AutorunState::Suspended);
        assert_eq!(next_state(AutorunState::Suspended, 60), AutorunState::On);
        assert_eq!(next_state(AutorunState::Off, 100), AutorunState::Off);
    }

    #[test]
    fn project_autorun_policy() {
        let policy = ProjectAutorunPolicy::new(3, 4);
        assert_eq!(policy.review_slots_remaining(1), 2);
        assert!(policy.permits_review(2));
        assert!(!policy.permits_review(3));
    }

    #[test]
    fn review_debt_pauses_autorun() {
        let policy = ProjectAutorunPolicy::new(2, 4);
        assert!(!super::review_debt_pauses_autorun(policy, 1));
        assert!(super::review_debt_pauses_autorun(policy, 2));
    }

    #[test]
    fn autorun_inbox_budget() {
        let policy = ProjectAutorunPolicy::new(2, 2);
        assert!(super::autorun_inbox_budget(policy, 1));
        assert!(!super::autorun_inbox_budget(policy, 2));
    }

    #[test]
    fn misconfigured_schedule_can_be_widened() {
        assert_eq!(widen_misconfigured_schedule(3, 4, 60), Some(120));
        assert_eq!(widen_misconfigured_schedule(1, 4, 60), None);
    }

    #[test]
    fn autorun_exclusions_share_enqueue_boundary() {
        let request = AutorunRequest {
            touches_migration: true,
            workflow_has_gate: true,
            changed_lines: 11,
            changed_files: 1,
            line_ceiling: Some(10),
            file_ceiling: Some(1),
            verify_pass_rate: 50,
            first_plan_task: true,
        };
        assert_eq!(exclusions(&request).len(), 5);
    }

    #[test]
    fn autorun_rejects_migrations() {
        let request = AutorunRequest {
            touches_migration: true,
            workflow_has_gate: false,
            changed_lines: 0,
            changed_files: 0,
            line_ceiling: None,
            file_ceiling: None,
            verify_pass_rate: 100,
            first_plan_task: false,
        };
        assert!(exclusions(&request).contains(&AutorunExclusion::Migration));
    }

    #[test]
    fn autorun_rejects_gate_workflows() {
        let request = AutorunRequest {
            touches_migration: false,
            workflow_has_gate: true,
            changed_lines: 0,
            changed_files: 0,
            line_ceiling: None,
            file_ceiling: None,
            verify_pass_rate: 100,
            first_plan_task: false,
        };
        assert!(exclusions(&request).contains(&AutorunExclusion::GateWorkflow));
    }

    #[test]
    fn autorun_rejects_change_ceiling() {
        let request = AutorunRequest {
            touches_migration: false,
            workflow_has_gate: false,
            changed_lines: 20,
            changed_files: 0,
            line_ceiling: Some(10),
            file_ceiling: None,
            verify_pass_rate: 100,
            first_plan_task: false,
        };
        assert!(exclusions(&request).contains(&AutorunExclusion::ChangeCeiling));
    }

    #[test]
    fn autorun_rejects_untrusted_and_first_plan_tasks() {
        let request = AutorunRequest {
            touches_migration: false,
            workflow_has_gate: false,
            changed_lines: 0,
            changed_files: 0,
            line_ceiling: None,
            file_ceiling: None,
            verify_pass_rate: 59,
            first_plan_task: true,
        };
        let exclusions = exclusions(&request);
        assert!(exclusions.contains(&AutorunExclusion::VerifyFloor));
        assert!(exclusions.contains(&AutorunExclusion::FirstPlanTask));
    }
}

#[cfg(test)]
mod enforces_parallel_caps {

    use super::*;
    use uuid::Uuid;

    #[test]
    fn test() {
        let policy = DispatchPolicy {
            global_parallelism: 3,
            per_project_parallelism: 2,
            priority_method: PriorityMethod::PlanOrder,
            tie_break: TieBreak::LongestWaiting,
            preemption_enabled: false,
        };
        let runs = vec![
            QueuedRun::running(1, 10),
            QueuedRun::running(2, 10),
            QueuedRun::queued(3, 10, 0),
            QueuedRun::queued(4, 20, 1),
        ];

        assert_eq!(
            select_to_start(&policy, runs),
            vec![RunId::new(Uuid::from_u128(4))]
        );
    }
}

#[cfg(test)]
mod preempts_at_boundary {
    use crate::ids::{AgentDefId, ProjectId, RunId, SessionId, TaskId};

    use serde_json::json;

    use super::{PreemptionController, PreemptionHandoff};
    use crate::runtime::session::{Run, RunStatus, Session, SessionStatus};

    #[test]
    fn test() {
        let session = Session {
            id: SessionId::generate(),
            project_id: ProjectId::generate(),
            agent_def_id: AgentDefId::generate(),
            name: "preempted work".into(),
            branch: "agent/preempted-work".into(),
            board_task_id: Some(TaskId::generate()),
            memory_base: json!({"decision": "keep the migration additive"}),
            pane_state: json!({}),
            status: SessionStatus::Active,
        };
        let mut run = Run {
            id: RunId::generate(),
            session_id: session.id,
            resolved_model_id: "test-model".into(),
            status: RunStatus::Running,
            permission_posture: Default::default(),
            events: vec![],
            usage: None,
            exit_code: None,
            cancel_reason: None,
            native_session_id: None,
            artifacts: vec![],
        };
        let handoff = PreemptionHandoff::from_session(&session);
        let mut preemption = PreemptionController::default();

        preemption
            .request(&run, handoff.clone())
            .expect("request preemption");
        assert_eq!(
            preemption
                .after_iteration(&mut run, false)
                .expect("do not preempt mid-iteration"),
            None
        );
        assert_eq!(run.status, RunStatus::Running);

        assert_eq!(
            preemption
                .after_iteration(&mut run, true)
                .expect("preempt at boundary"),
            Some(handoff)
        );
        assert_eq!(run.status, RunStatus::Paused);
    }
}

#[cfg(test)]
mod stop_all_restores {
    use crate::store::Store;

    use serde_json::json;
    use sqlx::{query, query_scalar};

    use super::*;
    use crate::store::backup::RetainedBackupConfig;

    async fn store() -> (Store, crate::testkit::postgres::DockerCleanup) {
        let (container, cleanup) =
            crate::testkit::postgres::start_postgres_named("locus-dispatch-test").await;
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
                &crate::testkit::postgres::NoopMigrationBackup,
                &RetainedBackupConfig::new(
                    "postgres://locus@localhost/locus",
                    "/var/lib/locus/artifacts",
                    "/var/lib/locus/backups",
                ),
            )
            .await
            .expect("run migrations");
        (store, cleanup)
    }

    #[test]
    fn priority_method_uses_longest_waiting_to_break_ties() {
        let policy = DispatchPolicy {
            global_parallelism: 1,
            per_project_parallelism: 1,
            priority_method: PriorityMethod::UnblocksMost,
            tie_break: TieBreak::LongestWaiting,
            preemption_enabled: false,
        };
        let mut older = QueuedRun::queued(1, 10, 0);
        older.priority.unblocks_count = 2;
        older.enqueued_at_ms = 1;
        let mut newer = QueuedRun::queued(2, 20, 0);
        newer.priority.unblocks_count = 2;
        newer.enqueued_at_ms = 2;

        assert_eq!(
            select_to_start(&policy, [newer, older]),
            vec![RunId::new(Uuid::from_u128(1))]
        );
    }

    #[test]
    fn stop_all_writes_handoffs_when_requested() {
        let snapshot = StopAllSnapshot {
            id: Uuid::new_v4(),
            run_ids: vec![RunId::generate()],
        };
        assert!(snapshot.preserves_durable_work());
        assert!(!snapshot.is_empty());
    }

    #[test]
    fn stop_all_immediate_without_handoffs() {
        let snapshot = StopAllSnapshot {
            id: Uuid::new_v4(),
            run_ids: Vec::new(),
        };
        assert!(snapshot.is_empty());
        assert!(snapshot.preserves_durable_work());
    }

    #[test]
    fn restore_stop_all_snapshot() {
        assert_eq!(StopAllSnapshot::restore_window_minutes(), 10);
    }

    #[tokio::test]
    async fn stop_all_restores() {
        let (store, _cleanup) = store().await;
        let project = ProjectId::generate();
        let agent = Uuid::new_v4();
        let queued_session = SessionId::generate();
        let running_session = SessionId::generate();
        let queued_run = RunId::generate();
        let running_run = RunId::generate();
        let workflow = Uuid::new_v4();
        let schedule = Uuid::new_v4();

        query("INSERT INTO core.projects (id, name) VALUES ($1, 'stop all')")
            .bind(project)
            .execute(store.pool())
            .await
            .expect("insert project");
        query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ($1, 'stop all test', 1, '{}'::jsonb, '')",
        )
        .bind(agent)
        .execute(store.pool())
        .await
        .expect("insert agent");
        query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch) VALUES
             ($1, $2, $3, 'queued', 'agent/queued'),
             ($4, $2, $3, 'running', 'agent/running')",
        )
        .bind(queued_session)
        .bind(project)
        .bind(agent)
        .bind(running_session)
        .execute(store.pool())
        .await
        .expect("insert sessions");
        query(
            "INSERT INTO agents.runs (id, session_id, resolved_model_id, status) VALUES
             ($1, $2, 'test-model', 'queued'),
             ($3, $4, 'test-model', 'running')",
        )
        .bind(queued_run)
        .bind(queued_session)
        .bind(running_run)
        .bind(running_session)
        .execute(store.pool())
        .await
        .expect("insert runs");
        query(
            "INSERT INTO workflows.workflow_defs
                 (id, project_id, name, version, graph, spec, verify_command)
             VALUES ($1, $2, 'stop all', 1, '{}'::jsonb, '{}'::jsonb, 'cargo test')",
        )
        .bind(workflow)
        .bind(project)
        .execute(store.pool())
        .await
        .expect("insert workflow");
        query(
            "INSERT INTO workflows.schedules (id, workflow_def_id, cron_expression)
             VALUES ($1, $2, '* * * * *')",
        )
        .bind(schedule)
        .bind(workflow)
        .execute(store.pool())
        .await
        .expect("insert schedule");
        store
            .set_project_autorun(project, true)
            .await
            .expect("enable autorun");

        let mut snapshot = store.stop_all().await.expect("stop all");
        snapshot.run_ids.sort();
        let mut expected_run_ids = vec![queued_run, running_run];
        expected_run_ids.sort();
        assert_eq!(snapshot.run_ids, expected_run_ids);
        assert_eq!(
            query_scalar::<_, String>("SELECT status FROM agents.runs WHERE id = $1")
                .bind(queued_run)
                .fetch_one(store.pool())
                .await
                .expect("read queued run"),
            "stopped"
        );
        assert_eq!(
            query_scalar::<_, String>("SELECT status FROM agents.runs WHERE id = $1")
                .bind(running_run)
                .fetch_one(store.pool())
                .await
                .expect("read running run"),
            "stopped"
        );
        assert!(!store.project_autorun(project).await.expect("read autorun"));
        assert!(query_scalar::<_, bool>(
            "SELECT paused_at IS NOT NULL FROM workflows.schedules WHERE id = $1",
        )
        .bind(schedule)
        .fetch_one(store.pool())
        .await
        .expect("read paused schedule"));

        store
            .restore_stop_all(snapshot.id)
            .await
            .expect("restore within ten minutes");
        assert_eq!(
            query_scalar::<_, String>("SELECT status FROM agents.runs WHERE id = $1")
                .bind(queued_run)
                .fetch_one(store.pool())
                .await
                .expect("read restored queued run"),
            "queued"
        );
        assert_eq!(
            query_scalar::<_, String>("SELECT status FROM agents.runs WHERE id = $1")
                .bind(running_run)
                .fetch_one(store.pool())
                .await
                .expect("read requeued running run"),
            "queued"
        );
        assert!(store
            .project_autorun(project)
            .await
            .expect("read restored autorun"));
        assert!(!query_scalar::<_, bool>(
            "SELECT paused_at IS NOT NULL FROM workflows.schedules WHERE id = $1",
        )
        .bind(schedule)
        .fetch_one(store.pool())
        .await
        .expect("read restored schedule"));
    }

    #[tokio::test]
    async fn restore_window_rejects_expired_snapshot() {
        let (store, _cleanup) = store().await;
        let snapshot = store.stop_all().await.expect("stop all");
        query(
            "UPDATE core.stop_all_snapshots
             SET stopped_at = now() - INTERVAL '11 minutes',
                 restore_expires_at = now() - INTERVAL '1 minute'
             WHERE id = $1",
        )
        .bind(snapshot.id)
        .execute(store.pool())
        .await
        .expect("expire snapshot");

        assert!(store.restore_stop_all(snapshot.id).await.is_err());
    }

    #[tokio::test]
    async fn durable_claim_respects_persisted_caps() {
        let (store, _cleanup) = store().await;
        let project_a = Uuid::new_v4();
        let project_b = Uuid::new_v4();
        let agent = Uuid::new_v4();
        query("INSERT INTO core.projects (id, name) VALUES ($1, 'a'), ($2, 'b')")
            .bind(project_a)
            .bind(project_b)
            .execute(store.pool())
            .await
            .expect("insert projects");
        query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ($1, 'dispatch test', 1, '{}'::jsonb, '')",
        )
        .bind(agent)
        .execute(store.pool())
        .await
        .expect("insert agent");

        let running_session = SessionId::generate();
        let blocked_session = SessionId::generate();
        let eligible_session = SessionId::generate();
        query(
            "INSERT INTO agents.sessions (id, project_id, agent_def_id, name, branch) VALUES
             ($1, $2, $4, 'running', 'agent/running'),
             ($3, $2, $4, 'blocked', 'agent/blocked'),
             ($5, $6, $4, 'eligible', 'agent/eligible')",
        )
        .bind(running_session)
        .bind(project_a)
        .bind(blocked_session)
        .bind(agent)
        .bind(eligible_session)
        .bind(project_b)
        .execute(store.pool())
        .await
        .expect("insert sessions");

        let running_run = RunId::generate();
        let blocked_run = RunId::generate();
        let eligible_run = RunId::generate();
        query(
            "INSERT INTO agents.runs (id, session_id, resolved_model_id, status) VALUES
             ($1, $2, 'test-model', 'running'),
             ($3, $4, 'test-model', 'queued'),
             ($5, $6, 'test-model', 'queued')",
        )
        .bind(running_run)
        .bind(running_session)
        .bind(blocked_run)
        .bind(blocked_session)
        .bind(eligible_run)
        .bind(eligible_session)
        .execute(store.pool())
        .await
        .expect("insert runs");

        let policy = DispatchPolicy {
            global_parallelism: 3,
            per_project_parallelism: 1,
            priority_method: PriorityMethod::PlanOrder,
            tie_break: TieBreak::LongestWaiting,
            preemption_enabled: false,
        };
        store
            .set_dispatch_policy(policy)
            .await
            .expect("persist policy");
        assert_eq!(store.dispatch_policy().await.expect("read policy"), policy);
        store
            .enqueue_dispatch(
                blocked_run,
                DispatchPriority {
                    plan_order: 0,
                    ..Default::default()
                },
            )
            .await
            .expect("enqueue project-a run");
        store
            .enqueue_dispatch(
                eligible_run,
                DispatchPriority {
                    plan_order: 1,
                    ..Default::default()
                },
            )
            .await
            .expect("enqueue project-b run");

        assert_eq!(
            store.claim_dispatchable_runs().await.expect("claim runs"),
            vec![eligible_run]
        );
        assert_eq!(
            query_scalar::<_, String>("SELECT status FROM agents.runs WHERE id = $1")
                .bind(blocked_run)
                .fetch_one(store.pool())
                .await
                .expect("read blocked status"),
            "queued"
        );
        assert_eq!(
            query_scalar::<_, String>("SELECT status FROM agents.runs WHERE id = $1")
                .bind(eligible_run)
                .fetch_one(store.pool())
                .await
                .expect("read eligible status"),
            "running"
        );
    }

    #[tokio::test]
    async fn preemption_handoff_is_durable_and_boundary_only() {
        let (store, _cleanup) = store().await;
        let project = ProjectId::generate();
        let agent = Uuid::new_v4();
        let session = SessionId::generate();
        let run = RunId::generate();
        let task = TaskId::generate();
        let memory_base = json!({"decision": "keep the migration additive"});
        query("INSERT INTO core.projects (id, name) VALUES ($1, 'preemption')")
            .bind(project)
            .execute(store.pool())
            .await
            .expect("insert project");
        query(
            "INSERT INTO agents.agent_defs (id, name, version, frontmatter, body)
             VALUES ($1, 'preemption test', 1, '{}'::jsonb, '')",
        )
        .bind(agent)
        .execute(store.pool())
        .await
        .expect("insert agent");
        query(
            "INSERT INTO agents.sessions
                 (id, project_id, agent_def_id, name, branch, board_task_id, memory_base)
             VALUES ($1, $2, $3, 'preempted', 'agent/preempted', $4, $5)",
        )
        .bind(session)
        .bind(project)
        .bind(agent)
        .bind(task)
        .bind(&memory_base)
        .execute(store.pool())
        .await
        .expect("insert session");
        query(
            "INSERT INTO agents.runs (id, session_id, resolved_model_id, status)
             VALUES ($1, $2, 'test-model', 'running')",
        )
        .bind(run)
        .bind(session)
        .execute(store.pool())
        .await
        .expect("insert running run");
        store
            .set_dispatch_policy(DispatchPolicy {
                global_parallelism: 1,
                per_project_parallelism: 1,
                priority_method: PriorityMethod::PlanOrder,
                tie_break: TieBreak::LongestWaiting,
                preemption_enabled: true,
            })
            .await
            .expect("enable boundary preemption");

        store
            .request_dispatch_preemption(run)
            .await
            .expect("persist preemption request");
        assert_eq!(
            store
                .preempt_dispatch_at_iteration_boundary(run)
                .await
                .expect("defer mid-iteration preemption"),
            None
        );
        assert_eq!(
            query_scalar::<_, String>("SELECT status FROM agents.runs WHERE id = $1")
                .bind(run)
                .fetch_one(store.pool())
                .await
                .expect("read mid-iteration run status"),
            "running"
        );
        assert_eq!(
            query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM agents.dispatch_preemptions WHERE run_id = $1",
            )
            .bind(run)
            .fetch_one(store.pool())
            .await
            .expect("read pending preemption"),
            1
        );

        let workflow = Uuid::new_v4();
        let execution = Uuid::new_v4();
        query(
            "INSERT INTO workflows.workflow_defs
                 (id, project_id, name, version, graph, spec, verify_command)
             VALUES ($1, $2, 'preemption', 1, '{}'::jsonb, '{}'::jsonb, 'true')",
        )
        .bind(workflow)
        .bind(project)
        .execute(store.pool())
        .await
        .expect("insert workflow");
        query(
            "INSERT INTO workflows.executions (id, workflow_def_id, status)
             VALUES ($1, $2, 'running')",
        )
        .bind(execution)
        .bind(workflow)
        .execute(store.pool())
        .await
        .expect("insert execution");
        query(
            "INSERT INTO workflows.iterations (id, execution_id, run_id, number, ended_at)
             VALUES ($1, $2, $3, 1, now())",
        )
        .bind(Uuid::new_v4())
        .bind(execution)
        .bind(run)
        .execute(store.pool())
        .await
        .expect("complete iteration");

        assert_eq!(
            store
                .preempt_dispatch_at_iteration_boundary(run)
                .await
                .expect("preempt at iteration boundary"),
            Some(PreemptionHandoff {
                session_id: session,
                branch: "agent/preempted".into(),
                board_task_id: Some(task),
                memory_base,
            })
        );
        assert_eq!(
            query_scalar::<_, String>("SELECT status FROM agents.runs WHERE id = $1")
                .bind(run)
                .fetch_one(store.pool())
                .await
                .expect("read preempted run status"),
            "paused"
        );
        assert_eq!(
            query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM agents.dispatch_preemptions WHERE run_id = $1",
            )
            .bind(run)
            .fetch_one(store.pool())
            .await
            .expect("read cleared preemption"),
            0
        );
    }
}
