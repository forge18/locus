//! Projections shared by Manage's Kanban, List, Graph, and Timeline views.
//!
//! These helpers accept folds and transition rows, never fixture-only screen state.  Kanban and
//! Timeline intentionally share [`dwell_by_column`] so their durations cannot drift.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::ids::{ProjectId, TaskId};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskColumn {
    Ready,
    InProgress,
    Testing,
    Reviewing,
    PendingApproval,
    Done,
}

impl TaskColumn {
    pub const ALL: [Self; 6] = [
        Self::Ready,
        Self::InProgress,
        Self::Testing,
        Self::Reviewing,
        Self::PendingApproval,
        Self::Done,
    ];
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::InProgress => "in_progress",
            Self::Testing => "testing",
            Self::Reviewing => "reviewing",
            Self::PendingApproval => "pending_approval",
            Self::Done => "done",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskCard {
    pub id: TaskId,
    pub project_id: ProjectId,
    pub title: String,
    pub column: TaskColumn,
    pub blocked: bool,
    pub gate: Option<String>,
    pub workflow: Option<String>,
    pub running_tokens: Option<u64>,
    pub stuck_iterations: u32,
    pub verify_command: Option<String>,
    pub evidence_summary: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KanbanCounts {
    pub ready: usize,
    pub in_progress: usize,
    pub testing: usize,
    pub reviewing: usize,
    pub pending_approval: usize,
    pub done: usize,
}

impl KanbanCounts {
    pub fn for_cards(cards: &[TaskCard], hide_done: bool) -> Self {
        let mut counts = Self::default();
        for card in cards {
            if hide_done && card.column == TaskColumn::Done {
                continue;
            }
            match card.column {
                TaskColumn::Ready => counts.ready += 1,
                TaskColumn::InProgress => counts.in_progress += 1,
                TaskColumn::Testing => counts.testing += 1,
                TaskColumn::Reviewing => counts.reviewing += 1,
                TaskColumn::PendingApproval => counts.pending_approval += 1,
                TaskColumn::Done => counts.done += 1,
            }
        }
        counts
    }

    pub fn get(&self, column: TaskColumn) -> usize {
        match column {
            TaskColumn::Ready => self.ready,
            TaskColumn::InProgress => self.in_progress,
            TaskColumn::Testing => self.testing,
            TaskColumn::Reviewing => self.reviewing,
            TaskColumn::PendingApproval => self.pending_approval,
            TaskColumn::Done => self.done,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CardDecoration {
    pub live_pulse: Option<String>,
    pub blocked_marker: Option<String>,
    pub stuck_ring: Option<String>,
    pub terminal_note: Option<String>,
}

pub fn card_decoration(card: &TaskCard) -> CardDecoration {
    CardDecoration {
        live_pulse: (card.column == TaskColumn::InProgress).then(|| {
            format!(
                "{} · {} tokens",
                card.workflow.as_deref().unwrap_or("workflow"),
                card.running_tokens.unwrap_or(0)
            )
        }),
        blocked_marker: card
            .blocked
            .then(|| card.gate.clone().unwrap_or_else(|| "blocked".into())),
        stuck_ring: (card.stuck_iterations > 0)
            .then(|| format!("stuck {}/3", card.stuck_iterations)),
        terminal_note: match card.column {
            TaskColumn::Testing => card.verify_command.clone(),
            TaskColumn::Reviewing => Some(format!(
                "Gate: {}",
                card.gate.as_deref().unwrap_or("reviewer agent")
            )),
            TaskColumn::Done => card.evidence_summary.clone(),
            _ => None,
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskTransition {
    pub task_id: TaskId,
    pub from: Option<TaskColumn>,
    pub to: TaskColumn,
    pub at_seconds: i64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DwellByColumn {
    pub seconds: BTreeMap<TaskColumn, Vec<u64>>,
}

impl DwellByColumn {
    pub fn median(&self, column: TaskColumn) -> Option<u64> {
        median(self.seconds.get(&column).into_iter().flatten().copied())
    }
    pub fn slowest_two(&self) -> Vec<(TaskColumn, u64)> {
        let mut values = TaskColumn::ALL
            .into_iter()
            .filter_map(|column| self.median(column).map(|seconds| (column, seconds)))
            .collect::<Vec<_>>();
        values.sort_by_key(|(_, seconds)| std::cmp::Reverse(*seconds));
        values.truncate(2);
        values
    }
}

pub fn dwell_by_column(transitions: &[TaskTransition]) -> DwellByColumn {
    let mut per_task = BTreeMap::<TaskId, Vec<&TaskTransition>>::new();
    for transition in transitions {
        per_task
            .entry(transition.task_id)
            .or_default()
            .push(transition);
    }
    let mut dwell = DwellByColumn::default();
    for task_transitions in per_task.values_mut() {
        task_transitions.sort_by_key(|transition| transition.at_seconds);
        for pair in task_transitions.windows(2) {
            if let [current, next] = pair {
                dwell
                    .seconds
                    .entry(current.to)
                    .or_default()
                    .push((next.at_seconds - current.at_seconds).max(0) as u64);
            }
        }
    }
    dwell
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyEdge {
    pub from: TaskId,
    pub to: TaskId,
    pub approval_owed: bool,
}

pub fn graph_edges(
    rows: impl IntoIterator<Item = (TaskId, TaskId)>,
    approval_held: &BTreeSet<TaskId>,
) -> Vec<DependencyEdge> {
    rows.into_iter()
        .map(|(from, to)| DependencyEdge {
            from,
            to,
            approval_owed: approval_held.contains(&from) || approval_held.contains(&to),
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineSegment {
    pub task_id: TaskId,
    pub column: TaskColumn,
    pub start_seconds: i64,
    pub end_seconds: i64,
    pub duration_seconds: u64,
}

pub fn timeline_segments(transitions: &[TaskTransition]) -> Vec<TimelineSegment> {
    let mut per_task = BTreeMap::<TaskId, Vec<&TaskTransition>>::new();
    for transition in transitions {
        per_task
            .entry(transition.task_id)
            .or_default()
            .push(transition);
    }
    let mut segments = Vec::new();
    for (task_id, task_transitions) in per_task {
        let mut ordered = task_transitions;
        ordered.sort_by_key(|transition| transition.at_seconds);
        for pair in ordered.windows(2) {
            if let [current, next] = pair {
                let duration = (next.at_seconds - current.at_seconds).max(0) as u64;
                segments.push(TimelineSegment {
                    task_id,
                    column: current.to,
                    start_seconds: current.at_seconds,
                    end_seconds: next.at_seconds,
                    duration_seconds: duration,
                });
            }
        }
    }
    segments
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionKind {
    Run,
    Wait,
    Idle,
    Bad,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveSession {
    pub id: String,
    pub project_id: ProjectId,
    pub agent: String,
    pub role: String,
    pub iteration: u32,
    pub max_iterations: u32,
    pub tool_errors: u32,
    pub baseline_tool_errors: u32,
    pub tokens: u64,
    pub last_file_write: Option<String>,
    pub kind: SessionKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosedSession {
    pub id: String,
    pub outcome: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionRecord {
    Live(LiveSession),
    Closed(ClosedSession),
}

pub fn closed_session_record(session: &SessionRecord) -> bool {
    matches!(session, SessionRecord::Closed(_))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffCounts {
    pub done: usize,
    pub remaining: usize,
    pub attempted: usize,
    pub open: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StuckBanner {
    pub visible: bool,
    pub handoff: HandoffCounts,
}

pub fn stuck_banner_payload(stuck_iterations: u32, handoff: HandoffCounts) -> StuckBanner {
    StuckBanner {
        visible: stuck_iterations >= 3,
        handoff,
    }
}

pub fn median(values: impl IntoIterator<Item = u64>) -> Option<u64> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    Some(values[(values.len() - 1) / 2])
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod manage {
    use super::*;
    use super::{
        card_decoration as decorate_card, closed_session_record as is_closed,
        dwell_by_column as calculate_dwell, graph_edges as build_edges,
        stuck_banner_payload as make_banner, timeline_segments as build_segments,
    };
    use crate::runtime::dispatch::{
        DispatchPolicy, DispatchPriority, PriorityMethod, QueuedRun, RunState,
    };
    use uuid::Uuid;

    fn task(column: TaskColumn) -> TaskCard {
        TaskCard {
            id: TaskId::new(Uuid::new_v4()),
            project_id: ProjectId::new(Uuid::new_v4()),
            title: "task".into(),
            column,
            blocked: false,
            gate: None,
            workflow: Some("build".into()),
            running_tokens: Some(10),
            stuck_iterations: 0,
            verify_command: Some("cargo test".into()),
            evidence_summary: Some("passed".into()),
        }
    }

    #[test]
    fn kanban_counts() {
        let counts =
            KanbanCounts::for_cards(&[task(TaskColumn::Ready), task(TaskColumn::Done)], false);
        assert_eq!(counts.ready, 1);
        assert_eq!(counts.done, 1);
        assert_eq!(
            KanbanCounts::for_cards(&[task(TaskColumn::Done)], true).done,
            0
        );
    }
    #[test]
    fn card_decoration() {
        let mut card = task(TaskColumn::InProgress);
        card.blocked = true;
        card.gate = Some("approval".into());
        card.stuck_iterations = 3;
        let decoration = decorate_card(&card);
        assert!(decoration.live_pulse.is_some());
        assert_eq!(decoration.blocked_marker.as_deref(), Some("approval"));
        assert_eq!(decoration.stuck_ring.as_deref(), Some("stuck 3/3"));
    }
    #[test]
    fn terminal_column_decoration() {
        let testing = decorate_card(&task(TaskColumn::Testing));
        assert_eq!(testing.terminal_note.as_deref(), Some("cargo test"));
        let reviewing = decorate_card(&task(TaskColumn::Reviewing));
        assert_eq!(
            reviewing.terminal_note.as_deref(),
            Some("Gate: reviewer agent")
        );
    }
    #[test]
    fn dwell_by_column() {
        let id = TaskId::new(Uuid::new_v4());
        let project = ProjectId::new(Uuid::new_v4());
        let rows = [
            TaskTransition {
                task_id: id,
                from: None,
                to: TaskColumn::Ready,
                at_seconds: 0,
            },
            TaskTransition {
                task_id: id,
                from: Some(TaskColumn::Ready),
                to: TaskColumn::Testing,
                at_seconds: 60,
            },
            TaskTransition {
                task_id: id,
                from: Some(TaskColumn::Testing),
                to: TaskColumn::Done,
                at_seconds: 180,
            },
        ];
        let dwell = calculate_dwell(&rows);
        assert_eq!(dwell.median(TaskColumn::Ready), Some(60));
        assert_eq!(dwell.median(TaskColumn::Testing), Some(120));
        let _ = project;
    }
    #[test]
    fn graph_edges() {
        let a = TaskId::new(Uuid::new_v4());
        let b = TaskId::new(Uuid::new_v4());
        let mut held = BTreeSet::new();
        held.insert(b);
        assert!(build_edges([(a, b)], &held)[0].approval_owed);
    }
    fn unblocks_most_order(runs: impl IntoIterator<Item = QueuedRun>) -> Vec<crate::ids::RunId> {
        let policy = DispatchPolicy {
            global_parallelism: u32::MAX,
            per_project_parallelism: u32::MAX,
            priority_method: PriorityMethod::UnblocksMost,
            tie_break: crate::runtime::dispatch::TieBreak::LongestWaiting,
            preemption_enabled: false,
        };
        crate::runtime::dispatch::select_to_start(&policy, runs)
    }

    #[test]
    fn unblocks_most_matches_dispatch() {
        let a = QueuedRun {
            run_id: crate::ids::RunId::generate(),
            project_id: ProjectId::generate(),
            state: RunState::Queued,
            priority: DispatchPriority {
                unblocks_count: 1,
                ..Default::default()
            },
            enqueued_at_ms: 0,
        };
        let b = QueuedRun {
            run_id: crate::ids::RunId::generate(),
            project_id: ProjectId::generate(),
            state: RunState::Queued,
            priority: DispatchPriority {
                unblocks_count: 4,
                ..Default::default()
            },
            enqueued_at_ms: 1,
        };
        assert_eq!(
            unblocks_most_order([a.clone(), b.clone()]),
            vec![b.run_id, a.run_id]
        );
    }
    #[test]
    fn timeline_segments() {
        let id = TaskId::new(Uuid::new_v4());
        let rows = [
            TaskTransition {
                task_id: id,
                from: None,
                to: TaskColumn::Ready,
                at_seconds: 10,
            },
            TaskTransition {
                task_id: id,
                from: Some(TaskColumn::Ready),
                to: TaskColumn::Done,
                at_seconds: 40,
            },
        ];
        let segments = build_segments(&rows);
        assert_eq!(segments[0].duration_seconds, 30);
    }
    #[test]
    fn dwell_and_timeline_agree() {
        let id = TaskId::new(Uuid::new_v4());
        let rows = [
            TaskTransition {
                task_id: id,
                from: None,
                to: TaskColumn::Ready,
                at_seconds: 0,
            },
            TaskTransition {
                task_id: id,
                from: Some(TaskColumn::Ready),
                to: TaskColumn::Done,
                at_seconds: 30,
            },
        ];
        assert_eq!(
            build_segments(&rows)[0].duration_seconds,
            calculate_dwell(&rows).median(TaskColumn::Ready).unwrap()
        );
    }
    #[test]
    fn closed_session_record() {
        let session = SessionRecord::Closed(ClosedSession {
            id: "x".into(),
            outcome: "passed".into(),
        });
        assert!(is_closed(&session));
    }
    #[test]
    fn stuck_banner_payload() {
        let counts = HandoffCounts {
            done: 3,
            remaining: 2,
            attempted: 4,
            open: 1,
        };
        assert_eq!(make_banner(3, counts).handoff.open, 1);
    }
    #[test]
    fn handoff_action() {
        let banner = make_banner(
            3,
            HandoffCounts {
                done: 1,
                remaining: 1,
                attempted: 1,
                open: 1,
            },
        );
        assert!(banner.visible);
    }
    #[test]
    fn let_it_run() {
        assert!(
            !make_banner(
                2,
                HandoffCounts {
                    done: 0,
                    remaining: 0,
                    attempted: 0,
                    open: 0
                }
            )
            .visible
        );
    }
}
