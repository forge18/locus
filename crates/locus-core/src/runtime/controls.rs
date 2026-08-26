//! ACP session controls shared by the Agent Pane and the run supervisor.
//!
//! These are projections and control decisions over the ACP stream. They do not add verbs to the
//! canonical telemetry vocabulary and they never replace the append-only event log.

use super::invoke::{InvocationContext, InvocationLimits, InvocationRequest, InvocationSupervisor};
use crate::{
    ids::{RunId, SessionId, TurnId},
    services::telemetry::Event,
};
use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use uuid::Uuid;

/// The permission behavior selected when a run is dispatched.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionPosture {
    /// The harness gate is disabled; an unexpected request is an alarm.
    #[default]
    Bypass,
    /// A protected request is a visible, replayable human-action gate.
    Gated,
}

impl PermissionPosture {
    pub const fn is_gated(self) -> bool {
        matches!(self, Self::Gated)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bypass => "bypass",
            Self::Gated => "gated",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "bypass" => Ok(Self::Bypass),
            "gated" => Ok(Self::Gated),
            other => bail!("unknown permission posture `{other}`"),
        }
    }
}

/// The lifecycle shown for one item in the one active session plan.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanItemStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl PlanItemStatus {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "in_progress" | "in-progress" => Ok(Self::InProgress),
            "completed" | "complete" => Ok(Self::Completed),
            "failed" | "error" => Ok(Self::Failed),
            "cancelled" | "canceled" => Ok(Self::Cancelled),
            other => bail!("unknown plan item status `{other}`"),
        }
    }
}

/// One item in an ACP plan projection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PlanItem {
    pub id: String,
    pub title: String,
    pub status: PlanItemStatus,
    pub priority: Option<String>,
}

impl PlanItem {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Result<Self> {
        let id = id.into();
        let title = title.into();
        ensure!(!id.trim().is_empty(), "plan item id must not be empty");
        ensure!(
            !title.trim().is_empty(),
            "plan item title must not be empty"
        );
        Ok(Self {
            id,
            title,
            status: PlanItemStatus::Pending,
            priority: None,
        })
    }

    pub fn with_status(mut self, status: PlanItemStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_priority(mut self, priority: impl Into<String>) -> Self {
        self.priority = Some(priority.into());
        self
    }
}

/// The three ACP plan representations. A session owns at most one active plan id.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PlanUpdate {
    pub plan_id: String,
    pub items: Vec<PlanItem>,
    pub markdown: Option<String>,
    pub file: Option<String>,
}

impl PlanUpdate {
    pub fn new(plan_id: impl Into<String>, items: Vec<PlanItem>) -> Result<Self> {
        let plan_id = plan_id.into();
        ensure!(!plan_id.trim().is_empty(), "plan id must not be empty");
        ensure!(
            markdown_or_items(&items, None, None),
            "plan must contain items or content"
        );
        Ok(Self {
            plan_id,
            items,
            markdown: None,
            file: None,
        })
    }

    pub fn markdown(plan_id: impl Into<String>, markdown: impl Into<String>) -> Result<Self> {
        let plan_id = plan_id.into();
        let markdown = markdown.into();
        ensure!(!plan_id.trim().is_empty(), "plan id must not be empty");
        ensure!(
            !markdown.trim().is_empty(),
            "plan markdown must not be empty"
        );
        Ok(Self {
            plan_id,
            items: Vec::new(),
            markdown: Some(markdown),
            file: None,
        })
    }

    pub fn file(plan_id: impl Into<String>, file: impl Into<String>) -> Result<Self> {
        let plan_id = plan_id.into();
        let file = file.into();
        ensure!(!plan_id.trim().is_empty(), "plan id must not be empty");
        ensure!(!file.trim().is_empty(), "plan file must not be empty");
        Ok(Self {
            plan_id,
            items: Vec::new(),
            markdown: None,
            file: Some(file),
        })
    }

