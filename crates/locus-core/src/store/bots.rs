//! Durable storage for persistent bots, their home sessions, and prompt routines.
//!
//! SQL stays here; lifecycle decisions live in `services::bots` so they can be tested without
//! Postgres and reused by the desktop and headless daemon paths.

use crate::{
    ids::{BotId, ProjectId, RoutineId, RunId, SessionId},
    services::{
        agents::AgentDefinition,
        bots::{
            self, Bot, BotContainerState, BotRoutine, BotRunStart, BotSettings, RoutineAttribution,
            RoutineClaim, RoutineClaimResult, RoutineExecution, RoutineExecutionStatus,
            RoutineResult, WarmStopAction, WarmStopDeadline, WarmWindow,
        },
    },
    store::Store,
};
use anyhow::{bail, Context, Result};
use serde_json::Value;
use sqlx::{query, query_as, query_scalar, Row};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct BotRow {
    id: Uuid,
    project_id: Uuid,
    name: String,
    agent_def_id: Uuid,
    home_session_id: Uuid,
    branch: String,
    container_id: Option<String>,
    container_state: String,
    warm_until: Option<String>,
    last_activity_at: Option<String>,
    total_cost_micros: Option<i64>,
}

impl BotRow {
    fn into_bot(self) -> Result<Bot> {
        let container_state = match self.container_state.as_str() {
            "cold" => BotContainerState::Cold,
            "running" => BotContainerState::Running,
            "warm" => BotContainerState::Warm,
            other => bail!("invalid persisted bot container state `{other}`"),
        };
        Ok(Bot {
            id: self.id.into(),
            project_id: self.project_id.into(),
            name: self.name,
            agent_def_id: self.agent_def_id.into(),
            home_session_id: self.home_session_id.into(),
            branch: self.branch,
            container_id: self.container_id,
            container_state,
            warm_until: self.warm_until,
            last_activity_at: self.last_activity_at,
            total_cost_micros: self
                .total_cost_micros
                .map(u64::try_from)
                .transpose()
                .context("bot cost is negative")?,
        })
    }
}

#[derive(sqlx::FromRow)]
struct RoutineRow {
    id: Uuid,
    bot_id: Uuid,
    prompt: String,
    cron_expression: String,
    enabled: bool,
    skipped_count: i32,
    schedule_id: Option<Uuid>,
}

impl RoutineRow {
    fn into_routine(self) -> Result<BotRoutine> {
        Ok(BotRoutine {
            id: self.id.into(),
            bot_id: self.bot_id.into(),
            prompt: self.prompt,
            cron_expression: self.cron_expression,
            enabled: self.enabled,
            skipped_count: u32::try_from(self.skipped_count)
                .context("routine skip count is negative")?,
            schedule_id: self.schedule_id,
        })
    }
}

