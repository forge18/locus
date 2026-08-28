//! Persistent named agents that sit outside the board workflow.
//!
//! A bot owns one ordinary agent definition, one durable home session, and one persistent
//! `bots/<bot-id>` workspace branch.  Routines reuse the existing cron shape but target a prompt
//! and never create a queue backlog.

use crate::{
    ids::{AgentDefId, BotId, ProjectId, RoutineId, RunId, SessionId},
    services::schedule::CronExpression,
};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

pub const DEFAULT_WARM_WINDOW_MINUTES: u32 = 10;
pub const MAX_WARM_WINDOW_MINUTES: u32 = 24 * 60;
pub const BOT_BRANCH_PREFIX: &str = "bots/";

pub fn bot_branch(bot_id: BotId) -> String {
    format!("{BOT_BRANCH_PREFIX}{bot_id}")
}

pub fn validate_bot_name(name: &str) -> Result<()> {
    if name.trim().is_empty() || name.contains('\0') {
        bail!("bot name must not be empty")
    }
    Ok(())
}

/// Project-level idle duration after the last bot activity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WarmWindow {
    minutes: u32,
}

impl WarmWindow {
    pub fn new(minutes: u32) -> Result<Self> {
        if minutes > MAX_WARM_WINDOW_MINUTES {
            bail!("bot warm window cannot exceed {MAX_WARM_WINDOW_MINUTES} minutes")
        }
        Ok(Self { minutes })
    }

    pub const fn minutes(self) -> u32 {
        self.minutes
    }

    pub fn expires_at(self, last_activity: OffsetDateTime) -> OffsetDateTime {
        last_activity + Duration::minutes(i64::from(self.minutes))
    }

    pub fn is_expired(self, last_activity: OffsetDateTime, now: OffsetDateTime) -> bool {
        now >= self.expires_at(last_activity)
    }
}

impl Default for WarmWindow {
    fn default() -> Self {
        Self {
            minutes: DEFAULT_WARM_WINDOW_MINUTES,
        }
    }
}

/// Settings stored under the project settings aggregate.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BotSettings {
    #[serde(default = "default_warm_window_minutes")]
    pub warm_window_minutes: u32,
}

const fn default_warm_window_minutes() -> u32 {
    DEFAULT_WARM_WINDOW_MINUTES
}

impl BotSettings {
    pub fn new(warm_window_minutes: u32) -> Result<Self> {
        WarmWindow::new(warm_window_minutes)?;
        Ok(Self {
            warm_window_minutes,
        })
    }

    pub fn warm_window(&self) -> Result<WarmWindow> {
        WarmWindow::new(self.warm_window_minutes)
    }
}

