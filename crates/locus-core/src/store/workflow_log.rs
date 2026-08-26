use crate::{
    ids::ProjectId,
    services::{
        condition::{rebuild_snapshot_as_of, Condition, RunSnapshot, StoredConditionEvent},
        workflow::{
            decode_entry_payload, ExecutionEntryPayload, GuardrailTripEntryPayload,
            IterationEntryPayload, VerifyResultEntryPayload, WorkflowEntry, WorkflowEntryKind,
            WorkflowPayload, WorkflowsProjection,
        },
    },
    store::Store,
};
use anyhow::{bail, Context, Result};
use serde_json::Value;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

impl Store {
    /// Append one validated domain entry and apply its workflow projection atomically.
    pub async fn append_workflow_entry(
        &self,
        project_id: ProjectId,
        kind: WorkflowEntryKind,
        version: u16,
        payload: Value,
        actor: &str,
        caused_by: Option<u64>,
    ) -> Result<WorkflowEntry> {
        let project = project_id;
        let mut entry = WorkflowEntry::new(project, 1, kind, version, payload, actor, caused_by);
        decode_entry_payload(&entry).map_err(|error| anyhow::anyhow!(error))?;
        if entry.actor.trim().is_empty() {
            bail!("workflow entry actor is required");
        }
        let mut transaction = self.pool().begin().await.context("begin workflow entry")?;
        let stream_pos: i64 = sqlx::query_scalar(
            "INSERT INTO log.project_streams (project_id, next_pos)
             VALUES ($1, 1)
             ON CONFLICT (project_id) DO UPDATE
                 SET next_pos = log.project_streams.next_pos + 1
             RETURNING next_pos",
        )
        .bind(project)
        .fetch_one(&mut *transaction)
        .await
        .context("reserve workflow stream position")?;
        entry.stream_pos =
            u64::try_from(stream_pos).context("workflow stream position is negative")?;
        sqlx::query(
            "INSERT INTO log.entries
                (project_id, stream_pos, kind, v, payload, actor, caused_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(project)
        .bind(stream_pos)
        .bind(kind.as_str())
        .bind(i16::try_from(version).context("workflow entry version exceeds SMALLINT")?)
        .bind(&entry.payload)
        .bind(&entry.actor)
        .bind(
            entry
                .caused_by
                .map(i64::try_from)
                .transpose()
                .context("causal position exceeds BIGINT")?,
        )
        .execute(&mut *transaction)
        .await
        .context("append workflow domain entry")?;

        let mut projection = WorkflowsProjection::default();
        projection
            .apply(&entry)
            .map_err(|error| anyhow::anyhow!(error))?;
        apply_workflow_projection(&mut transaction, &entry).await?;
        transaction
            .commit()
            .await
            .context("commit workflow entry")?;
        Ok(entry)
    }

    pub async fn append_execution_entry(
        &self,
        project_id: ProjectId,
        payload: ExecutionEntryPayload,
        actor: &str,
    ) -> Result<WorkflowEntry> {
        self.append_workflow_entry(
            project_id,
            WorkflowEntryKind::Execution,
            1,
            serde_json::to_value(payload)?,
            actor,
            None,
        )
        .await
    }

    pub async fn append_iteration_entry(
        &self,
        project_id: ProjectId,
        payload: IterationEntryPayload,
        actor: &str,
        caused_by: Option<u64>,
    ) -> Result<WorkflowEntry> {
        self.append_workflow_entry(
            project_id,
            WorkflowEntryKind::Iteration,
            1,
            serde_json::to_value(payload)?,
            actor,
            caused_by,
        )
        .await
    }

    /// Record arbiter classification as a version-one iteration entry; no separate arbiter verb exists.
    #[allow(clippy::too_many_arguments)]
    pub async fn append_arbiter_classification(
        &self,
        project_id: ProjectId,
        execution_id: Uuid,
        iteration_id: Uuid,
        iteration: &crate::services::arbiter::Iteration,
        run_id: Option<Uuid>,
        actor: &str,
        caused_by: Option<u64>,
    ) -> Result<WorkflowEntry> {
        let class = iteration
            .arbiter_class
            .ok_or_else(|| anyhow::anyhow!("iteration has no arbiter classification"))?;
        self.append_iteration_entry(
            project_id,
            IterationEntryPayload::arbiter_classification(
                iteration_id,
                execution_id,
                run_id,
                iteration.number,
                class,
            ),
            actor,
            caused_by,
        )
        .await
    }

    pub async fn append_guardrail_trip_entry(
        &self,
        project_id: ProjectId,
        payload: GuardrailTripEntryPayload,
        actor: &str,
        caused_by: Option<u64>,
    ) -> Result<WorkflowEntry> {
        self.append_workflow_entry(
            project_id,
            WorkflowEntryKind::GuardrailTrip,
            1,
            serde_json::to_value(payload)?,
            actor,
            caused_by,
        )
        .await
    }

    pub async fn append_verify_result_entry(
        &self,
        project_id: ProjectId,
        payload: VerifyResultEntryPayload,
        actor: &str,
        caused_by: Option<u64>,
    ) -> Result<WorkflowEntry> {
        self.append_workflow_entry(
            project_id,
            WorkflowEntryKind::VerifyResult,
            1,
            serde_json::to_value(payload)?,
            actor,
            caused_by,
        )
        .await
    }

    /// Rebuild the bounded condition snapshot from workflow entries at or before a position.
    pub async fn rebuild_workflow_snapshot(
        &self,
        project_id: ProjectId,
        stream_pos: u64,
    ) -> Result<RunSnapshot> {
        let rows = sqlx::query(
            "SELECT stream_pos, kind, v, payload, actor, caused_by
             FROM log.entries
             WHERE project_id = $1 AND stream_pos <= $2
             ORDER BY stream_pos
             LIMIT 4097",
        )
        .bind(project_id)
        .bind(i64::try_from(stream_pos).context("requested stream position exceeds BIGINT")?)
        .fetch_all(self.pool())
        .await
        .context("read workflow entries for condition replay")?;
        if rows.len() > 4096 {
            bail!("historical condition replay exceeds 4096 entries");
        }
        let mut events = Vec::with_capacity(rows.len());
        let mut snapshot = RunSnapshot::default();
        for row in rows {
            use sqlx::Row;
            let position: i64 = row.try_get("stream_pos")?;
            let kind = WorkflowEntryKind::parse(row.try_get("kind")?)
                .map_err(|error| anyhow::anyhow!(error))?;
            let entry = WorkflowEntry::new(
                project_id,
                u64::try_from(position).context("stored stream position is negative")?,
                kind,
                u16::try_from(row.try_get::<i16, _>("v")?).context("stored version is negative")?,
                row.try_get("payload")?,
                row.try_get::<String, _>("actor")?,
                row.try_get::<Option<i64>, _>("caused_by")?
                    .map(u64::try_from)
                    .transpose()
                    .context("stored causal position is negative")?,
            );
            let decoded = decode_entry_payload(&entry).map_err(|error| anyhow::anyhow!(error))?;
            apply_condition_payload(&mut snapshot, &decoded);
            events.push(StoredConditionEvent::new(
                entry.stream_pos,
                entry.kind.as_str(),
                snapshot.clone(),
            ));
        }
        rebuild_snapshot_as_of(&events, stream_pos).map_err(|error| anyhow::anyhow!(error))
    }

    pub async fn evaluate_condition_as_of(
        &self,
        project_id: ProjectId,
        stream_pos: u64,
        condition: &Condition,
    ) -> Result<bool> {
        Ok(condition.evaluate(
            &self
                .rebuild_workflow_snapshot(project_id, stream_pos)
                .await?,
        ))
    }

    pub async fn evaluate_condition_expression_as_of(
        &self,
        project_id: ProjectId,
        stream_pos: u64,
        expression: &str,
    ) -> Result<bool> {
        let condition = Condition::parse(expression).map_err(|error| anyhow::anyhow!(error))?;
        self.evaluate_condition_as_of(project_id, stream_pos, &condition)
            .await
    }
}

async fn apply_workflow_projection(
    transaction: &mut Transaction<'_, Postgres>,
    entry: &WorkflowEntry,
) -> Result<()> {
    let payload = decode_entry_payload(entry).map_err(|error| anyhow::anyhow!(error))?;
    match payload {
        WorkflowPayload::Execution(payload) => {
            sqlx::query(
                "INSERT INTO workflows.executions
                    (id, workflow_def_id, schedule_id, status, scheduled_for, started_at, ended_at)
                 VALUES ($1, $2, $3, $4, $5::timestamptz, $6::timestamptz, $7::timestamptz)
                 ON CONFLICT (id) DO UPDATE SET
                    workflow_def_id = EXCLUDED.workflow_def_id,
                    schedule_id = EXCLUDED.schedule_id,
                    status = EXCLUDED.status,
                    scheduled_for = EXCLUDED.scheduled_for,
                    started_at = EXCLUDED.started_at,
                    ended_at = EXCLUDED.ended_at",
            )
            .bind(payload.execution_id)
            .bind(payload.workflow_def_id)
            .bind(payload.schedule_id)
            .bind(payload.status)
            .bind(payload.scheduled_for)
            .bind(payload.started_at)
            .bind(payload.ended_at)
            .execute(&mut **transaction)
            .await
            .context("project workflow execution")?;
        }
        WorkflowPayload::Iteration(payload) => {
            sqlx::query(
                "INSERT INTO workflows.iterations
                    (id, execution_id, run_id, number, arbiter_class,
                     counts_toward_iteration_budget, started_at, ended_at)
                 VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7::timestamptz, now()), $8::timestamptz)
                 ON CONFLICT (id) DO UPDATE SET
                    execution_id = EXCLUDED.execution_id,
                    run_id = EXCLUDED.run_id,
                    number = EXCLUDED.number,
                    arbiter_class = EXCLUDED.arbiter_class,
                    counts_toward_iteration_budget = EXCLUDED.counts_toward_iteration_budget,
                    started_at = EXCLUDED.started_at,
                    ended_at = EXCLUDED.ended_at",
            )
            .bind(payload.iteration_id)
            .bind(payload.execution_id)
            .bind(payload.run_id)
            .bind(i32::try_from(payload.number).context("iteration number exceeds INTEGER")?)
            .bind(payload.arbiter_class)
            .bind(payload.counts_toward_iteration_budget)
            .bind(payload.started_at)
            .bind(payload.ended_at)
            .execute(&mut **transaction)
            .await
            .context("project workflow iteration")?;
        }
        WorkflowPayload::GuardrailTrip(payload) => {
            sqlx::query(
                "INSERT INTO workflows.guardrail_trips
                    (id, execution_id, iteration_id, run_id, guardrail, detail, tripped_at)
                 VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7::timestamptz, now()))
                 ON CONFLICT (id) DO UPDATE SET
                    execution_id = EXCLUDED.execution_id,
                    iteration_id = EXCLUDED.iteration_id,
                    run_id = EXCLUDED.run_id,
                    guardrail = EXCLUDED.guardrail,
                    detail = EXCLUDED.detail,
                    tripped_at = EXCLUDED.tripped_at",
            )
            .bind(payload.id)
            .bind(payload.execution_id)
            .bind(payload.iteration_id)
            .bind(payload.run_id)
            .bind(payload.guardrail)
            .bind(payload.detail)
            .bind(payload.tripped_at)
            .execute(&mut **transaction)
            .await
            .context("project workflow guardrail trip")?;
        }
        WorkflowPayload::VerifyResult(payload) => {
            sqlx::query(
                "INSERT INTO workflows.verify_results
                    (id, execution_id, iteration_id, verify_node_id, command, container_id,
                     exit_code, passed, stdout, stderr, completed_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                         COALESCE($11::timestamptz, now()))
                 ON CONFLICT (id) DO UPDATE SET
                    execution_id = EXCLUDED.execution_id,
                    iteration_id = EXCLUDED.iteration_id,
                    verify_node_id = EXCLUDED.verify_node_id,
                    command = EXCLUDED.command,
                    container_id = EXCLUDED.container_id,
                    exit_code = EXCLUDED.exit_code,
                    passed = EXCLUDED.passed,
                    stdout = EXCLUDED.stdout,
                    stderr = EXCLUDED.stderr,
                    completed_at = EXCLUDED.completed_at",
            )
            .bind(payload.id)
            .bind(payload.execution_id)
            .bind(payload.iteration_id)
            .bind(payload.verify_node_id)
            .bind(payload.command)
            .bind(payload.container_id)
            .bind(payload.exit_code)
            .bind(payload.passed)
            .bind(payload.stdout)
            .bind(payload.stderr)
            .bind(payload.completed_at)
            .execute(&mut **transaction)
            .await
            .context("project workflow verify result")?;
        }
    }
    Ok(())
}

fn apply_condition_payload(snapshot: &mut RunSnapshot, payload: &WorkflowPayload) {
    match payload {
        WorkflowPayload::Iteration(payload) => {
            snapshot.iteration = i64::from(payload.number);
        }
        WorkflowPayload::VerifyResult(payload) => {
            snapshot.verify_passed = payload.passed;
            snapshot.verify_exit_code = i64::from(payload.exit_code);
        }
        WorkflowPayload::Execution(_) | WorkflowPayload::GuardrailTrip(_) => {}
    }
}