impl Store {
    /// Create the immutable definition version and bind it to one durable bot home session.
    pub async fn create_bot(
        &self,
        project_id: ProjectId,
        definition: &AgentDefinition,
    ) -> Result<Bot> {
        if self
            .bot_by_name(project_id, &definition.frontmatter.name)
            .await?
            .is_some()
        {
            bail!(
                "bot `{}` already exists in this project",
                definition.frontmatter.name
            );
        }
        let persisted = self.save_agent_definition(definition).await?;
        let bot_id = BotId::generate();
        let session_id = SessionId::generate();
        let branch = bots::bot_branch(bot_id);
        let mut transaction = self.pool().begin().await.context("begin bot creation")?;
        query(
            "INSERT INTO agents.sessions
                (id, project_id, agent_def_id, name, branch)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(session_id)
        .bind(project_id)
        .bind(persisted.id)
        .bind(&definition.frontmatter.name)
        .bind(&branch)
        .execute(&mut *transaction)
        .await
        .context("create bot home session")?;
        query(
            "INSERT INTO bots.bots
                (id, project_id, name, agent_def_id, home_session_id, branch)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(bot_id)
        .bind(project_id)
        .bind(&definition.frontmatter.name)
        .bind(persisted.id)
        .bind(session_id)
        .bind(&branch)
        .execute(&mut *transaction)
        .await
        .context("create bot binding")?;
        transaction.commit().await.context("commit bot creation")?;
        self.bot(bot_id)
            .await?
            .context("created bot disappeared before it could be read")
    }

    pub async fn create_bot_from_markdown(
        &self,
        project_id: ProjectId,
        markdown: &str,
    ) -> Result<Bot> {
        let definition = AgentDefinition::parse(markdown)?;
        self.create_bot(project_id, &definition).await
    }

    pub async fn bot(&self, bot_id: BotId) -> Result<Option<Bot>> {
        query_as_bot(self.pool(), bot_id).await
    }

    pub async fn bot_by_name(&self, project_id: ProjectId, name: &str) -> Result<Option<Bot>> {
        let row = query_as::<_, BotRow>(
            "SELECT id, project_id, name, agent_def_id, home_session_id, branch,
                    container_id, container_state, warm_until::text AS warm_until,
                    last_activity_at::text AS last_activity_at, total_cost_micros
             FROM bots.bots WHERE project_id = $1 AND name = $2",
        )
        .bind(project_id)
        .bind(name)
        .fetch_optional(self.pool())
        .await
        .context("find bot by name")?;
        row.map(BotRow::into_bot).transpose()
    }

    pub async fn bots(&self, project_id: ProjectId) -> Result<Vec<Bot>> {
        let rows = query_as::<_, BotRow>(
            "SELECT id, project_id, name, agent_def_id, home_session_id, branch,
                    container_id, container_state, warm_until::text AS warm_until,
                    last_activity_at::text AS last_activity_at, total_cost_micros
             FROM bots.bots WHERE project_id = $1 ORDER BY name, id",
        )
        .bind(project_id)
        .fetch_all(self.pool())
        .await
        .context("list bots")?;
        rows.into_iter().map(BotRow::into_bot).collect()
    }

    pub async fn list_bots(&self, project_id: ProjectId) -> Result<Vec<Bot>> {
        self.bots(project_id).await
    }

    pub async fn bot_home_session(&self, bot_id: BotId) -> Result<SessionId> {
        query_scalar::<_, Uuid>("SELECT home_session_id FROM bots.bots WHERE id = $1")
            .bind(bot_id)
            .fetch_optional(self.pool())
            .await
            .context("read bot home session")?
            .map(Into::into)
            .ok_or_else(|| anyhow::anyhow!("bot `{bot_id}` was not found"))
    }

    /// Return the one active run created for a bot's home session.
    pub async fn active_bot_run(&self, bot_id: BotId) -> Result<Option<RunId>> {
        query_scalar::<_, Uuid>(
            "SELECT runs.id
             FROM agents.runs runs
             JOIN bots.bots bots ON bots.home_session_id = runs.session_id
             WHERE bots.id = $1 AND runs.status = 'running'
             ORDER BY runs.created_at DESC
             LIMIT 1",
        )
        .bind(bot_id)
        .fetch_optional(self.pool())
        .await
        .map(|run_id| run_id.map(Into::into))
        .context("read active bot run")
    }

    pub async fn set_bot_warm_window(
        &self,
        project_id: ProjectId,
        minutes: u32,
    ) -> Result<BotSettings> {
        let settings = self
            .project_settings(project_id)
            .await?
            .with_bot_warm_window_minutes(minutes)?;
        self.set_project_settings(project_id, &settings).await?;
        Ok(settings.bots().clone())
    }

    pub async fn bot_warm_window(&self, project_id: ProjectId) -> Result<WarmWindow> {
        self.project_settings(project_id)
            .await?
            .bots()
            .warm_window()
    }

    /// Start one run in the bot's home session. Latest definition resolution happens here, at
    /// run start; the run stores that immutable definition id before the session is advanced.
    pub async fn start_bot_run(
        &self,
        bot_id: BotId,
        run_id: RunId,
        resolved_model_id: &str,
    ) -> Result<BotRunStart> {
        self.start_bot_run_in_container(
            bot_id,
            run_id,
            resolved_model_id,
            format!("locus-agent-{run_id}"),
        )
        .await
    }

    pub async fn start_bot_run_in_container(
        &self,
        bot_id: BotId,
        run_id: RunId,
        resolved_model_id: &str,
        container_id: String,
    ) -> Result<BotRunStart> {
        self.start_bot_run_in_container_with_mode(
            bot_id,
            run_id,
            resolved_model_id,
            container_id,
            false,
        )
        .await
    }

    async fn start_bot_run_in_container_with_mode(
        &self,
        bot_id: BotId,
        run_id: RunId,
        resolved_model_id: &str,
        container_id: String,
        headless: bool,
    ) -> Result<BotRunStart> {
        if resolved_model_id.trim().is_empty() {
            bail!("bot run model must not be empty");
        }
        if container_id.trim().is_empty() {
            bail!("bot run container must not be empty");
        }
        let mut transaction = self.pool().begin().await.context("begin bot run")?;
        let row = query(
            "SELECT b.project_id, b.home_session_id, b.branch, b.agent_def_id,
                    b.container_state, b.container_id,
                    latest.id AS latest_agent_def_id,
                    EXISTS (
                        SELECT 1 FROM agents.runs prior
                        WHERE prior.session_id = b.home_session_id
                    ) AS has_history,
                    EXISTS (
                        SELECT 1 FROM agents.runs active
                        WHERE active.session_id = b.home_session_id
                          AND active.status = 'running'
                    ) AS has_active_run
             FROM bots.bots b
             JOIN agents.agent_defs current_def ON current_def.id = b.agent_def_id
             JOIN LATERAL (
                 SELECT candidate.id
                 FROM agents.agent_defs candidate
                 WHERE candidate.name = current_def.name
                 ORDER BY candidate.version DESC, candidate.id DESC
                 LIMIT 1
             ) latest ON TRUE
             WHERE b.id = $1
             FOR UPDATE OF b",
        )
        .bind(bot_id)
        .fetch_optional(&mut *transaction)
        .await
        .context("read bot run binding")?;
        let Some(row) = row else {
            bail!("bot `{bot_id}` was not found");
        };
        if row.try_get::<bool, _>("has_active_run")? {
            bail!("bot `{bot_id}` already has an active run");
        }
        let session_id: SessionId = row.try_get::<Uuid, _>("home_session_id")?.into();
        let branch: String = row.try_get("branch")?;
        let current_definition: Uuid = row.try_get("agent_def_id")?;
        let definition_id: Uuid = row.try_get("latest_agent_def_id")?;
        let resume: bool = row.try_get("has_history")?;
        let persisted_state: String = row.try_get("container_state")?;
        let persisted_container_id: Option<String> = row.try_get("container_id")?;
        let definition_changed = current_definition != definition_id;
        let reused_container =
            persisted_state == "warm" && persisted_container_id.is_some() && !definition_changed;
        let stop_container = (persisted_state == "warm" && !reused_container)
            .then_some(persisted_container_id.clone())
            .flatten();
        let container_id = if reused_container {
            persisted_container_id
                .clone()
                .ok_or_else(|| anyhow::anyhow!("warm bot reuse requires a container id"))?
        } else {
            container_id
        };
        if definition_changed {
            query("UPDATE bots.bots SET agent_def_id = $2, updated_at = now() WHERE id = $1")
                .bind(bot_id)
                .bind(definition_id)
                .execute(&mut *transaction)
                .await
                .context("advance bot definition")?;
            query("UPDATE agents.sessions SET agent_def_id = $2 WHERE id = $1")
                .bind(session_id)
                .bind(definition_id)
                .execute(&mut *transaction)
                .await
                .context("pin bot session definition")?;
        }
        query(
            "INSERT INTO agents.runs
                (id, session_id, agent_def_id, resolved_model_id, container_id, status, started_at)
             VALUES ($1, $2, $3, $4, $5, 'running', now())",
        )
        .bind(run_id)
        .bind(session_id)
        .bind(definition_id)
        .bind(resolved_model_id)
        .bind(&container_id)
        .execute(&mut *transaction)
        .await
        .context("create bot run")?;
        query(
            "UPDATE bots.bots
             SET container_id = $2, container_state = 'running', warm_until = NULL,
                 last_activity_at = now(), updated_at = now()
             WHERE id = $1",
        )
        .bind(bot_id)
        .bind(&container_id)
        .execute(&mut *transaction)
        .await
        .context("mark bot container running")?;
        transaction.commit().await.context("commit bot run")?;
        Ok(BotRunStart {
            run_id,
            session_id,
            definition_id: definition_id.into(),
            branch,
            container_id,
            stop_container,
            resume,
            reused_container,
            headless,
        })
    }

    pub async fn finish_bot_run(
        &self,
        bot_id: BotId,
        run_id: RunId,
        passed: bool,
        cost_micros: Option<u64>,
    ) -> Result<WarmStopDeadline> {
        let project_id: ProjectId =
            query_scalar::<_, Uuid>("SELECT project_id FROM bots.bots WHERE id = $1")
                .bind(bot_id)
                .fetch_optional(self.pool())
                .await
                .context("read bot project")?
                .map(Into::into)
                .ok_or_else(|| anyhow::anyhow!("bot `{bot_id}` was not found"))?;
        let warm_window = self.bot_warm_window(project_id).await?;
        let cost_micros = cost_micros
            .map(i64::try_from)
            .transpose()
            .context("bot cost exceeds BIGINT")?;
        let status = if passed { "completed" } else { "failed" };
        let mut transaction = self.pool().begin().await.context("begin bot completion")?;
        let updated = query(
            "UPDATE agents.runs AS runs
             SET status = $3, ended_at = now(), container_id = NULL
             FROM bots.bots AS bot
             WHERE bot.id = $1 AND runs.id = $2
               AND runs.session_id = bot.home_session_id
               AND runs.status = 'running'",
        )
        .bind(bot_id)
        .bind(run_id)
        .bind(status)
        .execute(&mut *transaction)
        .await
        .context("finish bot run")?;
        if updated.rows_affected() != 1 {
            bail!("run `{run_id}` is not an active run for bot `{bot_id}`");
        }
        let row = query(
            "UPDATE bots.bots
             SET container_state = 'warm',
                 warm_until = now() + make_interval(mins => $2::double precision),
                 last_activity_at = now(),
                 total_cost_micros = CASE
                     WHEN $3::bigint IS NULL THEN total_cost_micros
                     ELSE COALESCE(total_cost_micros, 0) + $3::bigint
                 END,
                 updated_at = now()
             WHERE id = $1
             RETURNING container_id,
                 to_char(warm_until AT TIME ZONE 'UTC',
                         'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS warm_until",
        )
        .bind(bot_id)
        .bind(f64::from(warm_window.minutes()))
        .bind(cost_micros)
        .fetch_one(&mut *transaction)
        .await
        .context("arm bot warm window")?;
        let container_id: Option<String> = row.try_get("container_id")?;
        let warm_until: String = row
            .try_get::<Option<String>, _>("warm_until")?
            .ok_or_else(|| anyhow::anyhow!("bot warm deadline was not returned"))?;
        transaction
            .commit()
            .await
            .context("commit bot completion")?;
        Ok(WarmStopDeadline {
            bot_id,
            container_id,
            warm_until: parse_timestamp(&warm_until)?,
        })
    }

    pub async fn mark_bot_activity(&self, bot_id: BotId) -> Result<()> {
        let project_id: ProjectId =
            query_scalar::<_, Uuid>("SELECT project_id FROM bots.bots WHERE id = $1")
                .bind(bot_id)
                .fetch_optional(self.pool())
                .await
                .context("read bot project for activity")?
                .map(Into::into)
                .ok_or_else(|| anyhow::anyhow!("bot `{bot_id}` was not found"))?;
        let minutes = self.bot_warm_window(project_id).await?.minutes();
        query(
            "UPDATE bots.bots
             SET last_activity_at = now(),
                 warm_until = CASE WHEN container_state = 'warm'
                     THEN now() + make_interval(mins => $2::double precision)
                     ELSE warm_until END,
                 updated_at = now()
             WHERE id = $1",
        )
        .bind(bot_id)
        .bind(f64::from(minutes))
        .execute(self.pool())
        .await
        .context("record bot activity")?;
        Ok(())
    }

    /// Atomically transitions an expired warm bot to cold and returns the old container id for
    /// the existing stop path. No transcript, session, definition, or branch is deleted.
    pub async fn expire_bot_warm_window(&self, bot_id: BotId) -> Result<Option<WarmStopAction>> {
        let row = query(
            "WITH expired AS (
                 SELECT id, container_id
                 FROM bots.bots
                 WHERE id = $1 AND container_state = 'warm'
                   AND warm_until IS NOT NULL AND warm_until <= now()
                 FOR UPDATE
             )
             UPDATE bots.bots AS bot
             SET container_state = 'cold', container_id = NULL, warm_until = NULL,
                 updated_at = now()
             FROM expired
             WHERE bot.id = expired.id
             RETURNING expired.container_id",
        )
        .bind(bot_id)
        .fetch_optional(self.pool())
        .await
        .context("expire bot warm window")?;
        Ok(row.map(|row| WarmStopAction {
            bot_id,
            container_id: row.try_get("container_id").ok().flatten(),
        }))
    }

    pub async fn create_bot_routine(
        &self,
        bot_id: BotId,
        prompt: &str,
        cron_expression: &str,
    ) -> Result<BotRoutine> {
        let routine_id = RoutineId::generate();
        let controller = bots::RoutineController::new(routine_id, bot_id, prompt, cron_expression)?;
        let bot = self
            .bot(bot_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("bot `{bot_id}` was not found"))?;
        let schedule_id = Uuid::new_v4();
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("begin routine creation")?;
        query(
            "INSERT INTO workflows.schedules
                (id, workflow_def_id, cron_expression, run_mode, project_id,
                 agent_def_id, prompt)
             VALUES ($1, NULL, $2, 'scheduled', $3, $4, $5)",
        )
        .bind(schedule_id)
        .bind(controller.cron_expression())
        .bind(bot.project_id)
        .bind(bot.agent_def_id)
        .bind(controller.prompt())
        .execute(&mut *transaction)
        .await
        .context("create bot schedule target")?;
        query(
            "INSERT INTO bots.routines
                (id, bot_id, prompt, cron_expression, schedule_id)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(routine_id)
        .bind(bot_id)
        .bind(controller.prompt())
        .bind(controller.cron_expression())
        .bind(schedule_id)
        .execute(&mut *transaction)
        .await
        .context("create bot routine")?;
        transaction
            .commit()
            .await
            .context("commit routine creation")?;
        Ok(BotRoutine {
            id: routine_id,
            bot_id,
            prompt: controller.prompt().into(),
            cron_expression: controller.cron_expression().into(),
            enabled: true,
            skipped_count: 0,
            schedule_id: Some(schedule_id),
        })
    }

    /// Alias with the cron-first order used by schedule callers.
    pub async fn create_routine(
        &self,
        bot_id: BotId,
        cron_expression: &str,
        prompt: &str,
    ) -> Result<BotRoutine> {
        self.create_bot_routine(bot_id, prompt, cron_expression)
            .await
    }

    pub async fn bot_routine(&self, routine_id: RoutineId) -> Result<Option<BotRoutine>> {
        let row = query_as::<_, RoutineRow>(
            "SELECT id, bot_id, prompt, cron_expression, enabled, skipped_count, schedule_id
             FROM bots.routines WHERE id = $1",
        )
        .bind(routine_id)
        .fetch_optional(self.pool())
        .await
        .context("read bot routine")?;
        row.map(RoutineRow::into_routine).transpose()
    }

    pub async fn bot_routines(&self, bot_id: BotId) -> Result<Vec<BotRoutine>> {
        let rows = query_as::<_, RoutineRow>(
            "SELECT id, bot_id, prompt, cron_expression, enabled, skipped_count, schedule_id
             FROM bots.routines WHERE bot_id = $1 ORDER BY created_at, id",
        )
        .bind(bot_id)
        .fetch_all(self.pool())
        .await
        .context("list bot routines")?;
        rows.into_iter().map(RoutineRow::into_routine).collect()
    }

    /// All routines for the headless scheduler. The bot id remains on each
    /// row so the scheduler can claim overlap atomically in the store.
    pub async fn all_bot_routines(&self) -> Result<Vec<BotRoutine>> {
        let rows = query_as::<_, RoutineRow>(
            "SELECT id, bot_id, prompt, cron_expression, enabled, skipped_count, schedule_id
             FROM bots.routines ORDER BY created_at, id",
        )
        .fetch_all(self.pool())
        .await
        .context("list all bot routines")?;
        rows.into_iter().map(RoutineRow::into_routine).collect()
    }

    pub async fn set_bot_routine_enabled(
        &self,
        routine_id: RoutineId,
        enabled: bool,
    ) -> Result<()> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("begin routine state update")?;
        let row = query("SELECT schedule_id FROM bots.routines WHERE id = $1 FOR UPDATE")
            .bind(routine_id)
            .fetch_optional(&mut *transaction)
            .await
            .context("read routine schedule for state update")?;
        let Some(row) = row else {
            bail!("routine `{routine_id}` was not found");
        };
        let schedule_id: Option<Uuid> = row.try_get("schedule_id")?;
        query(
            "UPDATE bots.routines
             SET enabled = $2, updated_at = now()
             WHERE id = $1",
        )
        .bind(routine_id)
        .bind(enabled)
        .execute(&mut *transaction)
        .await
        .context("set bot routine enabled")?;
        if let Some(schedule_id) = schedule_id {
            query(
                "UPDATE workflows.schedules
                 SET paused_at = CASE WHEN $2 THEN NULL ELSE COALESCE(paused_at, now()) END,
                     updated_at = now()
                 WHERE id = $1",
            )
            .bind(schedule_id)
            .bind(enabled)
            .execute(&mut *transaction)
            .await
            .context("set bot schedule pause state")?;
        }
        transaction
            .commit()
            .await
            .context("commit routine state update")?;
        Ok(())
    }

    pub async fn update_bot_routine(
        &self,
        routine_id: RoutineId,
        prompt: &str,
        cron_expression: &str,
    ) -> Result<BotRoutine> {
        let current = self
            .bot_routine(routine_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("routine `{routine_id}` was not found"))?;
        let controller =
            bots::RoutineController::new(routine_id, current.bot_id, prompt, cron_expression)?;
        let mut transaction = self.pool().begin().await.context("begin routine update")?;
        query(
            "UPDATE bots.routines
             SET prompt = $2, cron_expression = $3, updated_at = now()
             WHERE id = $1",
        )
        .bind(routine_id)
        .bind(controller.prompt())
        .bind(controller.cron_expression())
        .execute(&mut *transaction)
        .await
        .context("update bot routine")?;
        if let Some(schedule_id) = current.schedule_id {
            query(
                "UPDATE workflows.schedules
                 SET prompt = $2, cron_expression = $3, updated_at = now()
                 WHERE id = $1",
            )
            .bind(schedule_id)
            .bind(controller.prompt())
            .bind(controller.cron_expression())
            .execute(&mut *transaction)
            .await
            .context("update bot schedule target")?;
        }
        transaction
            .commit()
            .await
            .context("commit routine update")?;
        Ok(BotRoutine {
            id: routine_id,
            bot_id: current.bot_id,
            prompt: controller.prompt().into(),
            cron_expression: controller.cron_expression().into(),
            enabled: current.enabled,
            skipped_count: current.skipped_count,
            schedule_id: current.schedule_id,
        })
    }

    pub async fn delete_bot_routine(&self, routine_id: RoutineId) -> Result<()> {
        let mut transaction = self.pool().begin().await.context("begin routine delete")?;
        let row = query("SELECT schedule_id FROM bots.routines WHERE id = $1 FOR UPDATE")
            .bind(routine_id)
            .fetch_optional(&mut *transaction)
            .await
            .context("read routine schedule")?;
        let Some(row) = row else {
            bail!("routine `{routine_id}` was not found");
        };
        let schedule_id: Option<Uuid> = row.try_get("schedule_id")?;
        query("DELETE FROM bots.routines WHERE id = $1")
            .bind(routine_id)
            .execute(&mut *transaction)
            .await
            .context("delete bot routine")?;
        if let Some(schedule_id) = schedule_id {
            query("DELETE FROM workflows.schedules WHERE id = $1")
                .bind(schedule_id)
                .execute(&mut *transaction)
                .await
                .context("delete bot routine schedule")?;
        }
        transaction
            .commit()
            .await
            .context("commit routine delete")?;
        Ok(())
    }

    /// Claim one cron firing. A running bot or routine produces a durable skipped execution and
    /// no queue item; a test run bypasses the schedule's enabled/overlap state without mutating it.
    pub async fn claim_bot_routine(
        &self,
        routine_id: RoutineId,
        scheduled_for: OffsetDateTime,
        test_run: bool,
    ) -> Result<RoutineClaimResult> {
        let scheduled_for = scheduled_for
            .format(&Rfc3339)
            .context("format routine scheduled time")?;
        let mut transaction = self.pool().begin().await.context("begin routine claim")?;
        let row = query(
            "SELECT routine.bot_id, routine.prompt, routine.enabled,
                    schedule.paused_at IS NOT NULL AS schedule_paused,
                    bot.container_state,
                    EXISTS (
                        SELECT 1 FROM bots.routine_executions active
                        WHERE active.routine_id = routine.id
                          AND active.status = 'running' AND NOT active.test_run
                    ) AS routine_running
             FROM bots.routines routine
             JOIN bots.bots bot ON bot.id = routine.bot_id
             LEFT JOIN workflows.schedules schedule ON schedule.id = routine.schedule_id
             WHERE routine.id = $1
             FOR UPDATE OF routine, bot",
        )
        .bind(routine_id)
        .fetch_optional(&mut *transaction)
        .await
        .context("read routine claim")?;
        let Some(row) = row else {
            bail!("routine `{routine_id}` was not found");
        };
        let bot_id: BotId = row.try_get::<Uuid, _>("bot_id")?.into();
        let prompt: String = row.try_get("prompt")?;
        let enabled: bool = row.try_get("enabled")?;
        let schedule_paused: bool = row.try_get("schedule_paused")?;
        let bot_running: bool = row.try_get::<String, _>("container_state")? == "running";
        let routine_running: bool = row.try_get("routine_running")?;
        if !test_run && (!enabled || schedule_paused || bot_running || routine_running) {
            let execution_id = RoutineId::generate();
            query(
                "INSERT INTO bots.routine_executions
                    (id, routine_id, bot_id, prompt, scheduled_for, status, attribution, test_run)
                 VALUES ($1, $2, $3, $4, $5::timestamptz, 'skipped', 'routine-fired', FALSE)",
            )
            .bind(execution_id)
            .bind(routine_id)
            .bind(bot_id)
            .bind(&prompt)
            .bind(&scheduled_for)
            .execute(&mut *transaction)
            .await
            .context("record skipped bot routine")?;
            let skip_count: i32 = query_scalar(
                "UPDATE bots.routines
                 SET skipped_count = skipped_count + 1, updated_at = now()
                 WHERE id = $1
                 RETURNING skipped_count",
            )
            .bind(routine_id)
            .fetch_one(&mut *transaction)
            .await
            .context("count skipped bot routine")?;
            transaction
                .commit()
                .await
                .context("commit skipped routine")?;
            return Ok(RoutineClaimResult::Skipped {
                execution_id,
                skip_count: u32::try_from(skip_count).context("routine skip count is negative")?,
            });
        }
        let execution_id = RoutineId::generate();
        let attribution = if test_run {
            RoutineAttribution::TestRun
        } else {
            RoutineAttribution::RoutineFired
        };
        query(
            "INSERT INTO bots.routine_executions
                (id, routine_id, bot_id, prompt, scheduled_for, status, attribution, test_run)
             VALUES ($1, $2, $3, $4, $5::timestamptz, 'running', $6, $7)",
        )
        .bind(execution_id)
        .bind(routine_id)
        .bind(bot_id)
        .bind(&prompt)
        .bind(&scheduled_for)
        .bind(match attribution {
            RoutineAttribution::RoutineFired => "routine-fired",
            RoutineAttribution::TestRun => "test-run",
        })
        .bind(test_run)
        .execute(&mut *transaction)
        .await
        .context("claim bot routine")?;
        transaction.commit().await.context("commit routine claim")?;
        Ok(RoutineClaimResult::Started(RoutineClaim {
            execution_id,
            bot_id,
            prompt,
            attribution,
            test_run,
            headless: true,
        }))
    }

    pub async fn complete_bot_routine_execution(
        &self,
        execution_id: RoutineId,
        result: RoutineResult,
        run_id: Option<RunId>,
    ) -> Result<()> {
        let status = if result.passed { "completed" } else { "failed" };
        let updated = query(
            "UPDATE bots.routine_executions
             SET status = $2, result = $3, run_id = COALESCE($4, run_id), ended_at = now()
             WHERE id = $1 AND status = 'running'",
        )
        .bind(execution_id)
        .bind(status)
        .bind(serde_json::to_value(result)?)
        .bind(run_id)
        .execute(self.pool())
        .await
        .context("complete bot routine execution")?;
        if updated.rows_affected() != 1 {
            bail!("routine execution `{execution_id}` is not running");
        }
        Ok(())
    }

    pub async fn bot_routine_executions(&self, bot_id: BotId) -> Result<Vec<RoutineExecution>> {
        let rows = query(
            "SELECT id, bot_id, extract(epoch FROM scheduled_for)::bigint AS scheduled_for,
                    status, result, attribution, test_run
             FROM bots.routine_executions
             WHERE bot_id = $1
             ORDER BY scheduled_for, id",
        )
        .bind(bot_id)
        .fetch_all(self.pool())
        .await
        .context("list bot routine executions")?;
        rows.into_iter().map(routine_execution_from_row).collect()
    }

    pub async fn routine_executions(&self, bot_id: BotId) -> Result<Vec<RoutineExecution>> {
        self.bot_routine_executions(bot_id).await
    }

    /// A convenience path for the headless daemon: claim, then start the home session without
    /// requiring an attached desktop window. The routine execution remains attributed in the
    /// same conversation once the caller appends its prompt event.
    pub async fn fire_bot_routine(
        &self,
        routine_id: RoutineId,
        scheduled_for: OffsetDateTime,
        resolved_model_id: &str,
    ) -> Result<RoutineClaimResult> {
        let claim = self
            .claim_bot_routine(routine_id, scheduled_for, false)
            .await?;
        if let RoutineClaimResult::Started(start) = &claim {
            let run_id = RunId::generate();
            if let Err(error) = self
                .start_bot_run_in_container_with_mode(
                    start.bot_id,
                    run_id,
                    resolved_model_id,
                    format!("locus-agent-{run_id}"),
                    true,
                )
                .await
            {
                let _ = self
                    .complete_bot_routine_execution(
                        start.execution_id,
                        RoutineResult::failed(error.to_string()),
                        None,
                    )
                    .await;
                return Err(error);
            }
            query("UPDATE bots.routine_executions SET run_id = $2 WHERE id = $1")
                .bind(start.execution_id)
                .bind(run_id)
                .execute(self.pool())
                .await
                .context("link routine execution to home run")?;
        }
        Ok(claim)
    }
}

async fn query_as_bot(pool: &sqlx::PgPool, bot_id: BotId) -> Result<Option<Bot>> {
    let row = query_as::<_, BotRow>(
        "SELECT id, project_id, name, agent_def_id, home_session_id, branch,
                container_id, container_state, warm_until::text AS warm_until,
                last_activity_at::text AS last_activity_at, total_cost_micros
         FROM bots.bots WHERE id = $1",
    )
    .bind(bot_id)
    .fetch_optional(pool)
    .await
    .context("read bot")?;
    row.map(BotRow::into_bot).transpose()
}

fn parse_timestamp(value: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).context("parse bot warm deadline")
}

fn routine_execution_from_row(row: sqlx::postgres::PgRow) -> Result<RoutineExecution> {
    let status = match row.try_get::<String, _>("status")?.as_str() {
        "running" => RoutineExecutionStatus::Running,
        "completed" => RoutineExecutionStatus::Completed,
        "failed" => RoutineExecutionStatus::Failed,
        "skipped" => RoutineExecutionStatus::Skipped,
        other => bail!("invalid routine execution status `{other}`"),
    };
    let attribution = match row.try_get::<String, _>("attribution")?.as_str() {
        "routine-fired" => RoutineAttribution::RoutineFired,
        "test-run" => RoutineAttribution::TestRun,
        other => bail!("invalid routine attribution `{other}`"),
    };
    let result = row
        .try_get::<Option<Value>, _>("result")?
        .map(serde_json::from_value)
        .transpose()
        .context("decode routine execution result")?;
    Ok(RoutineExecution {
        id: row.try_get::<Uuid, _>("id")?.into(),
        bot_id: row.try_get::<Uuid, _>("bot_id")?.into(),
        scheduled_for: row.try_get("scheduled_for")?,
        status,
        result,
        attribution,
        test_run: row.try_get("test_run")?,
    })
}
