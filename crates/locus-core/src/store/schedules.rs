//! Durable schedule configuration and execution markers.

use crate::{
    runtime::dispatch::{ScheduleGuardrailOverrides, ScheduleMode},
    services::workflow::ExecutionEntryPayload,
    store::Store,
};
use anyhow::{bail, Context, Result};
use sqlx::{query, query_scalar};
use uuid::Uuid;

fn mode_name(mode: ScheduleMode) -> &'static str {
    match mode {
        ScheduleMode::RunOnce => "once",
        ScheduleMode::Scheduled => "scheduled",
        ScheduleMode::Hold => "hold",
    }
}

impl Store {
    #[allow(clippy::too_many_arguments)]
    pub async fn create_schedule(
        &self,
        id: Uuid,
        workflow_def_id: Option<Uuid>,
        cron_expression: &str,
        mode: ScheduleMode,
        project_id: Option<Uuid>,
        agent_def_id: Option<Uuid>,
        harness: Option<&str>,
        prompt: Option<&str>,
        overrides: ScheduleGuardrailOverrides,
    ) -> Result<()> {
        if cron_expression.trim().is_empty() {
            bail!("schedule expression is required");
        }
        query("INSERT INTO workflows.schedules (id, workflow_def_id, cron_expression, run_mode, project_id, agent_def_id, harness, prompt, guardrail_overrides) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind(id).bind(workflow_def_id).bind(cron_expression).bind(mode_name(mode)).bind(project_id).bind(agent_def_id).bind(harness).bind(prompt).bind(serde_json::to_value(overrides)?).execute(self.pool()).await.context("create schedule")?;
        Ok(())
    }

    pub async fn set_schedule_paused(&self, id: Uuid, paused: bool) -> Result<()> {
        query("UPDATE workflows.schedules SET paused_at = CASE WHEN $2 THEN COALESCE(paused_at, now()) ELSE NULL END, updated_at = now() WHERE id = $1").bind(id).bind(paused).execute(self.pool()).await.context("set schedule paused")?;
        Ok(())
    }

    pub async fn record_schedule_execution(
        &self,
        id: Uuid,
        workflow_def_id: Uuid,
        scheduled_for: Option<String>,
        status: &str,
    ) -> Result<Uuid> {
        if status.trim().is_empty() {
            bail!("schedule execution status is required");
        }
        if let Some(scheduled_for) = scheduled_for.as_deref() {
            if let Some(existing) = query_scalar::<_, Uuid>(
                "SELECT id FROM workflows.executions
                 WHERE schedule_id = $1 AND scheduled_for = $2::timestamptz",
            )
            .bind(id)
            .bind(scheduled_for)
            .fetch_optional(self.pool())
            .await
            .context("find existing schedule execution")?
            {
                return Ok(existing);
            }
        }
        let project_id: Uuid =
            query_scalar("SELECT project_id FROM workflows.workflow_defs WHERE id = $1")
                .bind(workflow_def_id)
                .fetch_one(self.pool())
                .await
                .context("find workflow project for schedule execution")?;
        let execution_id = Uuid::new_v4();
        self.append_execution_entry(
            project_id.into(),
            ExecutionEntryPayload {
                execution_id,
                workflow_def_id,
                schedule_id: Some(id),
                status: status.to_owned(),
                scheduled_for,
                started_at: None,
                ended_at: None,
            },
            "system",
        )
        .await
        .context("record schedule execution through workflow log")?;
        Ok(execution_id)
    }
}
