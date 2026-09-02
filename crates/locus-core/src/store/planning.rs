//! Durable planning stage and requirement projections.

use crate::{
    ids::ProjectId,
    services::planning::{EditableSpec, PlanningStage, Requirement},
    store::Store,
};
use anyhow::{bail, Context, Result};
use sqlx::{query, Row};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct PlanSummaryRow {
    pub id: Uuid,
    pub title: String,
    pub project: String,
    pub state: String,
    pub stage: String,
    pub confidence: Option<f64>,
    pub open_count: i32,
    pub updated_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PlanRequirementRow {
    pub requirement_id: String,
    pub body: String,
    pub changed: bool,
    pub carries_board_card: bool,
}

fn stage_name(stage: PlanningStage) -> &'static str {
    match stage {
        PlanningStage::Inputs => "inputs",
        PlanningStage::Orient => "orient",
        PlanningStage::Converse => "converse",
        PlanningStage::Synthesis => "synthesis",
        PlanningStage::Recommend => "recommend",
        PlanningStage::Decompose => "decompose",
        PlanningStage::Approved => "approved",
    }
}

impl Store {
    /// List plans for one project. The project filter is part of the query so a
    /// caller cannot accidentally receive another project's planning work.
    pub async fn plans_list(&self, project_id: Option<ProjectId>) -> Result<Vec<PlanSummaryRow>> {
        sqlx::query_as(
            "SELECT p.id,
                    p.title,
                    project.name AS project,
                    p.state,
                    p.stage,
                    p.confidence,
                    p.open_count,
                    p.updated_at::text AS updated_at
             FROM core.plans p
             JOIN core.projects project ON project.id = p.project_id
             WHERE ($1::uuid IS NULL OR p.project_id = $1)
             ORDER BY p.updated_at DESC, p.id",
        )
        .bind(project_id)
        .fetch_all(self.pool())
        .await
        .context("list plans")
    }

    pub async fn create_plan(
        &self,
        id: Uuid,
        project_id: Uuid,
        title: &str,
        goal: &str,
    ) -> Result<()> {
        if title.trim().is_empty() || goal.trim().is_empty() {
            bail!("plan title and goal are required");
        }
        query("INSERT INTO core.plans (id, project_id, title, goal) VALUES ($1, $2, $3, $4)")
            .bind(id)
            .bind(project_id)
            .bind(title)
            .bind(goal)
            .execute(self.pool())
            .await
            .context("create plan")?;
        Ok(())
    }

    pub async fn plan_belongs_to_project(
        &self,
        plan_id: Uuid,
        project_id: ProjectId,
    ) -> Result<bool> {
        sqlx::query_scalar(
            "SELECT EXISTS(
                 SELECT 1 FROM core.plans
                 WHERE id = $1 AND project_id = $2
             )",
        )
        .bind(plan_id)
        .bind(project_id)
        .fetch_one(self.pool())
        .await
        .context("validate plan project")
    }

    pub async fn plan_requirements(&self, plan_id: Uuid) -> Result<Vec<PlanRequirementRow>> {
        sqlx::query_as(
            "SELECT requirement_id, body, changed, carries_board_card
             FROM core.plan_requirements
             WHERE plan_id = $1
             ORDER BY requirement_id",
        )
        .bind(plan_id)
        .fetch_all(self.pool())
        .await
        .context("list plan requirements")
    }

    pub async fn set_plan_stage(
        &self,
        id: Uuid,
        stage: PlanningStage,
        description: &str,
        duration_seconds: Option<i64>,
    ) -> Result<()> {
        let mut tx = self.pool().begin().await.context("begin plan stage")?;
        query("UPDATE core.plans SET stage = $2, updated_at = now() WHERE id = $1")
            .bind(id)
            .bind(stage_name(stage))
            .execute(&mut *tx)
            .await
            .context("update plan stage")?;
        query("INSERT INTO core.plan_stage_history (id, plan_id, stage, description, duration_seconds) VALUES ($1, $2, $3, $4, $5)").bind(Uuid::new_v4()).bind(id).bind(stage_name(stage)).bind(description).bind(duration_seconds).execute(&mut *tx).await.context("record plan stage")?;
        tx.commit().await.context("commit plan stage")?;
        Ok(())
    }

    pub async fn save_plan_requirements(&self, plan_id: Uuid, spec: &EditableSpec) -> Result<()> {
        let mut tx = self
            .pool()
            .begin()
            .await
            .context("begin plan requirements")?;
        for requirement in spec.requirements() {
            query("INSERT INTO core.plan_requirements (plan_id, requirement_id, body, changed) VALUES ($1, $2, $3, TRUE) ON CONFLICT (plan_id, requirement_id) DO UPDATE SET body = EXCLUDED.body, changed = TRUE").bind(plan_id).bind(requirement.id()).bind(requirement.body()).execute(&mut *tx).await.context("save plan requirement")?;
        }
        tx.commit().await.context("commit plan requirements")?;
        Ok(())
    }

    pub async fn plan_stage(&self, id: Uuid) -> Result<PlanningStage> {
        let stage: String = query("SELECT stage FROM core.plans WHERE id = $1")
            .bind(id)
            .fetch_one(self.pool())
            .await
            .context("read plan stage")?
            .try_get("stage")?;
        match stage.as_str() {
            "inputs" => Ok(PlanningStage::Inputs),
            "orient" => Ok(PlanningStage::Orient),
            "converse" => Ok(PlanningStage::Converse),
            "synthesis" => Ok(PlanningStage::Synthesis),
            "recommend" => Ok(PlanningStage::Recommend),
            "decompose" => Ok(PlanningStage::Decompose),
            "approved" => Ok(PlanningStage::Approved),
            _ => bail!("unknown plan stage"),
        }
    }

    pub async fn plan_requirement(
        &self,
        plan_id: Uuid,
        requirement_id: &str,
    ) -> Result<Requirement> {
        let row = query("SELECT requirement_id, body FROM core.plan_requirements WHERE plan_id = $1 AND requirement_id = $2").bind(plan_id).bind(requirement_id).fetch_one(self.pool()).await.context("read plan requirement")?;
        Requirement::new(
            row.try_get::<String, _>("requirement_id")?,
            row.try_get::<String, _>("body")?,
        )
    }
}
