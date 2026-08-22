//! Durable queue policy for deciding which queued agent runs may start.
//!
//! The queue never displaces a running run. Task 22 adds boundary-only preemption;
//! task 23 owns global stopping and restore.

use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use uuid::Uuid;

use crate::store::Store;

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
    fn as_str(self) -> &'static str {
        match self {
            Self::PlanOrder => "plan_order",
            Self::Manual => "manual",
            Self::UnblocksMost => "unblocks_most",
            Self::ShortestFirst => "shortest_first",
        }
    }

    fn parse(value: &str) -> Result<Self> {
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
    fn as_str(self) -> &'static str {
        "longest_waiting"
    }

    fn parse(value: &str) -> Result<Self> {
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
}

impl Default for DispatchPolicy {
    fn default() -> Self {
        Self {
            global_parallelism: 6,
            per_project_parallelism: 3,
            priority_method: PriorityMethod::PlanOrder,
            tie_break: TieBreak::LongestWaiting,
        }
    }
}

impl DispatchPolicy {
    fn validate(self) -> Result<()> {
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

/// A supervisor-visible run used to apply capacity and priority rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueuedRun {
    pub run_id: Uuid,
    pub project_id: Uuid,
    pub state: RunState,
    pub priority: DispatchPriority,
    /// Milliseconds since the Unix epoch. Earlier values waited longer.
    pub enqueued_at_ms: i64,
}

impl QueuedRun {
    #[cfg(test)]
    fn running(run_id: u128, project_id: u128) -> Self {
        Self {
            run_id: Uuid::from_u128(run_id),
            project_id: Uuid::from_u128(project_id),
            state: RunState::Running,
            priority: DispatchPriority::default(),
            enqueued_at_ms: 0,
        }
    }

