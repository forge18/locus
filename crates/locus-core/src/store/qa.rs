//! Persistence adapters for the QA projections.

use crate::{
    ids::{ProjectId, RunId},
    services::qa::{
        CheckRun, CheckSource, CheckTrigger, Finding, FindingSeverity, QaError, QaStore,
    },
    store::Store,
};
use anyhow::{Context, Result};
use sqlx::{query, query_as, Row};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct QaFindingRow {
    pub id: Uuid,
    pub source_id: String,
    pub severity: String,
    pub title: String,
    pub project: String,
    pub location: String,
    pub explanation: String,
    pub sent_to_inbox: bool,
}

impl Store {
    pub async fn start_qa_check(
        &self,
        project_id: ProjectId,
        source: &CheckSource,
        trigger: CheckTrigger,
        now: i64,
    ) -> Result<CheckRun> {
        let inserted = query("INSERT INTO core.qa_check_runs (id, project_id, check_source_id, trigger, started_at) VALUES ($1, $2, $3, $4, to_timestamp($5)) ON CONFLICT DO NOTHING RETURNING id, started_at")
            .bind(Uuid::new_v4()).bind(project_id).bind(&source.id).bind(match trigger { CheckTrigger::Manual => "manual", CheckTrigger::Push => "push", CheckTrigger::Hourly => "hourly", CheckTrigger::Daily => "daily" }).bind(now)
            .fetch_optional(self.pool()).await.context("start QA check")?;
        let Some(row) = inserted else {
            // Preserve the overlap decision as a durable skipped execution instead of silently
            // dropping the scheduled firing.
            query("INSERT INTO core.qa_check_runs (id, project_id, check_source_id, trigger, started_at, skipped_at) VALUES ($1, $2, $3, $4, to_timestamp($5), to_timestamp($5))")
                .bind(Uuid::new_v4()).bind(project_id).bind(&source.id).bind(match trigger { CheckTrigger::Manual => "manual", CheckTrigger::Push => "push", CheckTrigger::Hourly => "hourly", CheckTrigger::Daily => "daily" }).bind(now)
                .execute(self.pool()).await.context("record skipped QA check")?;
            return Err(anyhow::anyhow!(
                "QA check source is already running; firing skipped"
            ));
        };
        Ok(CheckRun {
            id: row.try_get("id")?,
            project_id,
            check_source_id: source.id.clone(),
            trigger,
            started_at: now,
            finished_at: None,
        })
    }

    pub async fn finish_qa_check(
        &self,
        run_id: RunId,
        findings: &[Finding],
        now: i64,
    ) -> Result<()> {
        let mut tx = self
            .pool()
            .begin()
            .await
            .context("begin QA result replacement")?;
        let row = query("UPDATE core.qa_check_runs SET finished_at = to_timestamp($2) WHERE id = $1 AND finished_at IS NULL RETURNING project_id, check_source_id")
            .bind(run_id).bind(now).fetch_optional(&mut *tx).await.context("finish QA check")?;
        let Some(row) = row else {
            return Err(anyhow::anyhow!("unknown or finished QA check"));
        };
        let project_id: ProjectId = row.try_get("project_id")?;
        let source: String = row.try_get("check_source_id")?;
        query("DELETE FROM core.qa_findings WHERE project_id = $1 AND check_source_id = $2")
            .bind(project_id)
            .bind(&source)
            .execute(&mut *tx)
            .await
            .context("replace prior QA findings")?;
        for finding in findings {
            if finding.project_id != project_id || finding.check_source_id != source {
                return Err(anyhow::anyhow!("finding does not belong to QA check"));
            }
            let id = Uuid::parse_str(&finding.id).unwrap_or_else(|_| Uuid::new_v4());
            query("INSERT INTO core.qa_findings (id, check_run_id, project_id, check_source_id, severity, title, location, explanation, sent_to_inbox) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
                .bind(id).bind(run_id).bind(project_id).bind(&source).bind(match finding.severity { FindingSeverity::Fail => "fail", FindingSeverity::Warn => "warn" }).bind(&finding.title).bind(&finding.location).bind(&finding.explanation).bind(finding.sent_to_inbox).execute(&mut *tx).await.context("persist QA finding")?;
        }
        tx.commit().await.context("commit QA result replacement")?;
        Ok(())
    }

    pub async fn set_qa_schedule(
        &self,
        project_id: ProjectId,
        trigger: CheckTrigger,
    ) -> Result<()> {
        query("INSERT INTO core.qa_schedules (project_id, schedule) VALUES ($1, $2) ON CONFLICT (project_id) DO UPDATE SET schedule = EXCLUDED.schedule, updated_at = now()")
            .bind(project_id).bind(match trigger { CheckTrigger::Manual => "manual", CheckTrigger::Push => "push", CheckTrigger::Hourly => "hourly", CheckTrigger::Daily => "daily" }).execute(self.pool()).await.context("persist QA schedule")?;
        Ok(())
    }

    pub async fn qa_schedule(&self, project_id: ProjectId) -> Result<CheckTrigger> {
        let schedule: String = query("SELECT COALESCE((SELECT schedule FROM core.qa_schedules WHERE project_id = $1), 'manual') AS schedule").bind(project_id).fetch_one(self.pool()).await.context("read QA schedule")?.try_get("schedule")?;
        Ok(match schedule.as_str() {
            "push" => CheckTrigger::Push,
            "hourly" => CheckTrigger::Hourly,
            "daily" => CheckTrigger::Daily,
            _ => CheckTrigger::Manual,
        })
    }

    pub async fn qa_finding_count(&self, project_id: ProjectId, source: &str) -> Result<i64> {
        query("SELECT COUNT(*) AS count FROM core.qa_findings WHERE project_id = $1 AND check_source_id = $2").bind(project_id).bind(source).fetch_one(self.pool()).await.context("count QA findings").and_then(|row| row.try_get("count").context("decode QA finding count"))
    }

    pub async fn qa_findings(&self, project_id: ProjectId) -> Result<Vec<QaFindingRow>> {
        query_as(
            "SELECT f.id, f.check_source_id AS source_id, f.severity, f.title,
                    p.name AS project, f.location, f.explanation, f.sent_to_inbox
             FROM core.qa_findings f
             JOIN core.projects p ON p.id = f.project_id
             WHERE f.project_id = $1
             ORDER BY f.created_at, f.id",
        )
        .bind(project_id)
        .fetch_all(self.pool())
        .await
        .context("list project QA findings")
    }
}

#[allow(dead_code)]
fn _qa_store_is_service_boundary(_: &QaStore, _: Option<QaError>) {}
