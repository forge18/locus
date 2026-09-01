//! Persistence for dispatch policy and the run queue (`core.dispatch_policy`, `agents.dispatch_queue`).
//!
//! Moved out of `runtime/dispatch.rs` so every query in the crate lives under `store/`.

use crate::ids::{ProjectId, RunId};
use anyhow::{bail, Context, Result};
use serde_json::Value;
use sqlx::{query, query_as, Row};
use uuid::Uuid;

use crate::{
    runtime::dispatch::{
        select_to_start, DispatchPolicy, DispatchPriority, GuardrailDefaults, NetworkTier,
        PreemptionHandoff, PriorityMethod, ProjectAutorunPolicy, QueuedRun, RunState,
        StopAllSnapshot, TieBreak,
    },
    store::Store,
};

/// One dispatch schedule: a named cron that fires a workflow run.
#[derive(Debug, sqlx::FromRow)]
pub struct ScheduleRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub project: String,
    pub name: String,
    pub cron_expression: String,
    pub enabled: bool,
}

/// One schedule-execution row with its workflow name — the history list.
#[derive(Debug, sqlx::FromRow)]
pub struct ScheduleExecutionRow {
    pub id: Uuid,
    pub schedule_name: String,
    pub project: String,
    pub status: String,
    pub scheduled_for: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

/// One project's tri-state autorun posture with its project name.
#[derive(Debug, sqlx::FromRow)]
pub struct AutorunStateRow {
    pub project_id: Uuid,
    pub project: String,
    pub state: String,
}

/// The durable inputs needed to launch one claimed queue entry. The supervisor
/// resolves all remaining runtime policy from this row and the project settings;
/// it never trusts a caller-supplied project or agent definition.
#[derive(Clone, Debug)]
pub struct DispatchRun {
    pub run_id: Uuid,
    pub session_id: Uuid,
    pub project_id: Uuid,
    pub agent_def_id: Uuid,
    pub resolved_model_id: String,
    pub status: String,
    pub permission_posture: String,
    pub branch: String,
    pub board_task_id: Option<Uuid>,
    pub memory_base: Value,
    pub agent_name: String,
    pub agent_frontmatter: Value,
    pub agent_body: String,
    pub harness: Option<String>,
    pub workspace_remote: Option<String>,
}

#[derive(sqlx::FromRow)]
struct DispatchRunRow {
    run_id: Uuid,
    session_id: Uuid,
    project_id: Uuid,
    agent_def_id: Uuid,
    resolved_model_id: String,
    status: String,
    permission_posture: String,
    branch: String,
    board_task_id: Option<Uuid>,
    memory_base: Value,
    agent_name: String,
    agent_frontmatter: Value,
    agent_body: String,
    harness: Option<String>,
    workspace_remote: Option<String>,
}

impl From<DispatchRunRow> for DispatchRun {
    fn from(row: DispatchRunRow) -> Self {
        Self {
            run_id: row.run_id,
            session_id: row.session_id,
            project_id: row.project_id,
            agent_def_id: row.agent_def_id,
            resolved_model_id: row.resolved_model_id,
            status: row.status,
            permission_posture: row.permission_posture,
            branch: row.branch,
            board_task_id: row.board_task_id,
            memory_base: row.memory_base,
            agent_name: row.agent_name,
            agent_frontmatter: row.agent_frontmatter,
            agent_body: row.agent_body,
            harness: row.harness,
            workspace_remote: row.workspace_remote,
        }
    }
}

impl Store {
    /// Load the complete host-owned launch context for one claimed run.
    pub async fn dispatch_run(&self, run_id: RunId) -> Result<Option<DispatchRun>> {
        query_as::<_, DispatchRunRow>(
            "SELECT runs.id AS run_id, runs.session_id, sessions.project_id,
                    COALESCE(runs.agent_def_id, sessions.agent_def_id) AS agent_def_id,
                    runs.resolved_model_id, runs.status, runs.permission_posture, sessions.branch,
                    sessions.board_task_id, sessions.memory_base,
                    definitions.name AS agent_name, definitions.frontmatter AS agent_frontmatter,
                    definitions.body AS agent_body,
                    NULLIF(definitions.frontmatter ->> 'harness', '') AS harness,
                    (
                        SELECT remotes.bare_path
                        FROM core.local_remotes remotes
                        JOIN core.repos repos ON repos.id = remotes.repo_id
                        WHERE repos.project_id = sessions.project_id
                        ORDER BY remotes.bare_path
                        LIMIT 1
                    ) AS workspace_remote
             FROM agents.runs runs
             JOIN agents.sessions sessions ON sessions.id = runs.session_id
             JOIN agents.agent_defs definitions
               ON definitions.id = COALESCE(runs.agent_def_id, sessions.agent_def_id)
             WHERE runs.id = $1",
        )
        .bind(run_id)
        .fetch_optional(self.pool())
        .await
        .map(|row| row.map(Into::into))
        .context("load dispatch run")
    }

