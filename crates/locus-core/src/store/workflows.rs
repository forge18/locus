//! Atomic persistence for immutable compiled workflow definitions.

use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::{services::workflow::CompiledWorkflow, store::Store};

#[derive(Clone, Debug, PartialEq)]
pub struct PersistedWorkflowDefinition {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub version: i32,
    pub graph: Value,
    pub spec: Value,
    pub verify_command: String,
}

/// A small synchronous seam for compiler tests and non-Postgres callers. The production Store
/// implementation below uses the same all-fields-at-once contract inside one SQL transaction.
pub trait WorkflowDefinitionStore {
    fn save_workflow_definition(
        &mut self,
        project_id: Uuid,
        name: &str,
        compiled: &CompiledWorkflow,
    ) -> Result<PersistedWorkflowDefinition>;
}

#[derive(Default)]
pub struct InMemoryWorkflowDefinitions {
    rows: BTreeMap<(Uuid, String, i32), PersistedWorkflowDefinition>,
}

impl InMemoryWorkflowDefinitions {
    pub fn definitions(&self) -> impl Iterator<Item = &PersistedWorkflowDefinition> {
        self.rows.values()
    }
}

impl WorkflowDefinitionStore for InMemoryWorkflowDefinitions {
    fn save_workflow_definition(
        &mut self,
        project_id: Uuid,
        name: &str,
        compiled: &CompiledWorkflow,
    ) -> Result<PersistedWorkflowDefinition> {
        if name.trim().is_empty() {
            bail!("workflow name is required")
        }
        let version = self
            .rows
            .keys()
            .filter(|(project, workflow, _)| project == &project_id && workflow == name)
            .map(|(_, _, version)| *version)
            .max()
            .unwrap_or(0)
            + 1;
        let mut spec = compiled.persisted_spec();
        spec["version"] = serde_json::json!(version);
        let row = PersistedWorkflowDefinition {
            id: Uuid::new_v4(),
            project_id,
            name: name.to_owned(),
            version,
            graph: compiled.graph().clone(),
            spec,
            verify_command: compiled.verify_command().to_owned(),
        };
        self.rows
            .insert((project_id, name.to_owned(), version), row.clone());
        Ok(row)
    }
}

impl Store {
    /// Save graph, derived spec, version, and derived verify command atomically. Definitions are
    /// append-only; the advisory lock follows the agent-definition versioning convention.
    pub async fn save_workflow_definition(
        &self,
        project_id: Uuid,
        name: &str,
        compiled: &CompiledWorkflow,
    ) -> Result<PersistedWorkflowDefinition> {
        if name.trim().is_empty() {
            bail!("workflow name is required")
        }
        let graph = compiled.graph().clone();
        let mut spec = compiled.persisted_spec();
        let verify_command = compiled.verify_command().to_owned();
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("begin workflow definition")?;
        let lock_key = format!("{project_id}:{name}");
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(lock_key)
            .execute(&mut *transaction)
            .await
            .context("lock workflow definition version")?;
        let version: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) + 1
             FROM workflows.workflow_defs WHERE project_id = $1 AND name = $2",
        )
        .bind(project_id)
        .bind(name)
        .fetch_one(&mut *transaction)
        .await
        .context("allocate workflow definition version")?;
        spec["version"] = serde_json::json!(version);
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO workflows.workflow_defs
                (id, project_id, name, version, graph, spec, verify_command)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(id)
        .bind(project_id)
        .bind(name)
        .bind(version)
        .bind(&graph)
        .bind(&spec)
        .bind(&verify_command)
        .execute(&mut *transaction)
        .await
        .context("insert workflow definition")?;
        transaction
            .commit()
            .await
            .context("commit workflow definition")?;
        Ok(PersistedWorkflowDefinition {
            id,
            project_id,
            name: name.to_owned(),
            version,
            graph,
            spec,
            verify_command,
        })
    }
}