    #[cfg(test)]
    fn queued(run_id: u128, project_id: u128, plan_order: i64) -> Self {
        Self {
            run_id: Uuid::from_u128(run_id),
            project_id: Uuid::from_u128(project_id),
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

/// Select queued runs that fit both caps, without interrupting any running run.
pub fn select_to_start(
    policy: &DispatchPolicy,
    runs: impl IntoIterator<Item = QueuedRun>,
) -> Vec<Uuid> {
    let mut runs = runs.into_iter().collect::<Vec<_>>();
    let mut global_running = 0_u32;
    let mut project_running = BTreeMap::<Uuid, u32>::new();

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

impl Store {
    /// Read the durable, machine-wide dispatch policy.
    pub async fn dispatch_policy(&self) -> Result<DispatchPolicy> {
        let row = query(
            "SELECT global_parallelism, per_project_parallelism, priority_method, tie_break
             FROM core.dispatch_policy
             WHERE singleton = TRUE",
        )
        .fetch_one(self.pool())
        .await
        .context("read dispatch policy")?;
        policy_from_row(&row)
    }

    /// Replace the durable dispatch policy after validating both caps.
    pub async fn set_dispatch_policy(&self, policy: DispatchPolicy) -> Result<()> {
        policy.validate()?;
        query(
            "INSERT INTO core.dispatch_policy (
                 singleton, global_parallelism, per_project_parallelism, priority_method, tie_break
             ) VALUES (TRUE, $1, $2, $3, $4)
             ON CONFLICT (singleton) DO UPDATE SET
                 global_parallelism = EXCLUDED.global_parallelism,
                 per_project_parallelism = EXCLUDED.per_project_parallelism,
                 priority_method = EXCLUDED.priority_method,
                 tie_break = EXCLUDED.tie_break",
        )
        .bind(i32::try_from(policy.global_parallelism).context("global parallelism exceeds i32")?)
        .bind(
            i32::try_from(policy.per_project_parallelism)
                .context("per-project parallelism exceeds i32")?,
        )
        .bind(policy.priority_method.as_str())
        .bind(policy.tie_break.as_str())
        .execute(self.pool())
        .await
        .context("persist dispatch policy")?;
        Ok(())
    }

    /// Add a queued run to the durable dispatch queue. Its project is derived from the run session.
    pub async fn enqueue_dispatch(&self, run_id: Uuid, priority: DispatchPriority) -> Result<()> {
        let inserted = query(
            "INSERT INTO agents.dispatch_queue (
                 run_id, plan_order, manual_order, unblocks_count, estimate_minutes
             )
             SELECT $1, $2, $3, $4, $5
             FROM agents.runs
             WHERE id = $1 AND status = 'queued'
             ON CONFLICT (run_id) DO UPDATE SET
                 plan_order = EXCLUDED.plan_order,
                 manual_order = EXCLUDED.manual_order,
                 unblocks_count = EXCLUDED.unblocks_count,
                 estimate_minutes = EXCLUDED.estimate_minutes",
        )
        .bind(run_id)
        .bind(priority.plan_order)
        .bind(priority.manual_order)
        .bind(i32::try_from(priority.unblocks_count).context("unblocks count exceeds i32")?)
        .bind(i32::try_from(priority.estimate_minutes).context("estimate exceeds i32")?)
        .execute(self.pool())
        .await
        .context("enqueue dispatch run")?;
        if inserted.rows_affected() != 1 {
            bail!("only queued runs may enter the dispatch queue")
        }
        Ok(())
    }

    /// Atomically mark cap-eligible queued runs running, returning them in priority order.
    ///
    /// Locking the single policy row serializes claims across supervisor processes. This task only
    /// queues and starts work; it intentionally does not preempt active runs.
    pub async fn claim_dispatchable_runs(&self) -> Result<Vec<Uuid>> {
        let mut transaction = self.pool().begin().await.context("begin dispatch claim")?;
        let policy_row = query(
            "SELECT global_parallelism, per_project_parallelism, priority_method, tie_break
             FROM core.dispatch_policy
             WHERE singleton = TRUE
             FOR UPDATE",
        )
        .fetch_one(&mut *transaction)
        .await
        .context("lock dispatch policy")?;
        let policy = policy_from_row(&policy_row)?;

        let rows = query(
            "SELECT
                 runs.id AS run_id,
                 runs.status,
                 sessions.project_id,
                 COALESCE(queue.plan_order, 0) AS plan_order,
                 COALESCE(queue.manual_order, 0) AS manual_order,
                 COALESCE(queue.unblocks_count, 0) AS unblocks_count,
                 COALESCE(queue.estimate_minutes, 0) AS estimate_minutes,
                 COALESCE(
                     (EXTRACT(EPOCH FROM queue.enqueued_at) * 1000)::bigint,
                     0
                 ) AS enqueued_at_ms
             FROM agents.runs AS runs
             JOIN agents.sessions AS sessions ON sessions.id = runs.session_id
             LEFT JOIN agents.dispatch_queue AS queue ON queue.run_id = runs.id
             WHERE runs.status = 'running'
                OR (runs.status = 'queued' AND queue.run_id IS NOT NULL)
             FOR UPDATE OF runs SKIP LOCKED",
        )
        .fetch_all(&mut *transaction)
        .await
        .context("lock dispatch runs")?;
        let candidates = rows
            .iter()
            .map(queued_run_from_row)
            .collect::<Result<Vec<_>>>()?;
        let selected = select_to_start(&policy, candidates);

        for run_id in &selected {
            let changed = query(
                "UPDATE agents.runs
                 SET status = 'running', started_at = COALESCE(started_at, now())
                 WHERE id = $1 AND status = 'queued'",
            )
            .bind(run_id)
            .execute(&mut *transaction)
            .await
            .context("claim dispatch run")?;
            if changed.rows_affected() != 1 {
                bail!("dispatch run `{run_id}` was no longer queued")
            }
        }
        transaction
            .commit()
            .await
            .context("commit dispatch claim")?;
        Ok(selected)
    }
}

fn policy_from_row(row: &sqlx::postgres::PgRow) -> Result<DispatchPolicy> {
    let global_parallelism: i32 = row.try_get("global_parallelism")?;
    let per_project_parallelism: i32 = row.try_get("per_project_parallelism")?;
    let policy = DispatchPolicy {
        global_parallelism: u32::try_from(global_parallelism)
            .context("stored global parallelism is negative")?,
        per_project_parallelism: u32::try_from(per_project_parallelism)
            .context("stored per-project parallelism is negative")?,
        priority_method: PriorityMethod::parse(row.try_get::<&str, _>("priority_method")?)?,
        tie_break: TieBreak::parse(row.try_get::<&str, _>("tie_break")?)?,
    };
    policy.validate()?;
    Ok(policy)
}

fn queued_run_from_row(row: &sqlx::postgres::PgRow) -> Result<QueuedRun> {
    Ok(QueuedRun {
        run_id: row.try_get("run_id")?,
        project_id: row.try_get("project_id")?,
        state: match row.try_get::<&str, _>("status")? {
            "queued" => RunState::Queued,
            "running" => RunState::Running,
            state => bail!("unexpected dispatch run status `{state}`"),
        },
        priority: DispatchPriority {
            plan_order: row.try_get("plan_order")?,
            manual_order: row.try_get("manual_order")?,
            unblocks_count: u32::try_from(row.try_get::<i32, _>("unblocks_count")?)
                .context("stored unblocks count is negative")?,
            estimate_minutes: u32::try_from(row.try_get::<i32, _>("estimate_minutes")?)
                .context("stored estimate is negative")?,
        },
        enqueued_at_ms: row.try_get("enqueued_at_ms")?,
    })
}

#[cfg(test)]
mod enforces_parallel_caps {
    use super::*;

    #[test]
    fn test() {
        let policy = DispatchPolicy {
            global_parallelism: 3,
            per_project_parallelism: 2,
            priority_method: PriorityMethod::PlanOrder,
            tie_break: TieBreak::LongestWaiting,
        };
        let runs = vec![
            QueuedRun::running(1, 10),
            QueuedRun::running(2, 10),
            QueuedRun::queued(3, 10, 0),
            QueuedRun::queued(4, 20, 1),
        ];

        assert_eq!(select_to_start(&policy, runs), vec![Uuid::from_u128(4)]);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
    };

    use sqlx::{query, query_scalar};

    use super::*;
    use crate::{
        backup::{MigrationBackup, RetainedBackupConfig},
        store::{PostgresConfig, PostgresContainer},
    };

    struct NoopMigrationBackup;

    impl MigrationBackup for NoopMigrationBackup {
        fn create_retained(&self, _: &RetainedBackupConfig) -> Result<()> {
            Ok(())
        }
    }

    struct DockerCleanup {
        container_name: String,
        volume_name: String,
    }

    impl Drop for DockerCleanup {
        fn drop(&mut self) {
            let _ = Command::new("docker")
                .args(["rm", "--force", &self.container_name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let _ = Command::new("docker")
                .args(["volume", "rm", "--force", &self.volume_name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }

    fn unused_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind unused local port");
        listener.local_addr().expect("read local port").port()
    }

    async fn store() -> (Store, DockerCleanup) {
        let port = unused_port();
        let suffix = format!("{}-{port}", std::process::id());
        let container_name = format!("locus-dispatch-test-{suffix}");
        let volume_name = format!("locus-dispatch-test-data-{suffix}");
        let cleanup = DockerCleanup {
            container_name: container_name.clone(),
            volume_name: volume_name.clone(),
        };
        let container =
            PostgresContainer::new(PostgresConfig::for_test(container_name, volume_name, port));
        container.start().await.expect("start PostgreSQL");
        let store = Store::connect(&container.database_url())
            .await
            .expect("connect store");
        store
            .run_migrations(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
                &NoopMigrationBackup,
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
        };
        let mut older = QueuedRun::queued(1, 10, 0);
        older.priority.unblocks_count = 2;
        older.enqueued_at_ms = 1;
        let mut newer = QueuedRun::queued(2, 20, 0);
        newer.priority.unblocks_count = 2;
        newer.enqueued_at_ms = 2;

        assert_eq!(
            select_to_start(&policy, [newer, older]),
            vec![Uuid::from_u128(1)]
        );
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

        let running_session = Uuid::new_v4();
        let blocked_session = Uuid::new_v4();
        let eligible_session = Uuid::new_v4();
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

        let running_run = Uuid::new_v4();
        let blocked_run = Uuid::new_v4();
        let eligible_run = Uuid::new_v4();
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
}