    /// Record the host runtime's stable container identity for discard and reconciliation.
    pub async fn record_run_container(&self, run_id: RunId, container_id: &str) -> Result<()> {
        let updated = query(
            "UPDATE agents.runs SET container_id = $2
             WHERE id = $1 AND status = 'running'",
        )
        .bind(run_id)
        .bind(container_id)
        .execute(self.pool())
        .await
        .context("record run container")?;
        if updated.rows_affected() != 1 {
            bail!("run `{run_id}` is not running")
        }
        Ok(())
    }

    /// Persist the terminal result and remove the run from the dispatch queue.
    pub async fn complete_dispatch_run(&self, run_id: RunId, exit_code: i32) -> Result<()> {
        query(
            "UPDATE agents.runs
             SET status = CASE WHEN $2 = 0 THEN 'completed' ELSE 'aborted' END,
                 exit_code = $2, ended_at = now()
             WHERE id = $1 AND status = 'running'",
        )
        .bind(run_id)
        .bind(exit_code)
        .execute(self.pool())
        .await
        .context("complete dispatch run")?;
        query("DELETE FROM agents.dispatch_queue WHERE run_id = $1")
            .bind(run_id)
            .execute(self.pool())
            .await
            .context("remove completed dispatch run")?;
        Ok(())
    }

    /// Mark a claimed run as aborted when the host cannot launch it. The queue
    /// entry is removed so a malformed request cannot retry forever.
    pub async fn abort_dispatch_run(&self, run_id: RunId, reason: &str) -> Result<()> {
        let mut transaction = self.pool().begin().await.context("begin dispatch abort")?;
        query(
            "UPDATE agents.runs
             SET status = 'aborted', ended_at = now(), cancel_reason = $2
             WHERE id = $1 AND status = 'running'",
        )
        .bind(run_id)
        .bind(reason)
        .execute(&mut *transaction)
        .await
        .context("abort dispatch run")?;
        query("DELETE FROM agents.dispatch_queue WHERE run_id = $1")
            .bind(run_id)
            .execute(&mut *transaction)
            .await
            .context("remove aborted dispatch run")?;
        transaction
            .commit()
            .await
            .context("commit dispatch abort")?;
        Ok(())
    }