impl Default for BotSettings {
    fn default() -> Self {
        Self {
            warm_window_minutes: DEFAULT_WARM_WINDOW_MINUTES,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotContainerState {
    Cold,
    Running,
    Warm,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Bot {
    pub id: BotId,
    pub project_id: ProjectId,
    pub name: String,
    pub agent_def_id: AgentDefId,
    pub home_session_id: SessionId,
    pub branch: String,
    pub container_id: Option<String>,
    pub container_state: BotContainerState,
    pub warm_until: Option<String>,
    pub last_activity_at: Option<String>,
    pub total_cost_micros: Option<u64>,
}

impl Bot {
    pub fn new(
        id: BotId,
        project_id: ProjectId,
        name: impl Into<String>,
        agent_def_id: AgentDefId,
        home_session_id: SessionId,
        warm_window: WarmWindow,
    ) -> Result<Self> {
        let name = name.into();
        validate_bot_name(&name)?;
        let _ = warm_window;
        Ok(Self {
            id,
            project_id,
            name,
            agent_def_id,
            home_session_id,
            branch: bot_branch(id),
            container_id: None,
            container_state: BotContainerState::Cold,
            warm_until: None,
            last_activity_at: None,
            total_cost_micros: None,
        })
    }

    pub fn is_persistent_branch(&self) -> bool {
        self.branch == bot_branch(self.id) && !matches!(self.branch.as_str(), "main" | "master")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BotLifecycle {
    pub bot_id: BotId,
    pub session_id: SessionId,
    pub branch: String,
    pub warm_window: WarmWindow,
    state: BotContainerState,
    container_id: Option<String>,
    last_activity: Option<OffsetDateTime>,
    warm_until: Option<OffsetDateTime>,
    active_run: Option<RunId>,
    has_history: bool,
}

impl BotLifecycle {
    pub fn new(bot_id: BotId, session_id: SessionId, warm_window: WarmWindow) -> Self {
        Self {
            bot_id,
            session_id,
            branch: bot_branch(bot_id),
            warm_window,
            state: BotContainerState::Cold,
            container_id: None,
            last_activity: None,
            warm_until: None,
            active_run: None,
            has_history: false,
        }
    }

    pub fn state(&self) -> BotContainerState {
        self.state
    }

    pub fn active_run(&self) -> Option<RunId> {
        self.active_run
    }

    pub fn warm_until(&self) -> Option<OffsetDateTime> {
        self.warm_until
    }

    pub fn receive_message(
        &mut self,
        run_id: RunId,
        container_id: impl Into<String>,
        now: OffsetDateTime,
    ) -> BotMessageAction {
        let expired_container_id = self.expire(now).and_then(|action| action.container_id);
        if let Some(active_run) = self.active_run {
            self.record_activity(now);
            return BotMessageAction::DeliverToRun { run_id: active_run };
        }
        let resume = self.has_history;
        self.state = BotContainerState::Running;
        self.container_id = Some(container_id.into());
        self.active_run = Some(run_id);
        self.last_activity = Some(now);
        self.warm_until = None;
        BotMessageAction::BootHomeSession {
            run_id,
            session_id: self.session_id,
            branch: self.branch.clone(),
            stop_container: expired_container_id,
            resume,
        }
    }

    pub fn finish_run(&mut self, run_id: RunId, now: OffsetDateTime) -> Result<WarmStopDeadline> {
        if self.active_run != Some(run_id) {
            bail!("run is not the active bot run")
        }
        self.active_run = None;
        self.state = BotContainerState::Warm;
        self.has_history = true;
        self.last_activity = Some(now);
        let warm_until = self.warm_window.expires_at(now);
        self.warm_until = Some(warm_until);
        Ok(WarmStopDeadline {
            bot_id: self.bot_id,
            container_id: self.container_id.clone(),
            warm_until,
        })
    }

    pub fn record_activity(&mut self, now: OffsetDateTime) {
        self.last_activity = Some(now);
        if self.state == BotContainerState::Warm {
            self.warm_until = Some(self.warm_window.expires_at(now));
        }
    }

    pub fn expire(&mut self, now: OffsetDateTime) -> Option<WarmStopAction> {
        let expires = self.warm_until?;
        if now < expires {
            return None;
        }
        self.state = BotContainerState::Cold;
        self.warm_until = None;
        self.active_run = None;
        Some(WarmStopAction {
            bot_id: self.bot_id,
            container_id: self.container_id.take(),
        })
    }

    pub fn warm_stopped(&self) -> bool {
        self.state == BotContainerState::Cold && self.has_history
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BotMessageAction {
    DeliverToRun {
        run_id: RunId,
    },
    BootHomeSession {
        run_id: RunId,
        session_id: SessionId,
        branch: String,
        stop_container: Option<String>,
        resume: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WarmStopDeadline {
    pub bot_id: BotId,
    pub container_id: Option<String>,
    pub warm_until: OffsetDateTime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WarmStopAction {
    pub bot_id: BotId,
    pub container_id: Option<String>,
}

/// A warm-stopped bot is expected during reconciliation, unlike an ordinary lost run.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BotReconciliation {
    pub container_alive: bool,
    pub warm_stopped: bool,
    pub expected: bool,
    pub file_aborted_inbox_item: bool,
}

pub fn reconcile_warm_stop(container_alive: bool, warm_stopped: bool) -> BotReconciliation {
    BotReconciliation {
        container_alive,
        warm_stopped,
        expected: container_alive || warm_stopped,
        file_aborted_inbox_item: !container_alive && !warm_stopped,
    }
}

pub fn warm_stop_is_expected(container_alive: bool, warm_stopped: bool) -> bool {
    reconcile_warm_stop(container_alive, warm_stopped).expected
}

/// A routine prompt is attributed in the same home conversation as ad-hoc messages.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RoutineAttribution {
    RoutineFired,
    TestRun,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RoutineResult {
    pub passed: bool,
    pub summary: String,
}

impl RoutineResult {
    pub fn passed(summary: impl Into<String>) -> Self {
        Self {
            passed: true,
            summary: summary.into(),
        }
    }

    pub fn failed(summary: impl Into<String>) -> Self {
        Self {
            passed: false,
            summary: summary.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutineExecutionStatus {
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RoutineExecution {
    pub id: RoutineId,
    pub bot_id: BotId,
    pub scheduled_for: i64,
    pub status: RoutineExecutionStatus,
    pub result: Option<RoutineResult>,
    pub attribution: RoutineAttribution,
    pub test_run: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BotRoutine {
    pub id: RoutineId,
    pub bot_id: BotId,
    pub prompt: String,
    pub cron_expression: String,
    pub enabled: bool,
    pub skipped_count: u32,
    pub schedule_id: Option<Uuid>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutineClaim {
    pub execution_id: RoutineId,
    pub bot_id: BotId,
    pub prompt: String,
    pub attribution: RoutineAttribution,
    pub test_run: bool,
    pub headless: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoutineClaimResult {
    Started(RoutineClaim),
    Skipped {
        execution_id: RoutineId,
        skip_count: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BotRunStart {
    pub run_id: RunId,
    pub session_id: SessionId,
    pub definition_id: AgentDefId,
    pub branch: String,
    pub container_id: String,
    pub stop_container: Option<String>,
    pub resume: bool,
    pub reused_container: bool,
    pub headless: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutineStart {
    pub execution_id: RoutineId,
    pub bot_id: BotId,
    pub prompt: String,
    pub attribution: RoutineAttribution,
    pub test_run: bool,
    pub headless: bool,
    pub boot_home_session: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoutineFire {
    Started(RoutineStart),
    Skipped {
        execution_id: RoutineId,
        scheduled_for: i64,
        skip_count: u32,
    },
}

/// Deterministic routine controller. Persistence and the daemon supply the actual run.
#[derive(Clone, Debug)]
pub struct RoutineController {
    id: RoutineId,
    bot_id: BotId,
    prompt: String,
    cron: CronExpression,
    enabled: bool,
    active_execution: Option<RoutineId>,
    executions: Vec<RoutineExecution>,
}

impl RoutineController {
    pub fn new(
        id: RoutineId,
        bot_id: BotId,
        prompt: impl Into<String>,
        cron_expression: &str,
    ) -> Result<Self> {
        let prompt = prompt.into();
        if prompt.trim().is_empty() {
            bail!("routine prompt must not be empty")
        }
        Ok(Self {
            id,
            bot_id,
            prompt,
            cron: CronExpression::parse(cron_expression).map_err(|error| anyhow::anyhow!(error))?,
            enabled: true,
            active_execution: None,
            executions: Vec::new(),
        })
    }

    pub fn id(&self) -> RoutineId {
        self.id
    }

    pub fn bot_id(&self) -> BotId {
        self.bot_id
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    pub fn cron_expression(&self) -> &str {
        self.cron.source()
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn edit(&mut self, prompt: impl Into<String>, cron_expression: &str) -> Result<()> {
        let prompt = prompt.into();
        if prompt.trim().is_empty() {
            bail!("routine prompt must not be empty")
        }
        let cron =
            CronExpression::parse(cron_expression).map_err(|error| anyhow::anyhow!(error))?;
        self.prompt = prompt;
        self.cron = cron;
        Ok(())
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn active_execution(&self) -> Option<RoutineId> {
        self.active_execution
    }

    pub fn skipped_count(&self) -> u32 {
        self.executions
            .iter()
            .filter(|execution| execution.status == RoutineExecutionStatus::Skipped)
            .count() as u32
    }

    pub fn executions(&self) -> &[RoutineExecution] {
        &self.executions
    }

    pub fn fire(&mut self, scheduled_for: OffsetDateTime, bot_running: bool) -> RoutineFire {
        let scheduled_for = scheduled_for.unix_timestamp();
        if !self.enabled || bot_running || self.active_execution.is_some() {
            let execution_id = RoutineId::generate();
            self.executions.push(RoutineExecution {
                id: execution_id,
                bot_id: self.bot_id,
                scheduled_for,
                status: RoutineExecutionStatus::Skipped,
                result: None,
                attribution: RoutineAttribution::RoutineFired,
                test_run: false,
            });
            return RoutineFire::Skipped {
                execution_id,
                scheduled_for,
                skip_count: self.skipped_count(),
            };
        }
        let execution_id = RoutineId::generate();
        self.active_execution = Some(execution_id);
        self.executions.push(RoutineExecution {
            id: execution_id,
            bot_id: self.bot_id,
            scheduled_for,
            status: RoutineExecutionStatus::Running,
            result: None,
            attribution: RoutineAttribution::RoutineFired,
            test_run: false,
        });
        RoutineFire::Started(RoutineStart {
            execution_id,
            bot_id: self.bot_id,
            prompt: self.prompt.clone(),
            attribution: RoutineAttribution::RoutineFired,
            test_run: false,
            headless: true,
            boot_home_session: !bot_running,
        })
    }

    /// A test run does not enable, pause, edit, or otherwise mutate the routine schedule.
    pub fn test_run(&self) -> RoutineStart {
        RoutineStart {
            execution_id: RoutineId::generate(),
            bot_id: self.bot_id,
            prompt: self.prompt.clone(),
            attribution: RoutineAttribution::TestRun,
            test_run: true,
            headless: true,
            boot_home_session: true,
        }
    }

    pub fn complete(&mut self, execution_id: RoutineId, result: RoutineResult) -> Result<()> {
        if self.active_execution != Some(execution_id) {
            bail!("routine execution is not active")
        }
        self.active_execution = None;
        let execution = self
            .executions
            .iter_mut()
            .rev()
            .find(|execution| execution.id == execution_id)
            .ok_or_else(|| anyhow::anyhow!("routine execution was not recorded"))?;
        execution.status = if result.passed {
            RoutineExecutionStatus::Completed
        } else {
            RoutineExecutionStatus::Failed
        };
        execution.result = Some(result);
        Ok(())
    }
}

/// Missing cost remains unknown; known run costs are summed across the home session.
pub fn sum_run_costs(costs: impl IntoIterator<Item = Option<u64>>) -> Option<u64> {
    let mut total = 0u64;
    let mut known = false;
    for cost in costs.into_iter().flatten() {
        total = total.saturating_add(cost);
        known = true;
    }
    known.then_some(total)
}

/// A compact normalized prompt marker for the shared conversation/event stream.
pub fn routine_attribution_payload(
    routine_id: RoutineId,
    attribution: RoutineAttribution,
    test_run: bool,
) -> serde_json::Value {
    serde_json::json!({
        "routineId": routine_id,
        "attribution": attribution,
        "testRun": test_run,
    })
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod bots {
    use super::*;

    fn at(unix: i64) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(unix).expect("timestamp")
    }

    #[test]
    fn table() {
        let bot_id = BotId::generate();
        let bot = Bot::new(
            bot_id,
            ProjectId::generate(),
            "keeper",
            AgentDefId::generate(),
            SessionId::generate(),
            WarmWindow::default(),
        )
        .expect("valid bot");
        assert_eq!(bot.branch, format!("bots/{bot_id}"));
        assert!(bot.is_persistent_branch());
        assert_eq!(WarmWindow::default().minutes(), 10);
    }

    #[test]
    fn create() {
        let error = validate_bot_name("").expect_err("empty names are refused");
        assert!(error.to_string().contains("bot name"));
        let error = validate_bot_name("bad\0name").expect_err("NUL is refused");
        assert!(error.to_string().contains("bot name"));
    }

    #[test]
    fn home_session() {
        let bot_id = BotId::generate();
        let session_id = SessionId::generate();
        let run = RunId::generate();
        let mut lifecycle = BotLifecycle::new(bot_id, session_id, WarmWindow::default());
        assert_eq!(
            lifecycle.receive_message(run, "container-1", at(1_700_000_000)),
            BotMessageAction::BootHomeSession {
                run_id: run,
                session_id,
                branch: bot_branch(bot_id),
                stop_container: None,
                resume: false,
            }
        );
    }

    #[test]
    fn resumes_across_runs() {
        let bot_id = BotId::generate();
        let session_id = SessionId::generate();
        let mut lifecycle = BotLifecycle::new(bot_id, session_id, WarmWindow::default());
        let first = RunId::generate();
        let second = RunId::generate();
        lifecycle.receive_message(first, "container-1", at(1_700_000_000));
        lifecycle.finish_run(first, at(1_700_000_001)).unwrap();
        assert!(matches!(
            lifecycle.receive_message(second, "container-2", at(1_700_000_002)),
            BotMessageAction::BootHomeSession { resume: true, .. }
        ));
        assert_eq!(lifecycle.branch, bot_branch(bot_id));
    }

    #[test]
    fn warm_expiry() {
        let bot_id = BotId::generate();
        let mut lifecycle =
            BotLifecycle::new(bot_id, SessionId::generate(), WarmWindow::new(10).unwrap());
        let run = RunId::generate();
        lifecycle.receive_message(run, "container-1", at(1_700_000_000));
        let deadline = lifecycle.finish_run(run, at(1_700_000_000)).unwrap();
        assert_eq!(deadline.warm_until, at(1_700_000_600));
        assert!(lifecycle.expire(at(1_700_000_599)).is_none());
        assert!(lifecycle.expire(at(1_700_000_600)).is_some());
    }

    #[test]
    fn warm_resume() {
        let bot_id = BotId::generate();
        let mut lifecycle =
            BotLifecycle::new(bot_id, SessionId::generate(), WarmWindow::new(1).unwrap());
        let first = RunId::generate();
        let second = RunId::generate();
        lifecycle.receive_message(first, "container-1", at(1_700_000_000));
        lifecycle.finish_run(first, at(1_700_000_000)).unwrap();
        assert!(matches!(
            lifecycle.receive_message(second, "container-2", at(1_700_000_061)),
            BotMessageAction::BootHomeSession {
                resume: true,
                stop_container: Some(ref container),
                ..
            } if container == "container-1"
        ));
    }

    #[test]
    fn reconcile_warm_stopped_container() {
        let reconciliation = super::reconcile_warm_stop(false, true);
        assert!(reconciliation.expected);
        assert!(!reconciliation.file_aborted_inbox_item);
        assert!(!super::reconcile_warm_stop(false, false).expected);
    }

    #[test]
    fn routine_target() {
        let routine = RoutineController::new(
            RoutineId::generate(),
            BotId::generate(),
            "check the repository",
            "0 * * * *",
        )
        .expect("valid routine");
        assert_eq!(routine.cron_expression(), "0 * * * *");
        assert_eq!(routine.prompt(), "check the repository");
    }

    #[test]
    fn routine_fires_headless() {
        let mut routine = RoutineController::new(
            RoutineId::generate(),
            BotId::generate(),
            "check",
            "0 * * * *",
        )
        .unwrap();
        let RoutineFire::Started(start) = routine.fire(at(1_700_000_000), false) else {
            panic!("routine should start")
        };
        assert!(start.headless);
        assert!(start.boot_home_session);
    }

    #[test]
    fn routine_records_result() {
        let mut routine = RoutineController::new(
            RoutineId::generate(),
            BotId::generate(),
            "check",
            "0 * * * *",
        )
        .unwrap();
        let RoutineFire::Started(start) = routine.fire(at(1_700_000_000), false) else {
            panic!("routine should start")
        };
        routine
            .complete(start.execution_id, RoutineResult::passed("green"))
            .unwrap();
        assert_eq!(
            routine.executions()[0].status,
            RoutineExecutionStatus::Completed
        );
        assert_eq!(
            routine.executions()[0].result.as_ref().unwrap().summary,
            "green"
        );
    }

    #[test]
    fn routine_skips_never_queues() {
        let mut routine = RoutineController::new(
            RoutineId::generate(),
            BotId::generate(),
            "check",
            "0 * * * *",
        )
        .unwrap();
        let RoutineFire::Started(start) = routine.fire(at(1_700_000_000), false) else {
            panic!("routine should start")
        };
        let RoutineFire::Skipped { skip_count, .. } = routine.fire(at(1_700_000_060), true) else {
            panic!("overlap should skip")
        };
        assert_eq!(skip_count, 1);
        assert!(routine.active_execution().is_some());
        routine
            .complete(start.execution_id, RoutineResult::passed("done"))
            .unwrap();
    }

    #[test]
    fn routine_attribution() {
        let routine_id = RoutineId::generate();
        let payload =
            routine_attribution_payload(routine_id, RoutineAttribution::RoutineFired, false);
        assert_eq!(payload["attribution"], "routine-fired");
        assert_eq!(payload["routineId"], routine_id.to_string());
    }

    #[test]
    fn routine_lifecycle() {
        let mut routine = RoutineController::new(
            RoutineId::generate(),
            BotId::generate(),
            "check",
            "0 * * * *",
        )
        .unwrap();
        routine.set_enabled(false);
        assert!(matches!(
            routine.fire(at(1_700_000_000), false),
            RoutineFire::Skipped { .. }
        ));
        routine.set_enabled(true);
        routine.edit("updated", "*/5 * * * *").unwrap();
        assert_eq!(routine.prompt(), "updated");
        assert_eq!(routine.cron_expression(), "*/5 * * * *");
        let test = routine.test_run();
        assert!(test.test_run);
        assert!(routine.enabled());
    }

    #[test]
    fn definition_version_per_run() {
        let first = AgentDefId::generate();
        let second = AgentDefId::generate();
        assert_ne!(first, second);
        let bot = Bot::new(
            BotId::generate(),
            ProjectId::generate(),
            "versioned",
            first,
            SessionId::generate(),
            WarmWindow::default(),
        )
        .unwrap();
        assert_eq!(bot.agent_def_id, first);
    }

    #[test]
    fn sum_costs() {
        assert_eq!(sum_run_costs([Some(10), Some(20)]), Some(30));
        assert_eq!(sum_run_costs([None, None]), None);
    }
}
