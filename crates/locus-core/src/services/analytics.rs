//! Read-only Analytics and Telemetry projections.
use crate::ids::ProjectId;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalyticsScope {
    All,
    Project(ProjectId),
}
impl AnalyticsScope {
    pub fn includes(self, project: ProjectId) -> bool {
        matches!(self, Self::All) || matches!(self, Self::Project(id) if id == project)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalyticsRange {
    Hours24,
    Days7,
    Days30,
    Days90,
    All,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BucketUnit {
    Hour,
    Day,
    Week,
    Month,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RangeBucket {
    pub start: i64,
    pub end: i64,
    pub unit: BucketUnit,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedRange {
    pub range: AnalyticsRange,
    pub buckets: Vec<RangeBucket>,
}
impl ResolvedRange {
    pub fn new(range: AnalyticsRange, now: i64) -> Self {
        let (count, seconds, unit) = match range {
            AnalyticsRange::Hours24 => (24, 3_600, BucketUnit::Hour),
            AnalyticsRange::Days7 => (7, 86_400, BucketUnit::Day),
            AnalyticsRange::Days30 => (30, 86_400, BucketUnit::Day),
            AnalyticsRange::Days90 => (13, 604_800, BucketUnit::Week),
            AnalyticsRange::All => (12, 2_592_000, BucketUnit::Month),
        };
        let origin = now - i64::from(count) * seconds;
        Self {
            range,
            buckets: (0..count)
                .map(|i| {
                    let start = origin + i64::from(i) * seconds;
                    RangeBucket {
                        start,
                        end: start + seconds,
                        unit,
                    }
                })
                .collect(),
        }
    }
    pub fn contains(&self, timestamp: i64) -> bool {
        self.buckets.first().is_some_and(|b| timestamp >= b.start)
            && self.buckets.last().is_some_and(|b| timestamp < b.end)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalyticsMeasure {
    Spend,
    Tokens,
    CacheRead,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BreakdownDimension {
    Model,
    Harness,
    Agent,
    Role,
    Workflow,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyticsQuery {
    pub scope: AnalyticsScope,
    pub range: ResolvedRange,
    pub search: Option<String>,
    pub facets: BTreeMap<String, String>,
}
impl AnalyticsQuery {
    pub fn new(scope: AnalyticsScope, range: AnalyticsRange, now: i64) -> Self {
        Self {
            scope,
            range: ResolvedRange::new(range, now),
            search: None,
            facets: BTreeMap::new(),
        }
    }
    pub fn includes(&self, project: ProjectId, timestamp: i64) -> bool {
        self.scope.includes(project) && self.range.contains(timestamp)
    }
    pub fn with_search(mut self, search: impl Into<String>) -> Self {
        let s = search.into();
        self.search = (!s.trim().is_empty()).then_some(s);
        self
    }
    pub fn with_facet(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.facets.insert(key.into(), value.into());
        self
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct RunRecord {
    pub project_id: ProjectId,
    pub timestamp: i64,
    pub model: String,
    pub harness: String,
    pub agent: String,
    pub role: String,
    pub workflow: String,
    pub tokens: Option<u64>,
    pub cache_read: Option<u64>,
    pub spend_micros: Option<u64>,
    pub duration_seconds: Option<u64>,
    pub iterations: u32,
    pub verified: Option<bool>,
}
impl RunRecord {
    pub fn visible_to(&self, query: &AnalyticsQuery) -> bool {
        query.includes(self.project_id, self.timestamp)
            && query.facets.iter().all(|(key, value)| match key.as_str() {
                "model" => &self.model == value,
                "harness" => &self.harness == value,
                "agent" => &self.agent == value,
                "role" => &self.role == value,
                "workflow" => &self.workflow == value,
                _ => true,
            })
            && query.search.as_deref().is_none_or(|term| {
                [
                    self.model.as_str(),
                    self.harness.as_str(),
                    self.agent.as_str(),
                    self.role.as_str(),
                    self.workflow.as_str(),
                ]
                .iter()
                .any(|field| {
                    field
                        .to_ascii_lowercase()
                        .contains(&term.to_ascii_lowercase())
                })
            })
    }
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StatCards {
    pub spend_micros: Option<u64>,
    pub tokens: Option<u64>,
    pub cache_read: Option<u64>,
    pub runs: usize,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BreakdownRow {
    pub key: String,
    pub tokens: Option<u64>,
    pub cache_read: Option<u64>,
    pub spend_micros: Option<u64>,
    pub runs: usize,
}
impl BreakdownRow {
    pub fn measure(&self, measure: AnalyticsMeasure) -> Option<u64> {
        match measure {
            AnalyticsMeasure::Spend => self.spend_micros,
            AnalyticsMeasure::Tokens => self.tokens,
            AnalyticsMeasure::CacheRead => self.cache_read,
        }
    }
}
pub fn filtered_runs<'a>(query: &AnalyticsQuery, runs: &'a [RunRecord]) -> Vec<&'a RunRecord> {
    runs.iter().filter(|run| run.visible_to(query)).collect()
}
pub fn stat_cards(query: &AnalyticsQuery, runs: &[RunRecord]) -> StatCards {
    let visible = filtered_runs(query, runs);
    StatCards {
        spend_micros: sum_optional(visible.iter().map(|r| r.spend_micros)),
        tokens: sum_optional(visible.iter().map(|r| r.tokens)),
        cache_read: sum_optional(visible.iter().map(|r| r.cache_read)),
        runs: visible.len(),
    }
}
pub fn breakdown(
    query: &AnalyticsQuery,
    runs: &[RunRecord],
    dimension: BreakdownDimension,
) -> Vec<BreakdownRow> {
    let mut rows = BTreeMap::new();
    for run in filtered_runs(query, runs) {
        let key = match dimension {
            BreakdownDimension::Model => &run.model,
            BreakdownDimension::Harness => &run.harness,
            BreakdownDimension::Agent => &run.agent,
            BreakdownDimension::Role => &run.role,
            BreakdownDimension::Workflow => &run.workflow,
        };
        let row = rows.entry(key.clone()).or_insert_with(|| BreakdownRow {
            key: key.clone(),
            ..Default::default()
        });
        if row.runs == 0 {
            row.tokens = run.tokens;
            row.cache_read = run.cache_read;
            row.spend_micros = run.spend_micros;
        } else {
            row.tokens = add_optional(row.tokens, run.tokens);
            row.cache_read = add_optional(row.cache_read, run.cache_read);
            row.spend_micros = add_optional(row.spend_micros, run.spend_micros);
        }
        row.runs += 1;
    }
    rows.into_values().collect()
}
pub fn range_buckets(range: AnalyticsRange, now: i64) -> Vec<RangeBucket> {
    ResolvedRange::new(range, now).buckets
}
pub fn median_and_p90(values: impl IntoIterator<Item = u64>) -> (Option<u64>, Option<u64>) {
    let mut v = values.into_iter().collect::<Vec<_>>();
    if v.is_empty() {
        return (None, None);
    }
    v.sort_unstable();
    (
        Some(v[(v.len() - 1) / 2]),
        Some(v[((v.len() - 1) * 90) / 100]),
    )
}
pub fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs() as i64
}
fn sum_optional(values: impl Iterator<Item = Option<u64>>) -> Option<u64> {
    let v = values.collect::<Vec<_>>();
    v.iter()
        .all(Option::is_some)
        .then(|| v.into_iter().flatten().sum())
}
fn add_optional(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(a), Some(b)) => Some(a + b),
        (None, Some(b)) => Some(b),
        _ => None,
    }
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrendPoint {
    pub bucket_start: i64,
    pub value: Option<u64>,
    pub runs: usize,
}
pub fn trend(
    query: &AnalyticsQuery,
    runs: &[RunRecord],
    measure: AnalyticsMeasure,
) -> Vec<TrendPoint> {
    query
        .range
        .buckets
        .iter()
        .map(|b| {
            let visible = filtered_runs(query, runs)
                .into_iter()
                .filter(|r| r.timestamp >= b.start && r.timestamp < b.end)
                .collect::<Vec<_>>();
            let value = match measure {
                AnalyticsMeasure::Spend => sum_optional(visible.iter().map(|r| r.spend_micros)),
                AnalyticsMeasure::Tokens => sum_optional(visible.iter().map(|r| r.tokens)),
                AnalyticsMeasure::CacheRead => sum_optional(visible.iter().map(|r| r.cache_read)),
            };
            TrendPoint {
                bucket_start: b.start,
                value,
                runs: visible.len(),
            }
        })
        .collect()
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskAnalyticsRecord {
    pub project_id: ProjectId,
    pub landed: bool,
    pub abandoned: bool,
    pub reworked: bool,
    pub role: String,
    pub cost_micros: Option<u64>,
    pub iterations: u32,
    pub title: String,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TaskOutcomeTotals {
    pub landed: usize,
    pub abandoned: usize,
    pub still_open: usize,
    pub landed_after_rework: usize,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RoleCostRow {
    pub role: String,
    pub landed: usize,
    pub cost_micros: Option<u64>,
    pub runs: usize,
    pub first_try: usize,
}
pub fn project_rework_outcome(project: ProjectId, records: &[TaskAnalyticsRecord]) -> usize {
    records
        .iter()
        .filter(|record| record.project_id == project && record.landed && record.reworked)
        .count()
}

pub fn task_outcomes_and_cost(
    scope: AnalyticsScope,
    records: &[TaskAnalyticsRecord],
) -> (TaskOutcomeTotals, Vec<RoleCostRow>) {
    let mut totals = TaskOutcomeTotals::default();
    let mut roles = BTreeMap::new();
    for r in records.iter().filter(|r| scope.includes(r.project_id)) {
        if r.landed {
            totals.landed += 1;
            if r.reworked {
                totals.landed_after_rework += 1;
            }
        } else if r.abandoned {
            totals.abandoned += 1;
        } else {
            totals.still_open += 1;
        }
        let row = roles.entry(r.role.clone()).or_insert_with(|| RoleCostRow {
            role: r.role.clone(),
            ..Default::default()
        });
        row.runs += 1;
        if r.landed {
            row.landed += 1;
            if !r.reworked {
                row.first_try += 1;
            }
        }
        row.cost_micros = if row.runs == 1 {
            r.cost_micros
        } else {
            add_optional(row.cost_micros, r.cost_micros)
        };
    }
    (totals, roles.into_values().collect())
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowDurationRow {
    pub workflow: String,
    pub runs: usize,
    pub median_seconds: Option<u64>,
    pub p90_seconds: Option<u64>,
    pub iterations: u32,
    pub verified: usize,
}
pub fn workflow_duration_projection(
    query: &AnalyticsQuery,
    runs: &[RunRecord],
) -> Vec<WorkflowDurationRow> {
    let mut grouped: BTreeMap<String, Vec<&RunRecord>> = BTreeMap::new();
    for r in filtered_runs(query, runs) {
        grouped.entry(r.workflow.clone()).or_default().push(r);
    }
    grouped
        .into_iter()
        .map(|(workflow, rs)| {
            let (median_seconds, p90_seconds) =
                median_and_p90(rs.iter().filter_map(|r| r.duration_seconds));
            WorkflowDurationRow {
                workflow,
                runs: rs.len(),
                median_seconds,
                p90_seconds,
                iterations: rs.iter().map(|r| r.iterations).sum(),
                verified: rs.iter().filter(|r| r.verified == Some(true)).count(),
            }
        })
        .collect()
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetrievalTier {
    ShortTerm,
    LongTerm,
    Artifacts,
    Wiki,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetrievalFeedback {
    pub useful: Option<bool>,
    pub changed_answer: Option<bool>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetrievalRecord {
    pub project_id: ProjectId,
    pub timestamp: i64,
    pub tier: RetrievalTier,
    pub tokens: Option<u64>,
    pub feedback: RetrievalFeedback,
    pub fact_written: bool,
    pub promoted: bool,
    pub key: String,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RetrievalProjection {
    pub hits: usize,
    pub useful_percentage: Option<u8>,
    pub average_tokens: Option<u64>,
    pub facts_written: usize,
    pub promoted: usize,
}
pub fn feedback_percentage(values: &[RetrievalFeedback], changed: bool) -> Option<u8> {
    let v = values
        .iter()
        .filter_map(|f| if changed { f.changed_answer } else { f.useful })
        .collect::<Vec<_>>();
    (!v.is_empty()).then(|| (v.iter().filter(|x| **x).count() * 100 / v.len()) as u8)
}
pub fn memory_retrieval_projection(
    query: &AnalyticsQuery,
    records: &[RetrievalRecord],
    tier: RetrievalTier,
) -> RetrievalProjection {
    let selected = records
        .iter()
        .filter(|r| query.includes(r.project_id, r.timestamp) && r.tier == tier)
        .collect::<Vec<_>>();
    let tokens = selected.iter().filter_map(|r| r.tokens).collect::<Vec<_>>();
    RetrievalProjection {
        hits: selected.len(),
        useful_percentage: feedback_percentage(
            &selected
                .iter()
                .map(|r| r.feedback.clone())
                .collect::<Vec<_>>(),
            false,
        ),
        average_tokens: (!tokens.is_empty())
            .then(|| tokens.iter().sum::<u64>() / tokens.len() as u64),
        facts_written: selected.iter().filter(|r| r.fact_written).count(),
        promoted: selected.iter().filter(|r| r.promoted).count(),
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtensionKind {
    Skill,
    Rule,
    Hook,
    Linter,
    Style,
    Agent,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionUsageRecord {
    pub project_id: ProjectId,
    pub timestamp: i64,
    pub kind: ExtensionKind,
    pub name: String,
    pub materialized: bool,
    pub invoked: bool,
    pub failed: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionUsageRow {
    pub name: String,
    pub hits: usize,
    pub note: String,
}
pub fn extension_usage_projection(
    query: &AnalyticsQuery,
    records: &[ExtensionUsageRecord],
    kind: Option<ExtensionKind>,
) -> Vec<ExtensionUsageRow> {
    let mut rows = BTreeMap::new();
    for r in records.iter().filter(|r| {
        query.includes(r.project_id, r.timestamp)
            && kind.is_none_or(|k| k == r.kind)
            && (r.materialized || r.invoked)
    }) {
        let row = rows
            .entry(r.name.clone())
            .or_insert_with(|| ExtensionUsageRow {
                name: r.name.clone(),
                hits: 0,
                note: String::new(),
            });
        row.hits += 1;
        if r.failed {
            row.note = "1 failing".into();
        }
    }
    rows.into_values().collect()
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelemetryRecord {
    pub project_id: ProjectId,
    pub timestamp: i64,
    pub run_id: String,
    pub verb: TelemetryVerb,
    pub text: String,
    pub tool: Option<String>,
    pub output_tokens: Option<u64>,
    pub harness: String,
    pub agent: String,
    pub role: String,
    pub model_tier: String,
    pub verify: String,
    pub arbiter_class: Option<String>,
    pub branch: String,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryVerb {
    ToolCall,
    ToolResult,
    Assistant,
    Thinking,
    User,
    ToolError,
    SubagentStart,
    SubagentStop,
    SessionStart,
    SessionEnd,
    Aborted,
    PermissionRequest,
}
impl TelemetryVerb {
    pub const ALL: [Self; 12] = [
        Self::ToolCall,
        Self::ToolResult,
        Self::Assistant,
        Self::Thinking,
        Self::User,
        Self::ToolError,
        Self::SubagentStart,
        Self::SubagentStop,
        Self::SessionStart,
        Self::SessionEnd,
        Self::Aborted,
        Self::PermissionRequest,
    ];
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ToolCall => "tool_call",
            Self::ToolResult => "tool_result",
            Self::Assistant => "assistant",
            Self::Thinking => "thinking",
            Self::User => "user",
            Self::ToolError => "tool_error",
            Self::SubagentStart => "subagent_start",
            Self::SubagentStop => "subagent_stop",
            Self::SessionStart => "session_start",
            Self::SessionEnd => "session_end",
            Self::Aborted => "aborted",
            Self::PermissionRequest => "permission_request",
        }
    }
}
impl TelemetryRecord {
    pub fn searchable_text(&self) -> String {
        [
            self.text.as_str(),
            self.tool.as_deref().unwrap_or(""),
            self.verb.as_str(),
            self.branch.as_str(),
        ]
        .join(" ")
    }
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TelemetryFilters {
    pub scope: Option<AnalyticsScope>,
    pub range: Option<ResolvedRange>,
    pub search: Option<String>,
    pub facets: BTreeMap<String, String>,
}
impl TelemetryFilters {
    pub fn matches(&self, e: &TelemetryRecord) -> bool {
        self.scope.is_none_or(|s| s.includes(e.project_id))
            && self.range.as_ref().is_none_or(|r| r.contains(e.timestamp))
            && self.search.as_deref().is_none_or(|s| {
                e.searchable_text()
                    .to_ascii_lowercase()
                    .contains(&s.to_ascii_lowercase())
            })
            && self.facets.iter().all(|(k, v)| match k.as_str() {
                "harness" => &e.harness == v,
                "project" => e.project_id.to_string() == *v,
                "agent" => &e.agent == v,
                "role" => &e.role == v,
                "model_tier" => &e.model_tier == v,
                "verify" => &e.verify == v,
                "arbiter_class" => e.arbiter_class.as_deref() == Some(v),
                "branch" => &e.branch == v,
                _ => true,
            })
    }
}
pub fn telemetry_intersection<'a>(
    filters: &TelemetryFilters,
    events: &'a [TelemetryRecord],
) -> Vec<&'a TelemetryRecord> {
    events.iter().filter(|e| filters.matches(e)).collect()
}
pub fn facet_counts(events: &[&TelemetryRecord], facet: &str) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for e in events {
        let value = match facet {
            "harness" => Some(e.harness.clone()),
            "project" => Some(e.project_id.to_string()),
            "agent" => Some(e.agent.clone()),
            "role" => Some(e.role.clone()),
            "model_tier" => Some(e.model_tier.clone()),
            "verify" => Some(e.verify.clone()),
            "arbiter_class" => e.arbiter_class.clone(),
            "branch" => Some(e.branch.clone()),
            _ => None,
        };
        if let Some(v) = value {
            *out.entry(v).or_default() += 1;
        }
    }
    out
}
pub fn action_counts(events: &[&TelemetryRecord]) -> BTreeMap<&'static str, usize> {
    let mut out = BTreeMap::new();
    for e in events {
        *out.entry(e.verb.as_str()).or_default() += 1;
    }
    out
}
pub fn permission_request_is_alarm(events: &[&TelemetryRecord]) -> bool {
    events
        .iter()
        .any(|e| e.verb == TelemetryVerb::PermissionRequest)
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolPayloadRow {
    pub tool: String,
    pub calls: usize,
    pub payload_tokens: Option<u64>,
    pub anomaly: Option<String>,
}
pub fn tool_payload_projection(
    events: &[TelemetryRecord],
    allowlisted: &BTreeSet<String>,
) -> Vec<ToolPayloadRow> {
    let mut out = BTreeMap::new();
    for e in events.iter().filter(|e| {
        e.verb == TelemetryVerb::ToolResult
            && e.tool.as_ref().is_some_and(|t| allowlisted.contains(t))
    }) {
        let t = e.tool.clone().unwrap();
        let r = out.entry(t.clone()).or_insert_with(|| ToolPayloadRow {
            tool: t,
            calls: 0,
            payload_tokens: Some(0),
            anomaly: None,
        });
        r.calls += 1;
        r.payload_tokens = add_optional(r.payload_tokens, e.output_tokens);
    }
    out.into_values().collect()
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionTelemetryStatus {
    Running,
    Stuck,
    WaitingGate,
    Idle,
    HandedOff,
    Closed,
    Aborted,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionTelemetryRow {
    pub run_id: String,
    pub status: SessionTelemetryStatus,
    pub events: usize,
    pub errors: usize,
    pub tokens: Option<u64>,
}
pub fn session_table_projection(events: &[TelemetryRecord]) -> Vec<SessionTelemetryRow> {
    let mut out = BTreeMap::new();
    for e in events {
        let r = out
            .entry(e.run_id.clone())
            .or_insert_with(|| SessionTelemetryRow {
                run_id: e.run_id.clone(),
                status: SessionTelemetryStatus::Running,
                events: 0,
                errors: 0,
                tokens: Some(0),
            });
        r.events += 1;
        if e.verb == TelemetryVerb::ToolError {
            r.errors += 1;
        }
        r.tokens = add_optional(r.tokens, e.output_tokens);
        if e.verb == TelemetryVerb::Aborted {
            r.status = SessionTelemetryStatus::Aborted;
        }
    }
    out.into_values().collect()
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TelemetryStatProjection {
    pub sessions: usize,
    pub events: usize,
    pub tool_errors: usize,
    pub output_tokens: u64,
    pub permission_requests: usize,
}

pub fn telemetry_stat_projection(events: &[TelemetryRecord]) -> TelemetryStatProjection {
    let sessions = events
        .iter()
        .map(|event| event.run_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    TelemetryStatProjection {
        sessions,
        events: events.len(),
        tool_errors: events
            .iter()
            .filter(|event| event.verb == TelemetryVerb::ToolError)
            .count(),
        output_tokens: events.iter().filter_map(|event| event.output_tokens).sum(),
        permission_requests: events
            .iter()
            .filter(|event| event.verb == TelemetryVerb::PermissionRequest)
            .count(),
    }
}

pub fn action_vocabulary_projection(events: &[&TelemetryRecord]) -> BTreeMap<&'static str, usize> {
    action_counts(events)
}

pub fn telemetry_facets() -> [&'static str; 6] {
    [
        "harness",
        "agent",
        "role",
        "model_tier",
        "verify",
        "arbiter_class",
    ]
}

pub fn reset_filters(filters: &mut TelemetryFilters) {
    *filters = TelemetryFilters::default();
}

#[cfg(test)]
mod analytics_tests {
    use super::*;
    use super::{
        breakdown as project_breakdown, facet_counts as count_facets,
        permission_request_is_alarm as alarm, range_buckets as buckets, stat_cards as cards,
    };
    #[test]
    fn scope_applies_to_all_projections() {
        let p = ProjectId::generate();
        let q = AnalyticsQuery::new(AnalyticsScope::Project(p), AnalyticsRange::All, 10_000);
        let r = RunRecord {
            project_id: p,
            timestamp: 9_000,
            model: "m".into(),
            harness: "h".into(),
            agent: "a".into(),
            role: "r".into(),
            workflow: "w".into(),
            tokens: Some(2),
            cache_read: Some(1),
            spend_micros: Some(3),
            duration_seconds: Some(4),
            iterations: 1,
            verified: Some(true),
        };
        assert_eq!(cards(&q, &[r]).runs, 1);
    }
    #[test]
    fn range_buckets() {
        assert_eq!(buckets(AnalyticsRange::Hours24, 100).len(), 24);
        assert_eq!(buckets(AnalyticsRange::Days7, 100).len(), 7);
        assert_eq!(buckets(AnalyticsRange::Days30, 100).len(), 30);
        assert_eq!(buckets(AnalyticsRange::Days90, 100).len(), 13);
        assert_eq!(buckets(AnalyticsRange::All, 100).len(), 12);
    }
    #[test]
    fn range_and_scope_are_shared() {
        let q = AnalyticsQuery::new(AnalyticsScope::All, AnalyticsRange::Days7, 1_000_000);
        assert!(q.includes(ProjectId::generate(), 900_000));
        assert!(!q.includes(ProjectId::generate(), 100));
    }
    #[test]
    fn stat_cards_and_selected_measure() {
        let p = ProjectId::generate();
        let r = RunRecord {
            project_id: p,
            timestamp: 9_000,
            model: "m".into(),
            harness: "h".into(),
            agent: "a".into(),
            role: "r".into(),
            workflow: "w".into(),
            tokens: Some(20),
            cache_read: Some(10),
            spend_micros: Some(5),
            duration_seconds: None,
            iterations: 1,
            verified: None,
        };
        let q = AnalyticsQuery::new(AnalyticsScope::All, AnalyticsRange::All, 10_000);
        assert_eq!(
            project_breakdown(&q, &[r], BreakdownDimension::Model)[0]
                .measure(AnalyticsMeasure::Tokens),
            Some(20)
        );
    }
    #[test]
    fn telemetry_query_intersection() {
        let p = ProjectId::generate();
        let e = TelemetryRecord {
            project_id: p,
            timestamp: 9_000,
            run_id: "r".into(),
            verb: TelemetryVerb::ToolCall,
            text: "search".into(),
            tool: Some("rg".into()),
            output_tokens: Some(1),
            harness: "h".into(),
            agent: "a".into(),
            role: "r".into(),
            model_tier: "high".into(),
            verify: "passed".into(),
            arbiter_class: None,
            branch: "agent/r".into(),
        };
        let f = TelemetryFilters {
            scope: Some(AnalyticsScope::Project(p)),
            range: Some(ResolvedRange::new(AnalyticsRange::All, 10_000)),
            search: Some("rg".into()),
            ..Default::default()
        };
        assert_eq!(telemetry_intersection(&f, &[e]).len(), 1);
    }
    #[test]
    fn facet_counts_follow_result_set() {
        let p = ProjectId::generate();
        let e = TelemetryRecord {
            project_id: p,
            timestamp: 1,
            run_id: "r".into(),
            verb: TelemetryVerb::ToolCall,
            text: "".into(),
            tool: None,
            output_tokens: None,
            harness: "h".into(),
            agent: "a".into(),
            role: "r".into(),
            model_tier: "low".into(),
            verify: "n/a".into(),
            arbiter_class: None,
            branch: "agent/r".into(),
        };
        assert_eq!(count_facets(&[&e], "harness").get("h"), Some(&1));
    }
    #[test]
    fn permission_request_is_alarm() {
        let e = TelemetryRecord {
            project_id: ProjectId::generate(),
            timestamp: 1,
            run_id: "r".into(),
            verb: TelemetryVerb::PermissionRequest,
            text: "".into(),
            tool: None,
            output_tokens: None,
            harness: "h".into(),
            agent: "a".into(),
            role: "r".into(),
            model_tier: "low".into(),
            verify: "n/a".into(),
            arbiter_class: None,
            branch: "agent/r".into(),
        };
        assert!(alarm(&[&e]));
    }
    #[test]
    fn retrieval_feedback() {
        assert_eq!(
            feedback_percentage(
                &[
                    RetrievalFeedback {
                        useful: Some(true),
                        changed_answer: None
                    },
                    RetrievalFeedback {
                        useful: Some(false),
                        changed_answer: None
                    }
                ],
                false
            ),
            Some(50)
        );
    }

    #[test]
    fn trend_tracks_selected_measure() {
        let project = ProjectId::generate();
        let run = RunRecord {
            project_id: project,
            timestamp: 9_000,
            model: "m".into(),
            harness: "h".into(),
            agent: "a".into(),
            role: "r".into(),
            workflow: "w".into(),
            tokens: Some(20),
            cache_read: Some(3),
            spend_micros: Some(7),
            duration_seconds: None,
            iterations: 1,
            verified: None,
        };
        let query = AnalyticsQuery::new(AnalyticsScope::All, AnalyticsRange::All, 10_000);
        assert!(
            trend(&query, std::slice::from_ref(&run), AnalyticsMeasure::Tokens)
                .iter()
                .any(|point| point.value == Some(20))
        );
        assert!(
            trend(&query, std::slice::from_ref(&run), AnalyticsMeasure::Spend)
                .iter()
                .any(|point| point.value == Some(7))
        );
    }

    #[test]
    fn breakdown_dimensions() {
        let project = ProjectId::generate();
        let run = RunRecord {
            project_id: project,
            timestamp: 9_000,
            model: "m".into(),
            harness: "h".into(),
            agent: "a".into(),
            role: "r".into(),
            workflow: "w".into(),
            tokens: Some(1),
            cache_read: None,
            spend_micros: None,
            duration_seconds: None,
            iterations: 1,
            verified: None,
        };
        let query = AnalyticsQuery::new(AnalyticsScope::All, AnalyticsRange::All, 10_000);
        for dimension in [
            BreakdownDimension::Model,
            BreakdownDimension::Harness,
            BreakdownDimension::Agent,
            BreakdownDimension::Role,
            BreakdownDimension::Workflow,
        ] {
            assert_eq!(
                breakdown(&query, std::slice::from_ref(&run), dimension).len(),
                1
            );
        }
    }

    #[test]
    fn task_outcomes_and_cost() {
        let project = ProjectId::generate();
        let records = [TaskAnalyticsRecord {
            project_id: project,
            landed: true,
            abandoned: false,
            reworked: true,
            role: "builder".into(),
            cost_micros: Some(10),
            iterations: 2,
            title: "task".into(),
        }];
        let (totals, roles) = super::task_outcomes_and_cost(AnalyticsScope::All, &records);
        assert_eq!(totals.landed_after_rework, 1);
        assert_eq!(roles[0].cost_micros, Some(10));
    }

    #[test]
    fn project_rework_outcome() {
        let project = ProjectId::generate();
        let record = TaskAnalyticsRecord {
            project_id: project,
            landed: true,
            abandoned: false,
            reworked: true,
            role: "builder".into(),
            cost_micros: None,
            iterations: 2,
            title: "task".into(),
        };
        assert_eq!(super::project_rework_outcome(project, &[record]), 1);
    }

    #[test]
    fn workflow_duration_projection() {
        let project = ProjectId::generate();
        let query = AnalyticsQuery::new(AnalyticsScope::All, AnalyticsRange::All, 10_000);
        let run = RunRecord {
            project_id: project,
            timestamp: 9_000,
            model: "m".into(),
            harness: "h".into(),
            agent: "a".into(),
            role: "r".into(),
            workflow: "release".into(),
            tokens: None,
            cache_read: None,
            spend_micros: None,
            duration_seconds: Some(10),
            iterations: 2,
            verified: Some(true),
        };
        assert_eq!(
            super::workflow_duration_projection(&query, &[run])[0].verified,
            1
        );
    }

    #[test]
    fn memory_retrieval_projection() {
        let project = ProjectId::generate();
        let query = AnalyticsQuery::new(AnalyticsScope::All, AnalyticsRange::All, 10_000);
        let record = RetrievalRecord {
            project_id: project,
            timestamp: 9_000,
            tier: RetrievalTier::LongTerm,
            tokens: Some(12),
            feedback: RetrievalFeedback {
                useful: Some(true),
                changed_answer: None,
            },
            fact_written: true,
            promoted: true,
            key: "f".into(),
        };
        let projection =
            super::memory_retrieval_projection(&query, &[record], RetrievalTier::LongTerm);
        assert_eq!(projection.hits, 1);
        assert_eq!(projection.useful_percentage, Some(100));
    }

    #[test]
    fn extension_usage_projection() {
        let project = ProjectId::generate();
        let query = AnalyticsQuery::new(AnalyticsScope::All, AnalyticsRange::All, 10_000);
        let record = ExtensionUsageRecord {
            project_id: project,
            timestamp: 9_000,
            kind: ExtensionKind::Skill,
            name: "review".into(),
            materialized: false,
            invoked: true,
            failed: false,
        };
        assert_eq!(
            super::extension_usage_projection(&query, &[record], Some(ExtensionKind::Skill))[0]
                .hits,
            1
        );
    }

    #[test]
    fn telemetry_facets_are_acp_only() {
        assert_eq!(
            super::telemetry_facets(),
            [
                "harness",
                "agent",
                "role",
                "model_tier",
                "verify",
                "arbiter_class"
            ]
        );
        assert_eq!(TelemetryVerb::ALL.len(), 12);
    }

    #[test]
    fn telemetry_stat_projection() {
        let event = TelemetryRecord {
            project_id: ProjectId::generate(),
            timestamp: 1,
            run_id: "r".into(),
            verb: TelemetryVerb::ToolError,
            text: "error".into(),
            tool: None,
            output_tokens: Some(4),
            harness: "h".into(),
            agent: "a".into(),
            role: "r".into(),
            model_tier: "low".into(),
            verify: "n/a".into(),
            arbiter_class: None,
            branch: "b".into(),
        };
        let stats = super::telemetry_stat_projection(&[event]);
        assert_eq!(
            (
                stats.sessions,
                stats.events,
                stats.tool_errors,
                stats.output_tokens
            ),
            (1, 1, 1, 4)
        );
    }

    #[test]
    fn action_vocabulary_projection() {
        let event = TelemetryRecord {
            project_id: ProjectId::generate(),
            timestamp: 1,
            run_id: "r".into(),
            verb: TelemetryVerb::ToolCall,
            text: "".into(),
            tool: None,
            output_tokens: None,
            harness: "h".into(),
            agent: "a".into(),
            role: "r".into(),
            model_tier: "low".into(),
            verify: "n/a".into(),
            arbiter_class: None,
            branch: "b".into(),
        };
        assert_eq!(
            super::action_vocabulary_projection(&[&event]).get("tool_call"),
            Some(&1)
        );
    }

    #[test]
    fn tool_projection() {
        let event = TelemetryRecord {
            project_id: ProjectId::generate(),
            timestamp: 1,
            run_id: "r".into(),
            verb: TelemetryVerb::ToolResult,
            text: "".into(),
            tool: Some("rg".into()),
            output_tokens: Some(8),
            harness: "h".into(),
            agent: "a".into(),
            role: "r".into(),
            model_tier: "low".into(),
            verify: "n/a".into(),
            arbiter_class: None,
            branch: "b".into(),
        };
        let allowlisted = BTreeSet::from(["rg".into()]);
        assert_eq!(
            super::tool_payload_projection(&[event], &allowlisted)[0].payload_tokens,
            Some(8)
        );
    }

    #[test]
    fn session_table_projection() {
        let event = TelemetryRecord {
            project_id: ProjectId::generate(),
            timestamp: 1,
            run_id: "r".into(),
            verb: TelemetryVerb::Aborted,
            text: "".into(),
            tool: None,
            output_tokens: Some(8),
            harness: "h".into(),
            agent: "a".into(),
            role: "r".into(),
            model_tier: "low".into(),
            verify: "aborted".into(),
            arbiter_class: None,
            branch: "b".into(),
        };
        let rows = super::session_table_projection(&[event]);
        assert_eq!(rows[0].status, SessionTelemetryStatus::Aborted);
        assert_eq!(rows[0].tokens, Some(8));
    }
}