    /// Read the durable, machine-wide dispatch policy.
    pub async fn dispatch_policy(&self) -> Result<DispatchPolicy> {
        let row = query(
            "SELECT global_parallelism, per_project_parallelism, priority_method, tie_break,
                    preemption_enabled
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
                 singleton, global_parallelism, per_project_parallelism, priority_method, tie_break,
                 preemption_enabled
             ) VALUES (TRUE, $1, $2, $3, $4, $5)
             ON CONFLICT (singleton) DO UPDATE SET
                 global_parallelism = EXCLUDED.global_parallelism,
                 per_project_parallelism = EXCLUDED.per_project_parallelism,
                 priority_method = EXCLUDED.priority_method,
                 tie_break = EXCLUDED.tie_break,
                 preemption_enabled = EXCLUDED.preemption_enabled",
        )
        .bind(i32::try_from(policy.global_parallelism).context("global parallelism exceeds i32")?)
        .bind(
            i32::try_from(policy.per_project_parallelism)
                .context("per-project parallelism exceeds i32")?,
        )
        .bind(policy.priority_method.as_str())
        .bind(policy.tie_break.as_str())
        .bind(policy.preemption_enabled)
        .execute(self.pool())
        .await
        .context("persist dispatch policy")?;
        Ok(())
    }