    /// Decode an ACP `session/update` plan payload without making it a telemetry verb.
    pub fn from_value(value: &Value) -> Result<Self> {
        let object = value.as_object().context("plan update must be an object")?;
        let source = object
            .get("planUpdate")
            .or_else(|| object.get("plan_update"))
            .or_else(|| object.get("update"))
            .and_then(Value::as_object)
            .unwrap_or(object);
        let plan_id = string_field(source, &["planId", "plan_id", "id"])
            .context("plan update requires planId")?;
        let markdown = string_field(source, &["markdown", "content"]);
        let file = string_field(source, &["file", "filePath", "file_path"]);
        let items = source
            .get("items")
            .or_else(|| source.get("plan"))
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(parse_plan_item)
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default();
        ensure!(
            markdown_or_items(&items, markdown.as_deref(), file.as_deref()),
            "plan must contain items or content"
        );
        Ok(Self {
            plan_id,
            items,
            markdown,
            file,
        })
    }
}

fn markdown_or_items(items: &[PlanItem], markdown: Option<&str>, file: Option<&str>) -> bool {
    !items.is_empty()
        || markdown.is_some_and(|value| !value.trim().is_empty())
        || file.is_some_and(|value| !value.trim().is_empty())
}

fn string_field(object: &Map<String, Value>, names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|name| object.get(*name).and_then(Value::as_str).map(str::to_owned))
}

fn parse_plan_item(value: &Value) -> Result<PlanItem> {
    if let Some(title) = value.as_str() {
        return PlanItem::new(title, title);
    }
    let object = value
        .as_object()
        .context("plan items must be strings or objects")?;
    let id = string_field(object, &["id", "itemId", "item_id"])
        .or_else(|| string_field(object, &["title", "content"]))
        .context("plan item requires id")?;
    let title =
        string_field(object, &["title", "content", "description"]).unwrap_or_else(|| id.clone());
    let status = string_field(object, &["status", "state"])
        .map(|value| PlanItemStatus::parse(&value))
        .transpose()?
        .unwrap_or_default();
    let priority = string_field(object, &["priority"]);
    Ok(PlanItem {
        id,
        title,
        status,
        priority,
    })
}

/// The current one-plan projection owned by a session.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ActivePlan {
    pub plan_id: String,
    pub items: Vec<PlanItem>,
    pub markdown: Option<String>,
    pub file: Option<String>,
    pub revision: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PlanProjection {
    active: Option<ActivePlan>,
}

impl PlanProjection {
    pub fn active(&self) -> Option<&ActivePlan> {
        self.active.as_ref()
    }

    /// Replaces the session's active plan. There is never a second plan in the projection.
    pub fn apply(&mut self, update: PlanUpdate) -> &ActivePlan {
        let revision = self.active.as_ref().map_or(0, |plan| plan.revision) + 1;
        self.active = Some(ActivePlan {
            plan_id: update.plan_id,
            items: update.items,
            markdown: update.markdown,
            file: update.file,
            revision,
        });
        self.active.as_ref().expect("active plan was just set")
    }

    pub fn clear(&mut self) {
        self.active = None;
    }
}

/// Primitive property types permitted by the client-side elicitation form.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ElicitationType {
    String,
    Integer,
    Number,
    Boolean,
}

/// A restricted, flat elicitation property. Arrays and nested objects are deliberately excluded.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ElicitationProperty {
    #[serde(rename = "type")]
    pub kind: ElicitationType,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "enum", default)]
    pub choices: Vec<Value>,
    #[serde(default)]
    pub default: Option<Value>,
    pub format: Option<String>,
}

impl ElicitationProperty {
    fn validate(&self, name: &str) -> Result<()> {
        if self
            .choices
            .iter()
            .any(|choice| !matches!(choice, Value::String(_) | Value::Number(_) | Value::Bool(_)))
        {
            bail!("elicitation property `{name}` has a non-primitive enum value")
        }
        if let Some(default) = &self.default {
            validate_primitive(self.kind, default)
                .with_context(|| format!("invalid default for elicitation property `{name}`"))?;
            if !self.choices.is_empty() && !self.choices.iter().any(|choice| choice == default) {
                bail!("default for elicitation property `{name}` is not in enum")
            }
        }
        if self.format.as_deref() == Some("uri") {
            bail!("URL elicitation values must use URL mode")
        }
        Ok(())
    }
}

/// A flat object schema accepted by the elicitation client.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ElicitationSchema {
    pub properties: BTreeMap<String, ElicitationProperty>,
    #[serde(default)]
    pub required: BTreeSet<String>,
}

