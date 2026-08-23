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

use crate::runtime::session::{Run, RunStatus, Session};

/// Per-project durable autorun posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutorunState(bool);

impl AutorunState {
    pub fn enabled() -> Self {
        Self(true)
    }
    pub fn disabled() -> Self {
        Self(false)
    }
    pub fn is_enabled(self) -> bool {
        self.0
    }
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