    /// One dispatch schedule: a named cron that fires a workflow run.
    /// Every dispatch schedule with its project and workflow name, newest first.
    pub async fn schedules_list(&self) -> Result<Vec<ScheduleRow>> {
        query_as(
            "SELECT s.id, d.project_id, p.name AS project, d.name,
                    s.cron_expression AS cron_expression, s.paused_at IS NULL AS enabled,
                    d.created_at
             FROM workflows.schedules s
             JOIN workflows.workflow_defs d ON d.id = s.workflow_def_id
             JOIN core.projects p ON p.id = d.project_id
             ORDER BY d.created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .context("list dispatch schedules")
    }

    pub async fn schedule_executions(
        &self,
        project_id: Option<ProjectId>,
        limit: i64,
    ) -> Result<Vec<ScheduleExecutionRow>> {
        query_as(
            "SELECT e.id, d.name AS schedule_name, e.status, p.name AS project,
                    COALESCE(e.started_at, e.scheduled_for, e.created_at)::text AS scheduled_for,
                    e.started_at::text AS started_at,
                    e.ended_at::text AS ended_at
             FROM workflows.executions e
             JOIN workflows.workflow_defs d ON d.id = e.workflow_def_id
             JOIN core.projects p ON p.id = d.project_id
             WHERE ($1::uuid IS NULL OR d.project_id = $1)
             ORDER BY e.created_at DESC
             LIMIT $2",
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("list schedule executions: {e:#}"))
    }

    pub async fn guardrail_defaults(&self) -> Result<GuardrailDefaults> {
        let row = query("SELECT max_iterations, token_budget, stuck_iterations, change_lines_ceiling, change_files_ceiling, kill_and_reassign, network_tier, block_system_changes, autopilot FROM core.guardrail_defaults WHERE singleton = TRUE").fetch_one(self.pool()).await.context("read guardrail defaults")?;
        Ok(GuardrailDefaults {
            max_iterations: u32::try_from(row.try_get::<i32, _>("max_iterations")?)?,
            token_budget: row
                .try_get::<Option<i64>, _>("token_budget")?
                .map(|value| value as u64),
            stuck_iterations: u32::try_from(row.try_get::<i32, _>("stuck_iterations")?)?,
            change_lines: row
                .try_get::<Option<i32>, _>("change_lines_ceiling")?
                .map(|value| value as u32),
            change_files: row
                .try_get::<Option<i32>, _>("change_files_ceiling")?
                .map(|value| value as u32),
            kill_and_reassign: row.try_get("kill_and_reassign")?,
            network_tier: match row.try_get::<String, _>("network_tier")?.as_str() {
                "closed" => NetworkTier::Closed,
                "internal" => NetworkTier::Internal,
                _ => NetworkTier::Open,
            },
            block_system_changes: row.try_get("block_system_changes")?,
            autopilot: row.try_get("autopilot")?,
        })
    }

    pub async fn set_guardrail_defaults(
        &self,
        defaults: &GuardrailDefaults,
        explicit_looser_override: bool,
    ) -> Result<()> {
        let current = self.guardrail_defaults().await?;
        defaults.validate_change(&current, explicit_looser_override)?;
        query("UPDATE core.guardrail_defaults SET max_iterations = $1, token_budget = $2, stuck_iterations = $3, change_lines_ceiling = $4, change_files_ceiling = $5, kill_and_reassign = $6, network_tier = $7, block_system_changes = $8, autopilot = $9, updated_at = now() WHERE singleton = TRUE")
            .bind(i32::try_from(defaults.max_iterations)?).bind(defaults.token_budget.map(|value| value as i64)).bind(i32::try_from(defaults.stuck_iterations)?).bind(defaults.change_lines.map(|value| value as i32)).bind(defaults.change_files.map(|value| value as i32)).bind(defaults.kill_and_reassign).bind(match defaults.network_tier { NetworkTier::Closed => "closed", NetworkTier::Internal => "internal", NetworkTier::Open => "open" }).bind(defaults.block_system_changes).bind(defaults.autopilot).execute(self.pool()).await.context("persist guardrail defaults")?;
        Ok(())
    }

    /// Every project's tri-state autorun posture, for the Autorun switchboard.
    /// A project with no row defaults to Off.
    pub async fn autorun_states(&self) -> Result<Vec<AutorunStateRow>> {
        query_as(
            "SELECT p.id AS project_id, p.name AS project,
                    COALESCE(a.state, 'off') AS state
             FROM core.projects p
             LEFT JOIN core.project_autorun a ON a.project_id = p.id
             ORDER BY p.name",
        )
        .fetch_all(&self.pool)
        .await
        .context("list project autorun states")
    }

    /// Set whether a project may automatically start dispatchable work.
    pub async fn set_project_autorun(&self, project_id: ProjectId, enabled: bool) -> Result<()> {
        query(
            "INSERT INTO core.project_autorun (project_id, enabled, state)
             VALUES ($1, $2, CASE WHEN $2 THEN 'on' ELSE 'off' END)
             ON CONFLICT (project_id) DO UPDATE SET
                 enabled = EXCLUDED.enabled,
                 state = EXCLUDED.state,
                 updated_at = now()",
        )
        .bind(project_id)
        .bind(enabled)
        .execute(self.pool())
        .await
        .context("set project autorun")?;
        Ok(())
    }

    /// Replace the tri-state autorun posture, refusing to arm an archived project.
    pub async fn set_project_autorun_state(
        &self,
        project_id: ProjectId,
        state: crate::runtime::dispatch::AutorunState,
    ) -> Result<()> {
        if state == crate::runtime::dispatch::AutorunState::On
            && self.project_archived(project_id).await?
        {
            bail!("autorun cannot be turned on for an archived project");
        }
        let state_name = match state {
            crate::runtime::dispatch::AutorunState::On => "on",
            crate::runtime::dispatch::AutorunState::Off => "off",
            crate::runtime::dispatch::AutorunState::Suspended => "suspended",
        };
        query("INSERT INTO core.project_autorun (project_id, enabled, state) VALUES ($1, $2, $3) ON CONFLICT (project_id) DO UPDATE SET enabled = EXCLUDED.enabled, state = EXCLUDED.state, updated_at = now()")
            .bind(project_id).bind(state == crate::runtime::dispatch::AutorunState::On).bind(state_name).execute(self.pool()).await.context("set tri-state project autorun")?;
        Ok(())
    }

    pub async fn project_autorun_state(
        &self,
        project_id: ProjectId,
    ) -> Result<crate::runtime::dispatch::AutorunState> {
        let row = query("SELECT COALESCE((SELECT state FROM core.project_autorun WHERE project_id = $1), 'off') AS state").bind(project_id).fetch_one(self.pool()).await.context("read tri-state project autorun")?;
        match row.try_get::<&str, _>("state")? {
            "on" => Ok(crate::runtime::dispatch::AutorunState::On),
            "suspended" => Ok(crate::runtime::dispatch::AutorunState::Suspended),
            _ => Ok(crate::runtime::dispatch::AutorunState::Off),
        }
    }

    /// Read a project's autorun state; projects without a setting default to disabled.
    pub async fn project_autorun(&self, project_id: ProjectId) -> Result<bool> {
        let row = query(
            "SELECT COALESCE(
                 (SELECT enabled FROM core.project_autorun WHERE project_id = $1),
                 FALSE
             ) AS enabled",
        )
        .bind(project_id)
        .fetch_one(self.pool())
        .await
        .context("read project autorun")?;
        row.try_get("enabled").context("decode project autorun")
    }