pub type RestrictedElicitationSchema = ElicitationSchema;

impl ElicitationSchema {
    pub fn from_value(value: &Value) -> Result<Self> {
        let object = value
            .as_object()
            .context("elicitation schema must be an object")?;
        ensure!(
            object.get("type").and_then(Value::as_str) == Some("object"),
            "elicitation schema must have object type"
        );
        let properties = object
            .get("properties")
            .and_then(Value::as_object)
            .context("elicitation schema requires properties")?
            .iter()
            .map(|(name, value)| {
                let property: ElicitationProperty = serde_json::from_value(value.clone())
                    .with_context(|| format!("invalid elicitation property `{name}`"))?;
                property.validate(name)?;
                Ok((name.clone(), property))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        ensure!(
            properties.len() <= 32,
            "elicitation schema has too many properties"
        );
        let required = object
            .get("required")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .map(|value| {
                        value
                            .as_str()
                            .map(str::to_owned)
                            .context("required names must be strings")
                    })
                    .collect::<Result<BTreeSet<_>>>()
            })
            .transpose()?
            .unwrap_or_default();
        ensure!(
            required.iter().all(|name| properties.contains_key(name)),
            "elicitation required field is not declared"
        );
        Ok(Self {
            properties,
            required,
        })
    }

    pub fn validate_values(&self, values: &Map<String, Value>) -> Result<()> {
        ensure!(
            values.keys().all(|name| self.properties.contains_key(name)),
            "elicitation response contains an unknown field"
        );
        for (name, property) in &self.properties {
            let Some(value) = values.get(name) else {
                if self.required.contains(name) && property.default.is_none() {
                    bail!("elicitation response is missing required field `{name}`")
                }
                continue;
            };
            validate_primitive(property.kind, value)
                .with_context(|| format!("invalid value for elicitation property `{name}`"))?;
            if !property.choices.is_empty()
                && !property.choices.iter().any(|choice| choice == value)
            {
                bail!("value for elicitation property `{name}` is not in enum")
            }
        }
        Ok(())
    }

    pub fn with_defaults(&self, values: &Map<String, Value>) -> Map<String, Value> {
        let mut result: Map<String, Value> = self
            .properties
            .iter()
            .filter_map(|(name, property)| {
                property.default.clone().map(|value| (name.clone(), value))
            })
            .collect();
        result.extend(values.clone());
        result
    }
}

