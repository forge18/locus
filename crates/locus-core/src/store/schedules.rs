//! Durable schedule configuration and execution markers.

use crate::{
    runtime::dispatch::{ScheduleGuardrailOverrides, ScheduleMode},
    store::Store,
};
use anyhow::{bail, Context, Result};
use sqlx::query;
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
        let execution_id = Uuid::new_v4();
        query("INSERT INTO workflows.executions (id, workflow_def_id, schedule_id, status, scheduled_for) VALUES ($1, $2, $3, $4, $5::timestamptz) ON CONFLICT (schedule_id, scheduled_for) WHERE schedule_id IS NOT NULL AND scheduled_for IS NOT NULL DO NOTHING").bind(execution_id).bind(workflow_def_id).bind(id).bind(status).bind(scheduled_for).execute(self.pool()).await.context("record schedule execution")?;
        Ok(execution_id)
    }
}