    pub async fn project_autorun_policy(
        &self,
        project_id: ProjectId,
    ) -> Result<ProjectAutorunPolicy> {
        let row = query("SELECT review_pause_threshold, inbox_budget_per_hour, change_lines_ceiling, change_files_ceiling FROM core.project_autorun_policy WHERE project_id = $1")
            .bind(project_id).fetch_optional(self.pool()).await.context("read project autorun policy")?;
        let Some(row) = row else {
            return Ok(ProjectAutorunPolicy::default());
        };
        Ok(ProjectAutorunPolicy {
            review_pause_threshold: u32::try_from(
                row.try_get::<i32, _>("review_pause_threshold")?,
            )?,
            inbox_budget_per_hour: u32::try_from(row.try_get::<i32, _>("inbox_budget_per_hour")?)?,
            change_lines_ceiling: row
                .try_get::<Option<i32>, _>("change_lines_ceiling")?
                .map(u32::try_from)
                .transpose()?,
            change_files_ceiling: row
                .try_get::<Option<i32>, _>("change_files_ceiling")?
                .map(u32::try_from)
                .transpose()?,
        })
    }

    pub async fn set_project_autorun_policy(
        &self,
        project_id: ProjectId,
        policy: ProjectAutorunPolicy,
    ) -> Result<()> {
        query("INSERT INTO core.project_autorun_policy (project_id, review_pause_threshold, inbox_budget_per_hour, change_lines_ceiling, change_files_ceiling) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (project_id) DO UPDATE SET review_pause_threshold = EXCLUDED.review_pause_threshold, inbox_budget_per_hour = EXCLUDED.inbox_budget_per_hour, change_lines_ceiling = EXCLUDED.change_lines_ceiling, change_files_ceiling = EXCLUDED.change_files_ceiling, updated_at = now()")
            .bind(project_id).bind(i32::try_from(policy.review_pause_threshold)?).bind(i32::try_from(policy.inbox_budget_per_hour)?).bind(policy.change_lines_ceiling.map(i32::try_from).transpose()?).bind(policy.change_files_ceiling.map(i32::try_from).transpose()?).execute(self.pool()).await.context("persist project autorun policy")?;
        Ok(())
    }

    /// Snapshot and stop all active dispatch work, autorun settings, and schedules.
    ///
    /// Queued and running runs become `stopped`; branches, artifacts, memory, queue entries, and
    /// all other durable work remain untouched. The returned snapshot is restorable for ten minutes.
    pub async fn stop_all(&self) -> Result<StopAllSnapshot> {
        self.stop_all_with_handoffs(false).await
    }

