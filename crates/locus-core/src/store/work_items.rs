//! Durable adapters for external work-item snapshots and completion delivery.

use crate::{
    ids::{AgentDefId, ProjectId, RunId, SessionId, TaskId},
    services::{
        board::{
            BoardActor, BoardComment, BoardCommentOrigin, BoardEvent, BoardEvidenceLink, BoardTask,
        },
        manage::TaskColumn,
        task::{TaskEvidenceLink, TaskRunLink, WorkflowSelection},
    },
    store::Store,
    work_item::{
        CompletionDelivery, CompletionEvent, CompletionOutbox, WorkItemIdentity,
        WorkItemProviderConfig, WorkItemProviderId, WorkItemSnapshot, WorkItemSyncApplication,
        WorkItemSyncState,
    },
};
use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use sqlx::{query, query_as, query_scalar, Postgres, Transaction};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedExternalCompletion {
    pub id: Uuid,
    pub task_id: TaskId,
    pub identity: WorkItemIdentity,
    pub comment: String,
    pub locator: String,
    pub evidence: Vec<crate::ids::ArtifactId>,
    pub attempts: u32,
    pub commented: bool,
    pub resolved: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedExternalWorkItem {
    pub task: BoardTask,
    pub snapshot: WorkItemSnapshot,
    pub workflow: WorkflowSelection,
    pub runs: Vec<TaskRunLink>,
    pub evidence: Vec<TaskEvidenceLink>,
    pub sync_state: WorkItemSyncState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedExternalCompletionStatus {
    pub attempts: u32,
    pub commented: bool,
    pub resolved: Option<bool>,
    pub resolution_supported: bool,
    pub last_error: Option<String>,
    pub status: String,
}

#[derive(sqlx::FromRow)]
struct PersistedExternalWorkItemRow {
    id: TaskId,
    project_id: ProjectId,
    repo_id: Option<Uuid>,
    session_id: Option<SessionId>,
    summary: String,
    description: String,
    column_name: String,
    blocked: bool,
    blocked_reason: Option<String>,
    blocked_clear_condition: Option<String>,
    assigned_agent_def_id: Option<AgentDefId>,
    verify_command: Option<String>,
    workflow_def_id: Uuid,
    workflow_project_id: ProjectId,
    plugin_id: String,
    host: String,
    provider_project: String,
    native_id: String,
    url: String,
    title: String,
    body: String,
    labels: Value,
    source_status: String,
    pull_cursor: Option<String>,
    last_pushed_status: Option<String>,
    note_watermark: Option<String>,
    last_local_status_at: Option<String>,
    last_external_status_at: Option<String>,
    last_sync_error: Option<String>,
    last_synced_at: Option<String>,
    unmapped_external_status: Option<String>,
    last_conflict_winner: Option<String>,
    last_conflict_reason: Option<String>,
}

fn task_column(value: &str) -> Result<TaskColumn> {
    match value {
        "ready" => Ok(TaskColumn::Ready),
        "in_progress" => Ok(TaskColumn::InProgress),
        "testing" => Ok(TaskColumn::Testing),
        "reviewing" => Ok(TaskColumn::Reviewing),
        "pending_approval" | "waiting_for_approval" => Ok(TaskColumn::PendingApproval),
        "done" => Ok(TaskColumn::Done),
        other => Err(anyhow!("invalid persisted task column `{other}`")),
    }
}

fn database_task_column(column: TaskColumn) -> &'static str {
    match column {
        TaskColumn::PendingApproval => "waiting_for_approval",
        other => other.as_str(),
    }
}

async fn insert_external_completion(
    transaction: &mut Transaction<'_, Postgres>,
    delivery: &CompletionDelivery,
    snapshot: &WorkItemSnapshot,
) -> Result<u64> {
    query(
        "INSERT INTO board.external_completion_outbox
         (id, task_id, plugin_id, host, provider_project, native_id, comment, locator, evidence, attempts, resolution_supported)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         ON CONFLICT (task_id) DO NOTHING",
    )
    .bind(delivery.event.id)
    .bind(delivery.event.task_id)
    .bind(snapshot.identity.plugin_id.as_str())
    .bind(&snapshot.identity.host)
    .bind(&snapshot.identity.project)
    .bind(&snapshot.identity.native_id)
    .bind(&delivery.event.comment)
    .bind(&delivery.event.locator)
    .bind(serde_json::to_value(&delivery.event.evidence)?)
    .bind(
        i32::try_from(delivery.attempts).context("completion attempts exceed database range")?,
    )
    .bind(delivery.resolved.is_some())
    .execute(&mut **transaction)
    .await
    .context("enqueue external completion")
    .map(|result| result.rows_affected())
}

impl Store {
    pub async fn save_external_work_item_provider(
        &self,
        config: &WorkItemProviderConfig,
    ) -> Result<()> {
        query(
            "INSERT INTO board.external_work_item_providers
             (plugin_id, host, provider_project, sync_interval_seconds)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (plugin_id, host, provider_project) DO UPDATE
             SET host = EXCLUDED.host,
                 provider_project = EXCLUDED.provider_project,
                 sync_interval_seconds = EXCLUDED.sync_interval_seconds,
                 configured_at = now()",
        )
        .bind(config.plugin_id.as_str())
        .bind(&config.host)
        .bind(&config.project)
        .bind(i32::try_from(config.sync_interval_seconds).context("sync interval exceeds INTEGER")?)
        .execute(self.pool())
        .await
        .context("save external work-item provider")?;
        Ok(())
    }

    pub async fn load_external_work_item_providers(&self) -> Result<Vec<WorkItemProviderConfig>> {
        let rows = query_as::<_, (String, String, String, i32)>(
            "SELECT plugin_id, host, provider_project, sync_interval_seconds
             FROM board.external_work_item_providers
             ORDER BY plugin_id, host, provider_project",
        )
        .fetch_all(self.pool())
        .await
        .context("load external work-item providers")?;

        rows.into_iter()
            .map(|(plugin_id, host, project, sync_interval_seconds)| {
                let plugin_id = WorkItemProviderId::new(plugin_id)
                    .map_err(|error| anyhow!("invalid external provider id: {error}"))?;
                let config = WorkItemProviderConfig::new(plugin_id.as_str(), host, project)
                    .map_err(|error| anyhow!("invalid external provider configuration: {error}"))?;
                config
                    .with_sync_interval(
                        u32::try_from(sync_interval_seconds)
                            .context("stored sync interval is outside u32 range")?,
                    )
                    .map_err(|error| anyhow!("invalid external provider sync interval: {error}"))
            })
            .collect()
    }

    pub async fn external_work_item_task(
        &self,
        identity: &WorkItemIdentity,
    ) -> Result<Option<TaskId>> {
        query_scalar(
            "SELECT task_id
             FROM board.external_work_items
             WHERE plugin_id = $1 AND host = $2 AND provider_project = $3 AND native_id = $4",
        )
        .bind(identity.plugin_id.as_str())
        .bind(&identity.host)
        .bind(&identity.project)
        .bind(&identity.native_id)
        .fetch_optional(self.pool())
        .await
        .context("find external work-item task")
    }

    pub async fn external_sync_state(&self, task_id: TaskId) -> Result<Option<WorkItemSyncState>> {
        let row = query_as::<
            _,
            (
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
            ),
        >(
            "SELECT pull_cursor, last_pushed_status, note_watermark,
                    to_char(last_local_status_at AT TIME ZONE 'UTC',
                            'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"'),
                    to_char(last_external_status_at AT TIME ZONE 'UTC',
                            'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"'),
                    last_sync_error,
                    to_char(last_synced_at AT TIME ZONE 'UTC',
                            'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"'),
                    unmapped_external_status,
                    last_conflict_winner, last_conflict_reason
             FROM board.external_work_items
             WHERE task_id = $1",
        )
        .bind(task_id)
        .fetch_optional(self.pool())
        .await
        .context("load external work-item sync state")?;
        Ok(row.map(
            |(
                pull_cursor,
                last_pushed_status,
                note_watermark,
                last_local_status_at,
                last_external_status_at,
                last_sync_error,
                last_synced_at,
                unmapped_external_status,
                last_conflict_winner,
                last_conflict_reason,
            )| WorkItemSyncState {
                pull_cursor,
                last_pushed_status,
                note_watermark,
                last_local_status_at,
                last_external_status_at,
                last_sync_error,
                last_synced_at,
                unmapped_external_status,
                last_conflict_winner,
                last_conflict_reason,
            },
        ))
    }

    pub async fn save_external_sync_state(
        &self,
        task_id: TaskId,
        state: &WorkItemSyncState,
    ) -> Result<bool> {
        let updated = query(
            "UPDATE board.external_work_items
             SET pull_cursor = $2,
                 last_pushed_status = $3,
                 note_watermark = $4,
                 last_local_status_at = $5::timestamptz,
                 last_external_status_at = $6::timestamptz,
                 last_sync_error = $7,
                 last_synced_at = $8::timestamptz,
                 unmapped_external_status = $9,
                 last_conflict_winner = $10,
                 last_conflict_reason = $11
             WHERE task_id = $1",
        )
        .bind(task_id)
        .bind(&state.pull_cursor)
        .bind(&state.last_pushed_status)
        .bind(&state.note_watermark)
        .bind(&state.last_local_status_at)
        .bind(&state.last_external_status_at)
        .bind(&state.last_sync_error)
        .bind(&state.last_synced_at)
        .bind(&state.unmapped_external_status)
        .bind(&state.last_conflict_winner)
        .bind(&state.last_conflict_reason)
        .execute(self.pool())
        .await
        .context("save external work-item sync state")?;
        Ok(updated.rows_affected() == 1)
    }

    pub async fn persist_local_task_comment(
        &self,
        task_id: TaskId,
        note_id: &str,
        comment: &BoardComment,
    ) -> Result<bool> {
        if !matches!(comment.origin, BoardCommentOrigin::Local) {
            return Err(anyhow!("local task comment has a non-local origin"));
        }
        if note_id.trim().is_empty() || note_id.contains('\0') {
            return Err(anyhow!("local task comment id is invalid"));
        }
        let inserted = query(
            "INSERT INTO board.task_comments
             (id, task_id, author, body, origin, external_provider, external_note_id)
             VALUES ($1, $2, $3, $4, 'local', 'locus', $5)
             ON CONFLICT (task_id, origin, external_provider, external_note_id)
             DO NOTHING",
        )
        .bind(Uuid::new_v4())
        .bind(task_id)
        .bind(&comment.author)
        .bind(&comment.body)
        .bind(note_id)
        .execute(self.pool())
        .await
        .context("persist local task comment")?;
        Ok(inserted.rows_affected() == 1)
    }

    pub async fn record_external_sync_error(&self, task_id: TaskId, error: &str) -> Result<bool> {
        let error = error.chars().take(2048).collect::<String>();
        let updated = query(
            "UPDATE board.external_work_items
             SET last_sync_error = $2
             WHERE task_id = $1",
        )
        .bind(task_id)
        .bind(error)
        .execute(self.pool())
        .await
        .context("record external work-item sync error")?;
        Ok(updated.rows_affected() == 1)
    }

    pub async fn persist_external_sync(
        &self,
        task_id: TaskId,
        snapshot: &WorkItemSnapshot,
        application: &WorkItemSyncApplication,
        state: &WorkItemSyncState,
    ) -> Result<bool> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("begin external work-item sync")?;
        let persisted_identity = query_as::<_, (String, String, String, String)>(
            "SELECT plugin_id, host, provider_project, native_id
             FROM board.external_work_items
             WHERE task_id = $1
             FOR UPDATE",
        )
        .bind(task_id)
        .fetch_optional(&mut *transaction)
        .await
        .context("load external sync identity")?
        .ok_or_else(|| anyhow!("external work-item does not exist"))?;
        if persisted_identity.0 != snapshot.identity.plugin_id.as_str()
            || persisted_identity.1 != snapshot.identity.host
            || persisted_identity.2 != snapshot.identity.project
            || persisted_identity.3 != snapshot.identity.native_id
        {
            return Err(anyhow!("external sync identity does not match the task"));
        }
        let local_project_id: ProjectId =
            query_scalar("SELECT project_id FROM board.tasks WHERE id = $1")
                .bind(task_id)
                .fetch_optional(&mut *transaction)
                .await
                .context("load external task project")?
                .ok_or_else(|| anyhow!("external task does not exist"))?;

        let mut changed = false;
        for event in &application.events {
            match event {
                BoardEvent::Moved {
                    task_id: event_task_id,
                    from,
                    to,
                    actor,
                    evidence,
                } => {
                    if *event_task_id != task_id {
                        return Err(anyhow!("external sync move belongs to another task"));
                    }
                    let BoardActor::Sync { provider } = actor else {
                        return Err(anyhow!("external sync move has a non-sync actor"));
                    };
                    let external = evidence
                        .iter()
                        .find_map(|evidence| evidence.external.as_ref())
                        .ok_or_else(|| anyhow!("external sync move has no external evidence"))?;
                    let inserted = query_scalar::<_, String>(
                        "INSERT INTO board.external_sync_changes
                         (task_id, change_id, change_kind, occurred_at)
                         VALUES ($1, $2, 'status', $3::timestamptz)
                         ON CONFLICT (task_id, change_kind, change_id) DO NOTHING
                         RETURNING change_id",
                    )
                    .bind(task_id)
                    .bind(&external.change_id)
                    .bind(&external.occurred_at)
                    .fetch_optional(&mut *transaction)
                    .await
                    .context("record external status change")?;
                    if inserted.is_none() {
                        continue;
                    }
                    let current_column: String = query_scalar(
                        "SELECT column_name FROM board.tasks WHERE id = $1 FOR UPDATE",
                    )
                    .bind(task_id)
                    .fetch_optional(&mut *transaction)
                    .await
                    .context("load task column for external move")?
                    .ok_or_else(|| anyhow!("external task does not exist"))?;
                    if current_column != database_task_column(*from) {
                        return Err(anyhow!("external sync move is based on stale local state"));
                    }
                    query(
                        "UPDATE board.tasks
                         SET column_name = $2, updated_at = now()
                         WHERE id = $1",
                    )
                    .bind(task_id)
                    .bind(database_task_column(*to))
                    .execute(&mut *transaction)
                    .await
                    .context("project external status move")?;
                    query(
                        "INSERT INTO board.task_transitions
                         (id, task_id, from_column, to_column, actor_kind, actor_label, evidence)
                         VALUES ($1, $2, $3, $4, 'sync', $5, $6)",
                    )
                    .bind(Uuid::new_v4())
                    .bind(task_id)
                    .bind(database_task_column(*from))
                    .bind(database_task_column(*to))
                    .bind(provider)
                    .bind(serde_json::to_value(evidence)?)
                    .execute(&mut *transaction)
                    .await
                    .context("persist external status transition")?;
                    changed = true;
                }
                BoardEvent::Commented {
                    task_id: event_task_id,
                    comment,
                    actor,
                } => {
                    if *event_task_id != task_id {
                        return Err(anyhow!("external note belongs to another task"));
                    }
                    let BoardActor::Sync { provider } = actor else {
                        return Err(anyhow!("external note has a non-sync actor"));
                    };
                    let BoardCommentOrigin::External {
                        provider: comment_provider,
                        note_id,
                    } = &comment.origin
                    else {
                        return Err(anyhow!("external note has no external origin"));
                    };
                    if comment_provider != provider {
                        return Err(anyhow!("external note provider does not match its actor"));
                    }
                    let inserted = query(
                        "INSERT INTO board.external_sync_changes
                         (task_id, change_id, change_kind, occurred_at)
                         VALUES ($1, $2, 'note', now())
                         ON CONFLICT (task_id, change_kind, change_id) DO NOTHING",
                    )
                    .bind(task_id)
                    .bind(note_id)
                    .execute(&mut *transaction)
                    .await
                    .context("record external note change")?;
                    if inserted.rows_affected() == 0 {
                        continue;
                    }
                    query(
                        "INSERT INTO board.task_comments
                         (id, task_id, author, body, origin, external_provider, external_note_id)
                         VALUES ($1, $2, $3, $4, 'external', $5, $6)
                         ON CONFLICT (task_id, origin, external_provider, external_note_id)
                         DO NOTHING",
                    )
                    .bind(Uuid::new_v4())
                    .bind(task_id)
                    .bind(&comment.author)
                    .bind(&comment.body)
                    .bind(provider)
                    .bind(note_id)
                    .execute(&mut *transaction)
                    .await
                    .context("persist external note")?;
                    changed = true;
                }
                BoardEvent::ExternalSnapshotUpdated {
                    task_id: event_task_id,
                    snapshot: updated_snapshot,
                } => {
                    if *event_task_id != task_id {
                        return Err(anyhow!("external snapshot belongs to another task"));
                    }
                    query(
                        "UPDATE board.external_work_items
                         SET url = $2, title = $3, body = $4, labels = $5, source_status = $6
                         WHERE task_id = $1",
                    )
                    .bind(task_id)
                    .bind(&updated_snapshot.url)
                    .bind(&updated_snapshot.title)
                    .bind(&updated_snapshot.body)
                    .bind(serde_json::to_value(&updated_snapshot.labels)?)
                    .bind(&updated_snapshot.status)
                    .execute(&mut *transaction)
                    .await
                    .context("project external snapshot")?;
                    changed = true;
                }
                BoardEvent::Created { .. }
                | BoardEvent::Assigned { .. }
                | BoardEvent::WorkflowDependency { .. }
                | BoardEvent::RunCompleted { .. } => {
                    return Err(anyhow!("unsupported event in external sync application"));
                }
            }
        }

        for note_id in &application.echo_suppressed_notes {
            query(
                "INSERT INTO board.external_sync_changes
                 (task_id, change_id, change_kind, occurred_at)
                 VALUES ($1, $2, 'note', now())
                 ON CONFLICT (task_id, change_kind, change_id) DO NOTHING",
            )
            .bind(task_id)
            .bind(note_id)
            .execute(&mut *transaction)
            .await
            .context("record echoed external note")?;
        }

        if let Some(change_id) = &application.external_done_change_id {
            let completion_id = Uuid::new_v4();
            let locator = format!("locus://{}/task/{}", local_project_id, task_id);
            let comment = format!(
                "Completed {locator}; delivery satisfied by external close ({change_id})\n<!-- locus-completion:{completion_id} -->"
            );
            let inserted = query(
                "INSERT INTO board.external_completion_outbox
                 (id, task_id, plugin_id, host, provider_project, native_id,
                  comment, locator, evidence, attempts, resolution_supported,
                  commented_at, resolved_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '[]'::jsonb, 0, $9,
                         now(), CASE WHEN $9 THEN now() ELSE NULL END)
                 ON CONFLICT (task_id) DO NOTHING",
            )
            .bind(completion_id)
            .bind(task_id)
            .bind(snapshot.identity.plugin_id.as_str())
            .bind(&snapshot.identity.host)
            .bind(&snapshot.identity.project)
            .bind(&snapshot.identity.native_id)
            .bind(comment)
            .bind(locator)
            .bind(application.resolution_supported)
            .execute(&mut *transaction)
            .await
            .context("satisfy external completion from close")?;
            changed = changed || inserted.rows_affected() == 1;
        }

        let updated = query(
            "UPDATE board.external_work_items
             SET pull_cursor = $2,
                 last_pushed_status = $3,
                 note_watermark = $4,
                 last_local_status_at = $5::timestamptz,
                 last_external_status_at = $6::timestamptz,
                 last_sync_error = $7,
                 last_synced_at = $8::timestamptz,
                 unmapped_external_status = $9,
                 last_conflict_winner = $10,
                 last_conflict_reason = $11
             WHERE task_id = $1",
        )
        .bind(task_id)
        .bind(&state.pull_cursor)
        .bind(&state.last_pushed_status)
        .bind(&state.note_watermark)
        .bind(&state.last_local_status_at)
        .bind(&state.last_external_status_at)
        .bind(&state.last_sync_error)
        .bind(&state.last_synced_at)
        .bind(&state.unmapped_external_status)
        .bind(&state.last_conflict_winner)
        .bind(&state.last_conflict_reason)
        .execute(&mut *transaction)
        .await
        .context("persist external sync cursor")?;
        if updated.rows_affected() != 1 {
            return Err(anyhow!("external work-item disappeared during sync"));
        }
        transaction
            .commit()
            .await
            .context("commit external work-item sync")?;
        Ok(changed)
    }

    pub async fn external_task_is_done(&self, task_id: TaskId) -> Result<bool> {
        Ok(query_scalar::<_, bool>(
            "SELECT tasks.column_name = 'done'
             FROM board.tasks tasks
             JOIN board.external_work_items external_item ON external_item.task_id = tasks.id
             WHERE tasks.id = $1",
        )
        .bind(task_id)
        .fetch_optional(self.pool())
        .await
        .context("check external task Done state")?
        .unwrap_or(false))
    }

    pub async fn persist_imported_task(
        &self,
        task: &BoardTask,
        snapshot: &WorkItemSnapshot,
        workflow: &WorkflowSelection,
    ) -> Result<bool> {
        if task.external_work_item.as_ref() != Some(snapshot) {
            return Err(anyhow!(
                "imported task snapshot does not match its board task"
            ));
        }
        snapshot
            .validate()
            .map_err(|error| anyhow!("invalid imported work-item snapshot: {error}"))?;
        if !workflow.confirmed
            || workflow.task_id != task.id
            || workflow.project_id != task.project_id
        {
            return Err(anyhow!("imported task workflow is not confirmed"));
        }
        let workflow_def_id = workflow
            .workflow_def_id
            .ok_or_else(|| anyhow!("imported task workflow definition is missing"))?;
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("begin external work-item import")?;
        let workflow_project = query_scalar::<_, ProjectId>(
            "SELECT project_id FROM workflows.workflow_defs WHERE id = $1",
        )
        .bind(workflow_def_id)
        .fetch_optional(&mut *transaction)
        .await
        .context("validate imported workflow definition")?;
        if workflow_project != Some(task.project_id) {
            return Err(anyhow!("imported task workflow belongs to another project"));
        }
        let task_inserted = query(
            "INSERT INTO board.tasks
             (id, project_id, repo_id, summary, description, column_name, blocked,
              blocked_reason, blocked_clear_condition, assigned_agent_def_id, session_id,
              verify_command, workflow_def_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(task.id)
        .bind(task.project_id)
        .bind(task.repo_id)
        .bind(&task.summary)
        .bind(&task.description)
        .bind(database_task_column(task.column))
        .bind(task.blocked)
        .bind(&task.blocked_reason)
        .bind(&task.blocked_clear_condition)
        .bind(task.assigned_agent)
        .bind(task.session_id)
        .bind(&task.verify_command)
        .bind(workflow_def_id)
        .execute(&mut *transaction)
        .await
        .context("persist imported board task")?;
        if task_inserted.rows_affected() != 1 {
            transaction
                .rollback()
                .await
                .context("rollback duplicate imported board task")?;
            return Ok(false);
        }
        query(
            "INSERT INTO board.task_transitions
             (id, task_id, from_column, to_column, actor_kind)
             VALUES ($1, $2, NULL, $3, 'system')",
        )
        .bind(Uuid::new_v4())
        .bind(task.id)
        .bind(database_task_column(task.column))
        .execute(&mut *transaction)
        .await
        .context("persist imported task transition")?;

        let external_inserted = query(
            "INSERT INTO board.external_work_items
             (task_id, plugin_id, host, provider_project, native_id, url, title, body, labels, source_status, workflow_def_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             ON CONFLICT (plugin_id, host, provider_project, native_id) DO NOTHING",
        )
        .bind(task.id)
        .bind(snapshot.identity.plugin_id.as_str())
        .bind(&snapshot.identity.host)
        .bind(&snapshot.identity.project)
        .bind(&snapshot.identity.native_id)
        .bind(&snapshot.url)
        .bind(&snapshot.title)
        .bind(&snapshot.body)
        .bind(serde_json::to_value(&snapshot.labels)?)
        .bind(&snapshot.status)
        .bind(workflow_def_id)
        .execute(&mut *transaction)
        .await
        .context("persist external work-item snapshot")?;
        if external_inserted.rows_affected() != 1 {
            transaction
                .rollback()
                .await
                .context("rollback duplicate external work-item")?;
            return Ok(false);
        }
        transaction
            .commit()
            .await
            .context("commit external work-item import")?;
        Ok(true)
    }

    pub async fn persist_external_done_and_completion(
        &self,
        task: &BoardTask,
        from: TaskColumn,
        delivery: &CompletionDelivery,
        snapshot: &WorkItemSnapshot,
    ) -> Result<bool> {
        if task.column != TaskColumn::Done {
            return Err(anyhow!("external task is not ready to enter Done"));
        }
        if task.external_work_item.as_ref() != Some(snapshot) {
            return Err(anyhow!(
                "external task snapshot does not match its board task"
            ));
        }
        snapshot
            .validate()
            .map_err(|error| anyhow!("invalid external completion snapshot: {error}"))?;
        if delivery.event.task_id != task.id {
            return Err(anyhow!("external completion event does not match its task"));
        }
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("begin external completion")?;
        let (current_project_id, current_column) = query_as::<_, (ProjectId, String)>(
            "SELECT project_id, column_name
             FROM board.tasks WHERE id = $1 FOR UPDATE",
        )
        .bind(task.id)
        .fetch_optional(&mut *transaction)
        .await
        .context("load external task state")?
        .ok_or_else(|| anyhow!("external task does not exist"))?;
        if current_project_id != task.project_id {
            return Err(anyhow!("external task project does not match its owner"));
        }
        if delivery.event.locator != format!("locus://{}/task/{}", current_project_id, task.id) {
            return Err(anyhow!("external completion event does not match its task"));
        }
        let persisted_identity = query_as::<_, (String, String, String, String)>(
            "SELECT plugin_id, host, provider_project, native_id
             FROM board.external_work_items
             WHERE task_id = $1 FOR UPDATE",
        )
        .bind(task.id)
        .fetch_optional(&mut *transaction)
        .await
        .context("load persisted external task identity")?
        .ok_or_else(|| anyhow!("external task identity does not exist"))?;
        if persisted_identity.0 != snapshot.identity.plugin_id.as_str()
            || persisted_identity.1 != snapshot.identity.host
            || persisted_identity.2 != snapshot.identity.project
            || persisted_identity.3 != snapshot.identity.native_id
        {
            return Err(anyhow!(
                "external completion identity does not match persisted task"
            ));
        }
        if let Some(existing_event_id) = query_scalar::<_, Uuid>(
            "SELECT id FROM board.external_completion_outbox WHERE task_id = $1 FOR UPDATE",
        )
        .bind(task.id)
        .fetch_optional(&mut *transaction)
        .await
        .context("load existing external completion")?
        {
            if existing_event_id != delivery.event.id {
                return Err(anyhow!(
                    "external completion event identity does not match persisted event"
                ));
            }
        }

        let moved_rows = if current_column == database_task_column(TaskColumn::Done) {
            0
        } else if current_column != database_task_column(from) {
            return Err(anyhow!("external task is not ready to enter Done"));
        } else {
            query(
                "UPDATE board.tasks
                 SET column_name = 'done', updated_at = now()
                 WHERE id = $1",
            )
            .bind(task.id)
            .execute(&mut *transaction)
            .await
            .context("persist external task Done transition")?;
            query(
                "INSERT INTO board.task_transitions
                 (id, task_id, from_column, to_column, actor_kind)
                 VALUES ($1, $2, $3, 'done', 'human')",
            )
            .bind(Uuid::new_v4())
            .bind(task.id)
            .bind(database_task_column(from))
            .execute(&mut *transaction)
            .await
            .context("persist external task transition")?;
            1
        };

        let inserted_rows =
            insert_external_completion(&mut transaction, delivery, snapshot).await?;
        transaction
            .commit()
            .await
            .context("commit external completion")?;
        Ok(moved_rows == 1 || inserted_rows == 1)
    }

    pub async fn enqueue_external_completion(
        &self,
        delivery: &CompletionDelivery,
        snapshot: &WorkItemSnapshot,
    ) -> Result<bool> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("begin external completion enqueue")?;
        let inserted_rows =
            insert_external_completion(&mut transaction, delivery, snapshot).await?;
        transaction
            .commit()
            .await
            .context("commit external completion enqueue")?;
        Ok(inserted_rows == 1)
    }

    pub async fn load_external_work_items(&self) -> Result<Vec<PersistedExternalWorkItem>> {
        let rows = query_as::<_, PersistedExternalWorkItemRow>(
            "SELECT t.id, t.project_id, t.repo_id, t.session_id, t.summary, t.description,
                    t.column_name, t.blocked, t.blocked_reason, t.blocked_clear_condition,
                    t.assigned_agent_def_id, t.verify_command, e.workflow_def_id,
                    workflow.project_id, e.plugin_id, e.host, e.provider_project, e.native_id,
                    e.url, e.title, e.body, e.labels, e.source_status,
                    e.pull_cursor, e.last_pushed_status, e.note_watermark,
                    to_char(e.last_local_status_at AT TIME ZONE 'UTC',
                            'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS last_local_status_at,
                    to_char(e.last_external_status_at AT TIME ZONE 'UTC',
                            'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS last_external_status_at,
                    e.last_sync_error,
                    to_char(e.last_synced_at AT TIME ZONE 'UTC',
                            'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS last_synced_at,
                    e.unmapped_external_status,
                    e.last_conflict_winner,
                    e.last_conflict_reason
             FROM board.external_work_items e
             JOIN board.tasks t ON t.id = e.task_id
             JOIN workflows.workflow_defs workflow ON workflow.id = e.workflow_def_id
             ORDER BY e.imported_at, e.task_id",
        )
        .fetch_all(self.pool())
        .await
        .context("load external work-item snapshots")?;

        let mut items = rows
            .into_iter()
            .map(|row| {
                let task_id = row.id;
                let project_id = row.project_id;
                if row.workflow_project_id != project_id {
                    return Err(anyhow!("external task workflow belongs to another project"));
                }
                let plugin_id = WorkItemProviderId::new(row.plugin_id)
                    .map_err(|error| anyhow!("invalid external provider id: {error}"))?;
                let labels = serde_json::from_value(row.labels)
                    .context("decode external work-item labels")?;
                let snapshot = WorkItemSnapshot {
                    identity: WorkItemIdentity {
                        plugin_id,
                        host: row.host,
                        project: row.provider_project,
                        native_id: row.native_id,
                    },
                    url: row.url,
                    title: row.title,
                    body: row.body,
                    labels,
                    status: row.source_status,
                };
                snapshot
                    .validate()
                    .context("validate external work-item snapshot")?;
                let task = BoardTask {
                    id: task_id,
                    project_id,
                    repo_id: row.repo_id,
                    session_id: row.session_id,
                    summary: row.summary,
                    description: row.description,
                    column: task_column(&row.column_name)?,
                    blocked: row.blocked,
                    blocked_reason: row.blocked_reason,
                    blocked_clear_condition: row.blocked_clear_condition,
                    assigned_agent: row.assigned_agent_def_id,
                    blocked_by: Default::default(),
                    verify_command: row.verify_command,
                    evidence: Vec::new(),
                    comments: Vec::new(),
                    external_issue: None,
                    external_work_item: Some(snapshot.clone()),
                };
                Ok(PersistedExternalWorkItem {
                    task,
                    snapshot,
                    workflow: WorkflowSelection {
                        task_id,
                        project_id,
                        workflow_def_id: Some(row.workflow_def_id),
                        confirmed: true,
                    },
                    runs: Vec::new(),
                    evidence: Vec::new(),
                    sync_state: WorkItemSyncState {
                        pull_cursor: row.pull_cursor,
                        last_pushed_status: row.last_pushed_status,
                        note_watermark: row.note_watermark,
                        last_local_status_at: row.last_local_status_at,
                        last_external_status_at: row.last_external_status_at,
                        last_sync_error: row.last_sync_error,
                        last_synced_at: row.last_synced_at,
                        unmapped_external_status: row.unmapped_external_status,
                        last_conflict_winner: row.last_conflict_winner,
                        last_conflict_reason: row.last_conflict_reason,
                    },
                })
            })
            .collect::<Result<Vec<_>>>()?;

        for item in &mut items {
            let task_id = item.task.id;
            let run_rows = query_as::<_, (RunId, SessionId, Option<RunId>)>(
                "SELECT run.id, run.session_id, run_edge.parent_run_id
                 FROM board.task_runs task_run
                 JOIN agents.runs run ON run.id = task_run.run_id
                 LEFT JOIN LATERAL (
                     SELECT parent_run_id
                     FROM agents.run_edges
                     WHERE child_run_id = run.id
                     ORDER BY created_at, parent_run_id
                     LIMIT 1
                 ) run_edge ON TRUE
                 WHERE task_run.task_id = $1
                 ORDER BY task_run.linked_at, task_run.run_id",
            )
            .bind(task_id)
            .fetch_all(self.pool())
            .await
            .context("load imported task runs")?;
            item.runs = run_rows
                .into_iter()
                .map(|(run_id, root_session_id, parent_run_id)| TaskRunLink {
                    task_id,
                    root_session_id,
                    run_id,
                    parent_run_id,
                })
                .collect();

            let evidence_rows = query_as::<_, (RunId, Option<Uuid>, Uuid)>(
                "SELECT run_id, event_id, id
                 FROM board.task_evidence
                 WHERE task_id = $1
                 ORDER BY created_at, id",
            )
            .bind(task_id)
            .fetch_all(self.pool())
            .await
            .context("load imported task evidence")?;
            item.evidence = evidence_rows
                .into_iter()
                .map(|(run_id, event_id, evidence_id)| TaskEvidenceLink {
                    run_id,
                    event_ids: event_id.into_iter().collect(),
                    artifact_ids: vec![evidence_id],
                })
                .collect();
            item.task.evidence = item
                .evidence
                .iter()
                .map(|evidence| BoardEvidenceLink {
                    run_id: Some(evidence.run_id),
                    event_ids: evidence
                        .event_ids
                        .iter()
                        .copied()
                        .map(crate::ids::EventId::from)
                        .collect(),
                    artifact_ids: evidence
                        .artifact_ids
                        .iter()
                        .copied()
                        .map(crate::ids::ArtifactId::from)
                        .collect(),
                    external: None,
                })
                .collect();
            item.task.blocked_by = query_as::<_, (TaskId,)>(
                "SELECT blocked_by_task_id
                 FROM board.task_dependencies
                 WHERE task_id = $1
                 ORDER BY blocked_by_task_id",
            )
            .bind(task_id)
            .fetch_all(self.pool())
            .await
            .context("load imported task dependencies")?
            .into_iter()
            .map(|(blocked_by,)| blocked_by)
            .collect();

            let comment_rows =
                query_as::<_, (String, String, String, Option<String>, Option<String>)>(
                    "SELECT author, body, origin, external_provider, external_note_id
                 FROM board.task_comments
                 WHERE task_id = $1
                 ORDER BY created_at, id",
                )
                .bind(task_id)
                .fetch_all(self.pool())
                .await
                .context("load imported task comments")?;
            item.task.comments = comment_rows
                .into_iter()
                .map(|(author, body, origin, provider, note_id)| {
                    let origin = match origin.as_str() {
                        "local" => BoardCommentOrigin::Local,
                        "external" => BoardCommentOrigin::External {
                            provider: provider
                                .ok_or_else(|| anyhow!("external comment provider is missing"))?,
                            note_id: note_id
                                .ok_or_else(|| anyhow!("external comment id is missing"))?,
                        },
                        other => return Err(anyhow!("invalid persisted comment origin `{other}`")),
                    };
                    Ok(BoardComment {
                        author,
                        body,
                        origin,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
        }
        Ok(items)
    }

    pub async fn load_external_completions(&self) -> Result<Vec<PersistedExternalCompletion>> {
        let rows = query_as::<
            _,
            (
                Uuid,
                TaskId,
                String,
                String,
                String,
                String,
                String,
                String,
                Value,
                i32,
                bool,
                bool,
                bool,
            ),
        >(
            "SELECT id, task_id, plugin_id, host, provider_project, native_id, comment, locator,
                    evidence, attempts, commented_at IS NOT NULL, resolution_supported,
                    resolved_at IS NOT NULL
             FROM board.external_completion_outbox
             ORDER BY created_at, id",
        )
        .fetch_all(self.pool())
        .await
        .context("load external completions")?;

        rows.into_iter()
            .map(
                |(
                    id,
                    task_id,
                    plugin_id,
                    host,
                    project,
                    native_id,
                    comment,
                    locator,
                    evidence,
                    attempts,
                    commented,
                    resolution_supported,
                    resolved,
                )| {
                    let plugin_id = WorkItemProviderId::new(plugin_id)
                        .map_err(|error| anyhow!("invalid external provider id: {error}"))?;
                    let evidence = serde_json::from_value(evidence)
                        .context("decode external completion evidence")?;
                    Ok(PersistedExternalCompletion {
                        id,
                        task_id,
                        identity: WorkItemIdentity {
                            plugin_id,
                            host,
                            project,
                            native_id,
                        },
                        comment,
                        locator,
                        evidence,
                        attempts: u32::try_from(attempts)
                            .context("stored completion attempts are negative")?,
                        commented,
                        resolved: resolution_supported.then_some(resolved),
                    })
                },
            )
            .collect()
    }

    pub async fn external_completion_status(
        &self,
        task_id: TaskId,
    ) -> Result<Option<PersistedExternalCompletionStatus>> {
        let row = query_as::<_, (i32, bool, bool, bool, Option<String>)>(
            "SELECT attempts, commented_at IS NOT NULL, resolution_supported,
                    resolved_at IS NOT NULL, last_error
             FROM board.external_completion_outbox
             WHERE task_id = $1",
        )
        .bind(task_id)
        .fetch_optional(self.pool())
        .await
        .context("load external completion status")?;
        row.map(
            |(attempts, commented, resolution_supported, resolved, last_error)| {
                let status = if last_error.is_some() {
                    "failed"
                } else if resolved && resolution_supported {
                    "resolved"
                } else if commented {
                    "commented"
                } else {
                    "pending"
                };
                Ok(PersistedExternalCompletionStatus {
                    attempts: u32::try_from(attempts)
                        .context("stored completion attempts are negative")?,
                    commented,
                    resolved: resolution_supported.then_some(resolved),
                    resolution_supported,
                    last_error,
                    status: status.into(),
                })
            },
        )
        .transpose()
    }

    pub async fn pending_external_completions(&self) -> Result<Vec<PersistedExternalCompletion>> {
        Ok(self
            .load_external_completions()
            .await?
            .into_iter()
            .filter(|item| !item.commented || item.resolved == Some(false))
            .collect())
    }

    pub async fn restore_pending_external_completions(
        &self,
        outbox: &mut CompletionOutbox,
    ) -> Result<usize> {
        let pending = self.pending_external_completions().await?;
        let mut restored = 0;
        for item in pending {
            outbox
                .restore_delivery(CompletionDelivery {
                    event: CompletionEvent {
                        id: item.id,
                        task_id: item.task_id,
                        locator: item.locator,
                        evidence: item.evidence,
                        comment: item.comment,
                    },
                    attempts: item.attempts,
                    commented: item.commented,
                    resolved: item.resolved,
                })
                .map_err(|error| anyhow!("restore external completion: {error}"))?;
            restored += 1;
        }
        Ok(restored)
    }

    pub async fn record_external_completion_attempt(
        &self,
        event_id: uuid::Uuid,
        attempts: i32,
        commented: bool,
        resolved: bool,
        error: Option<&str>,
    ) -> Result<()> {
        query(
            "UPDATE board.external_completion_outbox
             SET attempts = $2,
                 commented_at = CASE WHEN $3 THEN COALESCE(commented_at, now()) ELSE commented_at END,
                 resolved_at = CASE WHEN $4 THEN COALESCE(resolved_at, now()) ELSE resolved_at END,
                 last_error = $5
             WHERE id = $1",
        )
        .bind(event_id)
        .bind(attempts)
        .bind(commented)
        .bind(resolved)
        .bind(error)
        .execute(self.pool())
        .await
        .context("record external completion attempt")?;
        Ok(())
    }
}

#[cfg(test)]
mod external_sync_state {
    #[test]
    fn external_sync_state() {
        let migration = include_str!("../../../../migrations/0026_external_work_item_sync.up.sql");
        for column in [
            "pull_cursor",
            "last_pushed_status",
            "note_watermark",
            "last_local_status_at",
            "last_external_status_at",
            "last_sync_error",
            "last_synced_at",
            "unmapped_external_status",
        ] {
            assert!(
                migration.contains(column),
                "sync migration contains {column}"
            );
        }
    }
}