fn validate_primitive(kind: ElicitationType, value: &Value) -> Result<()> {
    let valid = match kind {
        ElicitationType::String => value.is_string(),
        ElicitationType::Integer => value.as_i64().is_some() || value.as_u64().is_some(),
        ElicitationType::Number => value.is_number(),
        ElicitationType::Boolean => value.is_boolean(),
    };
    ensure!(valid, "value does not match elicitation property type");
    Ok(())
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ElicitationMode {
    Form,
    Url,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ElicitationRequest {
    pub request_id: String,
    pub mode: ElicitationMode,
    pub message: String,
    pub schema: Option<ElicitationSchema>,
    pub defaults: Map<String, Value>,
}

impl ElicitationRequest {
    pub fn form(
        request_id: impl Into<String>,
        message: impl Into<String>,
        schema: ElicitationSchema,
    ) -> Result<Self> {
        let request_id = request_id.into();
        ensure!(
            !request_id.trim().is_empty(),
            "elicitation request id must not be empty"
        );
        Ok(Self {
            request_id,
            mode: ElicitationMode::Form,
            message: message.into(),
            schema: Some(schema),
            defaults: Map::new(),
        })
    }

    pub fn url(request_id: impl Into<String>, message: impl Into<String>) -> Result<Self> {
        let request_id = request_id.into();
        ensure!(
            !request_id.trim().is_empty(),
            "elicitation request id must not be empty"
        );
        Ok(Self {
            request_id,
            mode: ElicitationMode::Url,
            message: message.into(),
            schema: None,
            defaults: Map::new(),
        })
    }

    pub fn with_defaults(mut self, defaults: Map<String, Value>) -> Self {
        self.defaults = defaults;
        self
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ElicitationAction {
    Accept,
    Decline,
    Cancel,
}

pub type ElicitationResponseAction = ElicitationAction;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ElicitationResponse {
    pub request_id: String,
    pub action: ElicitationAction,
    #[serde(default)]
    pub values: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElicitationResult {
    Accepted(Map<String, Value>),
    Declined,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElicitationRecord {
    pub request_id: String,
    pub result: ElicitationResult,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ElicitationHistory {
    records: Vec<ElicitationRecord>,
}

impl ElicitationHistory {
    pub fn respond(
        &mut self,
        request: &ElicitationRequest,
        response: ElicitationResponse,
    ) -> Result<ElicitationResult> {
        ensure!(
            response.request_id == request.request_id,
            "elicitation response id does not match request"
        );
        let result = match response.action {
            ElicitationAction::Accept => {
                let schema = request
                    .schema
                    .as_ref()
                    .context("form request has no schema")?;
                let values = schema
                    .with_defaults(&request.defaults)
                    .into_iter()
                    .chain(response.values)
                    .collect();
                schema.validate_values(&values)?;
                ElicitationResult::Accepted(values)
            }
            ElicitationAction::Decline => ElicitationResult::Declined,
            ElicitationAction::Cancel => ElicitationResult::Cancelled,
        };
        self.records.push(ElicitationRecord {
            request_id: request.request_id.clone(),
            result: result.clone(),
        });
        Ok(result)
    }

    pub fn records(&self) -> &[ElicitationRecord] {
        &self.records
    }
}

/// Commands that operate on a Locus session rather than on the agent's prompt text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionCommand {
    NewSession,
    Compact,
    ClearContext,
    ViewContext,
}

impl SessionCommand {
    pub fn parse(input: &str) -> Option<Self> {
        let command = input.trim().strip_prefix('/').unwrap_or(input.trim());
        match command.split_whitespace().next()? {
            "new-session" | "new" => Some(Self::NewSession),
            "compact" => Some(Self::Compact),
            "clear-context" | "clear" => Some(Self::ClearContext),
            "context" | "view-context" => Some(Self::ViewContext),
            _ => None,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::NewSession => "new-session",
            Self::Compact => "compact",
            Self::ClearContext => "clear-context",
            Self::ViewContext => "context",
        }
    }
}

/// A small, serializable view of the context shown by the panel.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ContextView {
    pub session_id: Option<SessionId>,
    pub event_count: usize,
    pub memory_keys: Vec<String>,
}

impl ContextView {
    pub fn from_session(session_id: SessionId, memory: &Value, event_count: usize) -> Self {
        let mut memory_keys = memory
            .as_object()
            .map(|object| object.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        memory_keys.sort();
        Self {
            session_id: Some(session_id),
            event_count,
            memory_keys,
        }
    }
}

/// The turn-boundary controller. Steering is queued; stopping never consumes queued steering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SteeringBoundary {
    active_turn: Option<TurnId>,
    queued: VecDeque<String>,
    cancel_requested: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnCompletion {
    pub turn_id: TurnId,
    pub cancelled: bool,
    pub next_steer: Option<String>,
}

impl SteeringBoundary {
    pub fn begin_turn(&mut self, turn_id: TurnId) -> Option<String> {
        self.active_turn = Some(turn_id);
        self.cancel_requested = false;
        self.queued.pop_front()
    }

    pub fn queue_steer(&mut self, prompt: impl Into<String>) -> Result<()> {
        let prompt = prompt.into();
        ensure!(
            !prompt.trim().is_empty(),
            "steering prompt must not be empty"
        );
        self.queued.push_back(prompt);
        Ok(())
    }

    pub fn stop_active_turn(&mut self) -> bool {
        if self.active_turn.is_some() {
            self.cancel_requested = true;
            true
        } else {
            false
        }
    }

    pub fn finish_turn(&mut self) -> Option<TurnCompletion> {
        let turn_id = self.active_turn.take()?;
        let cancelled = self.cancel_requested;
        self.cancel_requested = false;
        Some(TurnCompletion {
            turn_id,
            cancelled,
            next_steer: self.queued.pop_front(),
        })
    }

    pub fn queued_count(&self) -> usize {
        self.queued.len()
    }
}

/// Request for a child run created directly from the Agent Pane.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PanelSubagentRequest {
    pub caller_run_id: RunId,
    pub agent: String,
    pub version: i32,
    pub clone_remote: String,
    pub context: InvocationContext,
    pub limits: InvocationLimits,
}

impl PanelSubagentRequest {
    pub fn new(
        caller_run_id: RunId,
        agent: impl Into<String>,
        version: i32,
        clone_remote: impl Into<String>,
        context: InvocationContext,
    ) -> Self {
        Self {
            caller_run_id,
            agent: agent.into(),
            version,
            clone_remote: clone_remote.into(),
            context,
            limits: InvocationLimits::HARD,
        }
    }

    pub fn with_limits(mut self, limits: InvocationLimits) -> Self {
        self.limits = limits;
        self
    }
}

/// Uses the same bounded invocation path as `locus agent invoke`.
pub fn invoke_panel_subagent<'launcher, Launcher>(
    supervisor: &InvocationSupervisor<'launcher, Launcher>,
    request: PanelSubagentRequest,
) -> Result<super::invoke::NestedRunPlan>
where
    Launcher: super::invoke::NestedRunLauncher,
{
    supervisor.invoke(InvocationRequest {
        caller_run_id: request.caller_run_id,
        agent: request.agent,
        version: request.version,
        clone_remote: request.clone_remote,
        context: request.context,
        limits: request.limits,
    })
}

/// A workspace snapshot captured immediately before an edit.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct WorkspaceSnapshot {
    pub branch: String,
    pub files: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Checkpoint {
    pub id: Uuid,
    pub run_id: RunId,
    pub ordinal: u64,
    pub workspace: WorkspaceSnapshot,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckpointLedger {
    checkpoints: Vec<Checkpoint>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RestoreResult {
    pub workspace: WorkspaceSnapshot,
    /// The transcript is returned unchanged; restore is not transcript deletion.
    pub transcript: Vec<Event>,
}

impl CheckpointLedger {
    pub fn snapshot_before_edit(
        &mut self,
        run_id: RunId,
        workspace: WorkspaceSnapshot,
    ) -> Checkpoint {
        let checkpoint = Checkpoint {
            id: Uuid::new_v4(),
            run_id,
            ordinal: self.checkpoints.len() as u64,
            workspace,
        };
        self.checkpoints.push(checkpoint.clone());
        checkpoint
    }

    pub fn restore(&self, checkpoint_id: Uuid, transcript: &[Event]) -> Result<RestoreResult> {
        let checkpoint = self
            .checkpoints
            .iter()
            .find(|checkpoint| checkpoint.id == checkpoint_id)
            .context("checkpoint not found")?;
        Ok(RestoreResult {
            workspace: checkpoint.workspace.clone(),
            transcript: transcript.to_vec(),
        })
    }

    pub fn undo(&mut self, transcript: &[Event]) -> Result<RestoreResult> {
        let checkpoint = self.checkpoints.pop().context("no checkpoint to undo")?;
        Ok(RestoreResult {
            workspace: checkpoint.workspace,
            transcript: transcript.to_vec(),
        })
    }

    pub fn checkpoints(&self) -> &[Checkpoint] {
        &self.checkpoints
    }
}

/// State handed to a newly attached Agent Pane. It is a replay, not a new agent invocation.
#[derive(Clone, Debug, PartialEq)]
pub struct PanelReplay {
    pub session_id: SessionId,
    pub events: Vec<Event>,
}

impl PanelReplay {
    pub fn attach(session_id: SessionId, events: impl IntoIterator<Item = Event>) -> Self {
        Self {
            session_id,
            events: events.into_iter().collect(),
        }
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

pub fn replay_panel(session_id: SessionId, events: impl IntoIterator<Item = Event>) -> PanelReplay {
    PanelReplay::attach(session_id, events)
}

// Keep this import available to the parser helpers without making the public API depend on it.
trait ContextResult<T> {
    fn context(self, message: &'static str) -> Result<T>;
}

impl<T> ContextResult<T> for Option<T> {
    fn context(self, message: &'static str) -> Result<T> {
        self.ok_or_else(|| anyhow::anyhow!(message))
    }
}

trait WithContext<T> {
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> WithContext<T> for std::result::Result<T, E>
where
    E: std::fmt::Display,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|error| anyhow::anyhow!("{}: {error}", f()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bypass_is_the_default_and_gated_is_explicit() {
        assert_eq!(PermissionPosture::default(), PermissionPosture::Bypass);
        assert!(PermissionPosture::Gated.is_gated());
        assert_eq!(
            PermissionPosture::parse("gated").unwrap(),
            PermissionPosture::Gated
        );
    }

    #[test]
    fn only_one_plan_is_active() {
        let first = PlanUpdate::new("one", vec![PlanItem::new("a", "A").unwrap()]).unwrap();
        let second = PlanUpdate::markdown("two", "# Two").unwrap();
        let mut projection = PlanProjection::default();
        assert_eq!(projection.apply(first).plan_id, "one");
        assert_eq!(projection.apply(second).plan_id, "two");
        assert_eq!(projection.active().unwrap().revision, 2);
    }

    #[test]
    fn restricted_schema_validates_defaults_enums_and_unknown_fields() {
        let schema = ElicitationSchema::from_value(&serde_json::json!({
            "type": "object",
            "properties": {
                "language": {"type": "string", "enum": ["rust", "ts"], "default": "rust"},
                "count": {"type": "integer"}
            },
            "required": ["language"]
        }))
        .unwrap();
        let mut values = Map::new();
        values.insert("count".into(), Value::from(2));
        schema
            .validate_values(&schema.with_defaults(&values))
            .unwrap();
        values.insert("extra".into(), Value::Bool(true));
        assert!(schema.validate_values(&values).is_err());
    }

    #[test]
    fn elicitation_records_accept_decline_and_cancel() {
        let schema = ElicitationSchema::from_value(&serde_json::json!({
            "type": "object",
            "properties": {"answer": {"type": "boolean"}},
            "required": ["answer"]
        }))
        .unwrap();
        let request = ElicitationRequest::form("r1", "Continue?", schema).unwrap();
        let mut history = ElicitationHistory::default();
        let mut values = Map::new();
        values.insert("answer".into(), Value::Bool(true));
        assert!(matches!(
            history
                .respond(
                    &request,
                    ElicitationResponse {
                        request_id: "r1".into(),
                        action: ElicitationAction::Accept,
                        values: values.clone()
                    }
                )
                .unwrap(),
            ElicitationResult::Accepted(_)
        ));
        assert!(matches!(
            history
                .respond(
                    &request,
                    ElicitationResponse {
                        request_id: "r1".into(),
                        action: ElicitationAction::Decline,
                        values: Map::new()
                    }
                )
                .unwrap(),
            ElicitationResult::Declined
        ));
        assert!(matches!(
            history
                .respond(
                    &request,
                    ElicitationResponse {
                        request_id: "r1".into(),
                        action: ElicitationAction::Cancel,
                        values: Map::new()
                    }
                )
                .unwrap(),
            ElicitationResult::Cancelled
        ));
        assert_eq!(history.records().len(), 3);
    }

    #[test]
    fn commands_are_session_scoped() {
        assert_eq!(
            SessionCommand::parse("/new-session"),
            Some(SessionCommand::NewSession)
        );
        assert_eq!(
            SessionCommand::parse("/compact"),
            Some(SessionCommand::Compact)
        );
        assert_eq!(
            SessionCommand::parse("/clear-context"),
            Some(SessionCommand::ClearContext)
        );
        assert_eq!(
            SessionCommand::parse("/context"),
            Some(SessionCommand::ViewContext)
        );
        assert_eq!(SessionCommand::parse("ordinary prompt"), None);
    }

    #[test]
    fn stop_cancels_only_active_turn_and_steer_runs_at_boundary() {
        let first = TurnId::generate();
        let mut turns = SteeringBoundary::default();
        assert_eq!(turns.begin_turn(first), None);
        turns.queue_steer("continue with tests").unwrap();
        assert!(turns.stop_active_turn());
        let completion = turns.finish_turn().unwrap();
        assert!(completion.cancelled);
        assert_eq!(
            completion.next_steer.as_deref(),
            Some("continue with tests")
        );
        assert_eq!(turns.queued_count(), 0);
    }
}