    /// Stop all with an optional bounded handoff write before each run is stopped.
    pub async fn stop_all_with_handoffs(&self, write_handoffs: bool) -> Result<StopAllSnapshot> {
        let mut transaction = self.pool().begin().await.context("begin stop all")?;
        query("SELECT singleton FROM core.dispatch_policy WHERE singleton = TRUE FOR UPDATE")
            .fetch_one(&mut *transaction)
            .await
            .context("lock dispatch policy for stop all")?;
        let existing = query(
            "SELECT id FROM core.stop_all_snapshots
             WHERE restored_at IS NULL AND restore_expires_at > now()
             LIMIT 1",
        )
        .fetch_optional(&mut *transaction)
        .await
        .context("check active stop all snapshot")?;
        if existing.is_some() {
            bail!("an unexpired Stop all snapshot already exists")
        }

        let id = Uuid::new_v4();
        query(
            "INSERT INTO core.stop_all_snapshots (id, stopped_at, restore_expires_at)
             VALUES ($1, now(), now() + INTERVAL '10 minutes')",
        )
        .bind(id)
        .execute(&mut *transaction)
        .await
        .context("create stop all snapshot")?;

        let rows = query(
            "INSERT INTO core.stop_all_run_snapshots (snapshot_id, run_id, prior_status)
             SELECT $1, id, status
             FROM agents.runs
             WHERE status IN ('queued', 'running')
             RETURNING run_id",
        )
        .bind(id)
        .fetch_all(&mut *transaction)
        .await
        .context("snapshot active runs")?;
        let run_ids = rows
            .iter()
            .map(|row| row.try_get::<RunId, _>("run_id").map_err(Into::into))
            .collect::<Result<Vec<_>>>()
            .context("decode stopped run ids")?;
        if write_handoffs {
            query("INSERT INTO core.stop_all_handoffs (snapshot_id, run_id, payload) SELECT $1, run_id, jsonb_build_object('done', '[]'::jsonb, 'remaining', '[]'::jsonb, 'attempted', '[]'::jsonb, 'open', '[]'::jsonb) FROM core.stop_all_run_snapshots WHERE snapshot_id = $1")
                .bind(id).execute(&mut *transaction).await.context("write stop all handoffs")?;
        }
        query(
            "UPDATE agents.runs AS runs
             SET status = 'stopped'
             FROM core.stop_all_run_snapshots AS snapshot
             WHERE snapshot.snapshot_id = $1
               AND snapshot.run_id = runs.id
               AND runs.status = snapshot.prior_status",
        )
        .bind(id)
        .execute(&mut *transaction)
        .await
        .context("stop active runs")?;

        query(
            "INSERT INTO core.stop_all_autorun_snapshots (snapshot_id, project_id, enabled, state)
             SELECT $1, project_id, enabled, state FROM core.project_autorun",
        )
        .bind(id)
        .execute(&mut *transaction)
        .await
        .context("snapshot autorun state")?;
        query("UPDATE core.project_autorun SET enabled = FALSE, state = 'off', updated_at = now() WHERE enabled OR state <> 'off'")
            .execute(&mut *transaction)
            .await
            .context("disable autorun")?;

        query(
            "INSERT INTO core.stop_all_schedule_snapshots (snapshot_id, schedule_id, paused_at)
             SELECT $1, id, paused_at FROM workflows.schedules",
        )
        .bind(id)
        .execute(&mut *transaction)
        .await
        .context("snapshot schedule state")?;
        query("UPDATE workflows.schedules SET paused_at = now(), updated_at = now() WHERE paused_at IS NULL")
            .execute(&mut *transaction)
            .await
            .context("pause schedules")?;

        transaction.commit().await.context("commit stop all")?;
        Ok(StopAllSnapshot { id, run_ids })
    }

    /// Restore a Stop all snapshot before its ten-minute window expires.
    ///
    /// A stopped run that had been running is requeued, rather than falsely marked running after
    /// its container was stopped. Queued runs return to the queue unchanged.
    pub async fn restore_stop_all(&self, snapshot_id: Uuid) -> Result<()> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("begin stop all restore")?;
        query("SELECT singleton FROM core.dispatch_policy WHERE singleton = TRUE FOR UPDATE")
            .fetch_one(&mut *transaction)
            .await
            .context("lock dispatch policy for stop all restore")?;
        let snapshot = query(
            "SELECT restored_at IS NULL AND restore_expires_at >= now() AS restorable
             FROM core.stop_all_snapshots
             WHERE id = $1
             FOR UPDATE",
        )
        .bind(snapshot_id)
        .fetch_optional(&mut *transaction)
        .await
        .context("read stop all snapshot")?;
        let restorable: bool = snapshot
            .ok_or_else(|| anyhow::anyhow!("Stop all snapshot does not exist"))?
            .try_get("restorable")
            .context("decode Stop all restore window")?;
        if !restorable {
            bail!("Stop all snapshot is expired or already restored")
        }

        query(
            "INSERT INTO agents.dispatch_queue (
                 run_id, plan_order, manual_order, unblocks_count, estimate_minutes
             )
             SELECT snapshot.run_id, 0, 0, 0, 0
             FROM core.stop_all_run_snapshots AS snapshot
             JOIN agents.runs AS runs ON runs.id = snapshot.run_id
             WHERE snapshot.snapshot_id = $1 AND runs.status = 'stopped'
             ON CONFLICT (run_id) DO NOTHING",
        )
        .bind(snapshot_id)
        .execute(&mut *transaction)
        .await
        .context("restore dispatch queue entries")?;
        query(
            "UPDATE agents.runs AS runs
             SET status = CASE snapshot.prior_status
                 WHEN 'running' THEN 'queued'
                 ELSE snapshot.prior_status
             END
             FROM core.stop_all_run_snapshots AS snapshot
             WHERE snapshot.snapshot_id = $1
               AND snapshot.run_id = runs.id
               AND runs.status = 'stopped'",
        )
        .bind(snapshot_id)
        .execute(&mut *transaction)
        .await
        .context("restore stopped runs")?;
        query(
            "INSERT INTO core.project_autorun (project_id, enabled, state)
             SELECT project_id, enabled, state
             FROM core.stop_all_autorun_snapshots
             WHERE snapshot_id = $1
             ON CONFLICT (project_id) DO UPDATE SET
                 enabled = EXCLUDED.enabled,
                 state = EXCLUDED.state,
                 updated_at = now()",
        )
        .bind(snapshot_id)
        .execute(&mut *transaction)
        .await
        .context("restore autorun state")?;
        query(
            "UPDATE workflows.schedules AS schedules
             SET paused_at = snapshot.paused_at, updated_at = now()
             FROM core.stop_all_schedule_snapshots AS snapshot
             WHERE snapshot.snapshot_id = $1 AND snapshot.schedule_id = schedules.id",
        )
        .bind(snapshot_id)
        .execute(&mut *transaction)
        .await
        .context("restore schedules")?;
        query("UPDATE core.stop_all_snapshots SET restored_at = now() WHERE id = $1")
            .bind(snapshot_id)
            .execute(&mut *transaction)
            .await
            .context("mark Stop all snapshot restored")?;
        transaction
            .commit()
            .await
            .context("commit Stop all restore")?;
        Ok(())
    }

    /// Add a queued run to the durable dispatch queue. Its project is derived from the run session.
    pub async fn enqueue_autorun_dispatch(
        &self,
        project_id: ProjectId,
        run_id: RunId,
        priority: DispatchPriority,
        request: &crate::runtime::dispatch::AutorunRequest,
        unread_landed: u32,
        autorun_runs_last_hour: u32,
    ) -> Result<()> {
        use crate::runtime::dispatch::{autorun_exclusions, review_debt_pauses_autorun};
        if self.project_autorun_state(project_id).await?
            != crate::runtime::dispatch::AutorunState::On
        {
            bail!("project autorun is not on");
        }
        let policy = self.project_autorun_policy(project_id).await?;
        if review_debt_pauses_autorun(policy, unread_landed) {
            bail!("autorun paused by review debt");
        }
        if !policy.permits_inbox_run(autorun_runs_last_hour) {
            bail!("autorun hourly inbox budget exhausted");
        }
        if !autorun_exclusions(request).is_empty() {
            bail!("autorun request is excluded");
        }
        self.enqueue_dispatch(run_id, priority).await
    }

    pub async fn enqueue_dispatch(&self, run_id: RunId, priority: DispatchPriority) -> Result<()> {
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
    pub async fn claim_dispatchable_runs(&self) -> Result<Vec<RunId>> {
        let mut transaction = self.pool().begin().await.context("begin dispatch claim")?;
        let policy_row = query(
            "SELECT global_parallelism, per_project_parallelism, priority_method, tie_break,
                    preemption_enabled
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

    /// Persist an explicit supervisor request to pause a run at its next completed iteration.
    ///
    /// The snapshot is derived from the run's own session so a caller cannot substitute context
    /// from another task. The request is refused while boundary preemption is disabled.
    pub async fn request_dispatch_preemption(&self, run_id: RunId) -> Result<()> {
        let requested = query(
            "INSERT INTO agents.dispatch_preemptions (run_id, handoff_context)
             SELECT $1, jsonb_build_object(
                 'session_id', sessions.id,
                 'branch', sessions.branch,
                 'board_task_id', sessions.board_task_id,
                 'memory_base', sessions.memory_base
             )
             FROM agents.runs AS runs
             JOIN agents.sessions AS sessions ON sessions.id = runs.session_id
             JOIN core.dispatch_policy AS policy ON policy.singleton = TRUE
             WHERE runs.id = $1
               AND runs.status = 'running'
               AND policy.preemption_enabled = TRUE
             ON CONFLICT (run_id) DO UPDATE SET
                 handoff_context = EXCLUDED.handoff_context,
                 requested_at = now()",
        )
        .bind(run_id)
        .execute(self.pool())
        .await
        .context("request dispatch preemption")?;
        if requested.rows_affected() != 1 {
            bail!("only running runs may be preempted when boundary preemption is enabled")
        }
        Ok(())
    }

    /// Apply a pending preemption only once the run's workflow iteration has completed.
    ///
    /// A missing or incomplete iteration leaves the request durable and the run running, so a
    /// restart cannot turn a mid-iteration request into an interruption.
    pub async fn preempt_dispatch_at_iteration_boundary(
        &self,
        run_id: RunId,
    ) -> Result<Option<PreemptionHandoff>> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("begin dispatch preemption")?;
        let handoff = query(
            "SELECT preemptions.handoff_context
             FROM agents.dispatch_preemptions AS preemptions
             JOIN agents.runs AS runs ON runs.id = preemptions.run_id
             WHERE preemptions.run_id = $1
               AND runs.status = 'running'
               AND EXISTS (
                   SELECT 1
                   FROM workflows.iterations AS iterations
                   WHERE iterations.run_id = runs.id
                     AND iterations.ended_at IS NOT NULL
               )
             FOR UPDATE OF preemptions, runs",
        )
        .bind(run_id)
        .fetch_optional(&mut *transaction)
        .await
        .context("lock pending dispatch preemption")?;

        let Some(handoff) = handoff else {
            transaction
                .commit()
                .await
                .context("commit unchanged dispatch preemption")?;
            return Ok(None);
        };
        let handoff: PreemptionHandoff =
            serde_json::from_value(handoff.try_get("handoff_context")?)
                .context("decode dispatch preemption handoff")?;
        let paused =
            query("UPDATE agents.runs SET status = 'paused' WHERE id = $1 AND status = 'running'")
                .bind(run_id)
                .execute(&mut *transaction)
                .await
                .context("pause preempted run")?;
        if paused.rows_affected() != 1 {
            bail!("dispatch run `{run_id}` was no longer running")
        }
        query("DELETE FROM agents.dispatch_preemptions WHERE run_id = $1")
            .bind(run_id)
            .execute(&mut *transaction)
            .await
            .context("clear completed dispatch preemption")?;
        transaction
            .commit()
            .await
            .context("commit dispatch preemption")?;
        Ok(Some(handoff))
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
        preemption_enabled: row.try_get("preemption_enabled")?,
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
