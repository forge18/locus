//! Read-only dashboard metric projections.
//!
//! Metrics consume already-recorded run, workflow, board, and telemetry rows. This
//! module intentionally has no Store dependency and no write path.

use std::collections::BTreeMap;

use crate::{
    ids::{ProjectId, RunId, TaskId},
    runtime::dispatch::NetworkTier,
    services::{arbiter::FailureClass, manage::TaskColumn},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageMetric {
    pub input_tokens: u64,
    pub cache_read_tokens: u64,
    pub output_tokens: u64,
    pub spend_micros: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolResultMetric {
    pub tool: String,
    pub payload_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GateOutcome {
    pub found_issue: bool,
    pub correct_issue: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetricRun {
    pub run_id: RunId,
    pub project_id: ProjectId,
    pub timestamp: i64,
    pub agent: String,
    pub harness: String,
    pub usage: Option<UsageMetric>,
    pub verify_passed: Option<bool>,
    pub guardrail_trips: Vec<String>,
    pub rejected_artifacts: u32,
    pub task_id: Option<TaskId>,
    pub iterations: u32,
    pub arbiter_classes: Vec<FailureClass>,
    pub gate_outcomes: Vec<GateOutcome>,
    pub tool_results: Vec<ToolResultMetric>,
    pub prefix_id: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MetricSlice {
    pub agent: Option<String>,
    pub project_id: Option<ProjectId>,
    pub harness: Option<String>,
}

impl MetricSlice {
    fn includes(&self, run: &MetricRun) -> bool {
        self.agent.as_deref().is_none_or(|agent| agent == run.agent)
            && self
                .project_id
                .is_none_or(|project| project == run.project_id)
            && self
                .harness
                .as_deref()
                .is_none_or(|harness| harness == run.harness)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RunsAndSpend {
    pub runs: usize,
    pub spend_micros: Option<u64>,
}

pub fn runs_and_spend(runs: &[MetricRun]) -> RunsAndSpend {
    RunsAndSpend {
        runs: runs.len(),
        spend_micros: sum_usage(runs, |usage| usage.spend_micros),
    }
}

fn sum_usage(runs: &[MetricRun], value: impl Fn(&UsageMetric) -> u64) -> Option<u64> {
    runs.iter()
        .map(|run| run.usage.as_ref().map(&value))
        .collect::<Option<Vec<_>>>()
        .map(|values| values.into_iter().sum())
}

/// `None` is an unknown cache rate, never an invented 0%, when any run has no usage.
pub fn cache_rate(runs: &[MetricRun]) -> Option<f64> {
    if runs.is_empty() {
        return None;
    }
    let usage = runs
        .iter()
        .map(|run| run.usage.as_ref())
        .collect::<Option<Vec<_>>>()?;
    let input: u64 = usage.iter().map(|usage| usage.input_tokens).sum();
    if input == 0 {
        return None;
    }
    Some(
        usage
            .iter()
            .map(|usage| usage.cache_read_tokens)
            .sum::<u64>() as f64
            * 100.0
            / input as f64,
    )
}

pub fn cache_alert(rate: Option<f64>) -> bool {
    rate.is_some_and(|rate| rate < 80.0)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OffenderRow {
    pub tool: String,
    pub agent: String,
    pub project_id: ProjectId,
    pub harness: String,
    pub payload_bytes: u64,
}

pub fn offender_ranking(runs: &[MetricRun], slice: &MetricSlice) -> Vec<OffenderRow> {
    let mut rows = BTreeMap::<(String, String, ProjectId, String), u64>::new();
    for run in runs.iter().filter(|run| slice.includes(run)) {
        for result in &run.tool_results {
            *rows
                .entry((
                    result.tool.clone(),
                    run.agent.clone(),
                    run.project_id,
                    run.harness.clone(),
                ))
                .or_default() += result.payload_bytes;
        }
    }
    let mut ranking = rows
        .into_iter()
        .map(
            |((tool, agent, project_id, harness), payload_bytes)| OffenderRow {
                tool,
                agent,
                project_id,
                harness,
                payload_bytes,
            },
        )
        .collect::<Vec<_>>();
    ranking.sort_by(|left, right| {
        right
            .payload_bytes
            .cmp(&left.payload_bytes)
            .then_with(|| left.tool.cmp(&right.tool))
    });
    ranking
}

pub fn verify_pass_rate(runs: &[MetricRun]) -> Option<f64> {
    let verified = runs
        .iter()
        .filter_map(|run| run.verify_passed)
        .collect::<Vec<_>>();
    (!verified.is_empty()).then(|| {
        verified.iter().filter(|passed| **passed).count() as f64 * 100.0 / verified.len() as f64
    })
}

pub fn guardrail_trips(runs: &[MetricRun]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for trip in runs.iter().flat_map(|run| &run.guardrail_trips) {
        *counts.entry(trip.clone()).or_default() += 1;
    }
    counts
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoardTransitionMetric {
    pub task_id: TaskId,
    pub from: Option<TaskColumn>,
    pub to: TaskColumn,
}

pub fn board_throughput(transitions: &[BoardTransitionMetric]) -> usize {
    transitions
        .iter()
        .filter(|transition| transition.to == TaskColumn::Done)
        .count()
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ArbiterRates {
    pub spec_gap_percent: f64,
    pub ambiguity_percent: f64,
}

pub fn arbiter_rates(classes: &[FailureClass]) -> ArbiterRates {
    if classes.is_empty() {
        return ArbiterRates::default();
    }
    ArbiterRates {
        spec_gap_percent: percentage(classes, FailureClass::SpecGap),
        ambiguity_percent: percentage(classes, FailureClass::Ambiguity),
    }
}

fn percentage(classes: &[FailureClass], target: FailureClass) -> f64 {
    classes.iter().filter(|class| **class == target).count() as f64 * 100.0 / classes.len() as f64
}

pub fn iterations_per_task(runs: &[MetricRun]) -> Option<f64> {
    let mut grouped = BTreeMap::<TaskId, Vec<u32>>::new();
    for run in runs {
        if let Some(task_id) = run.task_id {
            grouped.entry(task_id).or_default().push(run.iterations);
        }
    }
    (!grouped.is_empty()).then(|| {
        grouped
            .values()
            .map(|values| values.iter().sum::<u32>() as f64 / values.len() as f64)
            .sum::<f64>()
            / grouped.len() as f64
    })
}

pub fn gate_precision(outcomes: &[GateOutcome]) -> Option<f64> {
    let reviewed = outcomes
        .iter()
        .filter(|outcome| outcome.found_issue)
        .count();
    (reviewed > 0).then(|| {
        outcomes
            .iter()
            .filter(|outcome| outcome.found_issue && outcome.correct_issue)
            .count() as f64
            * 100.0
            / reviewed as f64
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrustDiscounts {
    pub base: f64,
    pub guardrails: f64,
    pub rejected_artifacts: f64,
    pub tokens: f64,
    pub score: f64,
    pub tokens_per_passing_run: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentTrust {
    pub agent: String,
    pub runs: usize,
    pub score: f64,
    pub discounts: TrustDiscounts,
}

pub fn agent_trust(runs: &[MetricRun]) -> Vec<AgentTrust> {
    let mut recent = runs.to_vec();
    recent.sort_by_key(|run| std::cmp::Reverse(run.timestamp));
    recent.truncate(20);
    let mut grouped = BTreeMap::<String, Vec<MetricRun>>::new();
    for run in recent {
        grouped.entry(run.agent.clone()).or_default().push(run);
    }
    grouped
        .into_iter()
        .map(|(agent, runs)| AgentTrust {
            agent,
            runs: runs.len(),
            score: trust_discounts(&runs).score,
            discounts: trust_discounts(&runs),
        })
        .collect()
}

pub fn trust_discounts(runs: &[MetricRun]) -> TrustDiscounts {
    let verified = runs
        .iter()
        .filter_map(|run| run.verify_passed)
        .collect::<Vec<_>>();
    let base = if verified.is_empty() {
        0.0
    } else {
        verified.iter().filter(|passed| **passed).count() as f64 / verified.len() as f64
    };
    let guardrail_count = runs
        .iter()
        .map(|run| run.guardrail_trips.len() as u32)
        .sum::<u32>();
    let rejected_count = runs.iter().map(|run| run.rejected_artifacts).sum::<u32>();
    let guardrails = 0.9_f64.powi(guardrail_count as i32);
    let rejected_artifacts = 0.9_f64.powi(rejected_count as i32);
    let passing = runs
        .iter()
        .filter(|run| run.verify_passed == Some(true))
        .collect::<Vec<_>>();
    let tokens_per_passing_run = (!passing.is_empty()).then(|| {
        passing
            .iter()
            .filter_map(|run| {
                run.usage
                    .as_ref()
                    .map(|usage| usage.input_tokens + usage.output_tokens)
            })
            .sum::<u64>() as f64
            / passing.len() as f64
    });
    let tokens = tokens_per_passing_run.map_or(1.0, |tokens| 1.0 / (1.0 + tokens / 100_000.0));
    TrustDiscounts {
        base,
        guardrails,
        rejected_artifacts,
        tokens,
        score: base * guardrails * rejected_artifacts * tokens,
        tokens_per_passing_run,
    }
}

pub fn trust_by_tokens(runs: &[MetricRun]) -> Option<f64> {
    trust_discounts(runs).tokens_per_passing_run
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrefixDrift {
    pub run_id: RunId,
    pub prefix_id: String,
    pub cache_rate: f64,
}

pub fn detects_prefix_drift(runs: &[MetricRun]) -> Option<PrefixDrift> {
    runs.iter()
        .filter_map(|run| {
            let usage = run.usage.as_ref()?;
            if usage.input_tokens == 0 {
                return None;
            }
            let rate = usage.cache_read_tokens as f64 * 100.0 / usage.input_tokens as f64;
            (rate < 80.0).then(|| PrefixDrift {
                run_id: run.run_id,
                prefix_id: run.prefix_id.clone().unwrap_or_else(|| "unknown".into()),
                cache_rate: rate,
            })
        })
        .min_by(|left, right| left.cache_rate.total_cmp(&right.cache_rate))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetricNetworkPolicy {
    pub tier: NetworkTier,
}

#[cfg(test)]
mod metrics {
    use super::*;

    fn run(agent: &str, timestamp: i64, cache_read: Option<u64>) -> MetricRun {
        MetricRun {
            run_id: RunId::generate(),
            project_id: ProjectId::generate(),
            timestamp,
            agent: agent.into(),
            harness: "claude".into(),
            usage: cache_read.map(|cache_read| UsageMetric {
                input_tokens: 100,
                cache_read_tokens: cache_read,
                output_tokens: 20,
                spend_micros: 10,
            }),
            verify_passed: Some(true),
            guardrail_trips: vec![],
            rejected_artifacts: 0,
            task_id: Some(TaskId::generate()),
            iterations: 2,
            arbiter_classes: vec![],
            gate_outcomes: vec![],
            tool_results: vec![],
            prefix_id: Some("prefix-a".into()),
        }
    }

    #[test]
    fn runs_and_spend() {
        assert_eq!(
            super::runs_and_spend(&[run("builder", 1, Some(80))]).spend_micros,
            Some(10)
        );
    }

    #[test]
    fn cache_rate() {
        assert_eq!(
            super::cache_rate(&[run("builder", 1, Some(80))]),
            Some(80.0)
        );
    }

    #[test]
    fn cache_unknown_not_zero() {
        assert_eq!(super::cache_rate(&[run("builder", 1, None)]), None);
    }

    #[test]
    fn offender_ranking() {
        let mut first = run("builder", 1, Some(80));
        first.tool_results.push(ToolResultMetric {
            tool: "read".into(),
            payload_bytes: 20,
        });
        let mut second = run("builder", 2, Some(80));
        second.project_id = first.project_id;
        second.tool_results.push(ToolResultMetric {
            tool: "read".into(),
            payload_bytes: 30,
        });
        assert_eq!(
            super::offender_ranking(&[first, second], &MetricSlice::default())[0].payload_bytes,
            50
        );
    }

    #[test]
    fn ranking_slices() {
        let mut selected = run("builder", 1, Some(80));
        selected.tool_results.push(ToolResultMetric {
            tool: "read".into(),
            payload_bytes: 20,
        });
        let mut other = run("reviewer", 2, Some(80));
        other.tool_results.push(ToolResultMetric {
            tool: "read".into(),
            payload_bytes: 30,
        });
        assert_eq!(
            super::offender_ranking(
                &[selected.clone(), other],
                &MetricSlice {
                    agent: Some("builder".into()),
                    ..Default::default()
                }
            )
            .len(),
            1
        );
    }

    #[test]
    fn verify_pass_rate() {
        let mut failed = run("builder", 1, Some(80));
        failed.verify_passed = Some(false);
        assert_eq!(
            super::verify_pass_rate(&[run("builder", 1, Some(80)), failed]),
            Some(50.0)
        );
    }

    #[test]
    fn guardrail_trips() {
        let mut run = run("builder", 1, Some(80));
        run.guardrail_trips = vec!["idle".into(), "idle".into(), "budget".into()];
        assert_eq!(super::guardrail_trips(&[run]).get("idle"), Some(&2));
    }

    #[test]
    fn board_throughput() {
        assert_eq!(
            super::board_throughput(&[BoardTransitionMetric {
                task_id: TaskId::generate(),
                from: None,
                to: TaskColumn::Done
            }]),
            1
        );
    }

    #[test]
    fn arbiter_rates() {
        assert_eq!(
            super::arbiter_rates(&[FailureClass::SpecGap, FailureClass::Ambiguity])
                .spec_gap_percent,
            50.0
        );
    }

    #[test]
    fn iterations_per_task() {
        let mut first = run("builder", 1, Some(80));
        first.task_id = Some(TaskId::new(uuid::Uuid::from_u128(1)));
        let mut second = first.clone();
        second.iterations = 4;
        assert_eq!(super::iterations_per_task(&[first, second]), Some(3.0));
    }

    #[test]
    fn gate_precision() {
        assert_eq!(
            super::gate_precision(&[
                GateOutcome {
                    found_issue: true,
                    correct_issue: true
                },
                GateOutcome {
                    found_issue: true,
                    correct_issue: false
                }
            ]),
            Some(50.0)
        );
    }

    #[test]
    fn agent_trust() {
        let trust = super::agent_trust(&[run("builder", 1, Some(80))]);
        assert_eq!(trust[0].agent, "builder");
        assert!(trust[0].score > 0.0);
    }

    #[test]
    fn trust_discounts() {
        let mut run = run("builder", 1, Some(80));
        run.guardrail_trips.push("stuck".into());
        run.rejected_artifacts = 1;
        assert!(super::trust_discounts(&[run]).score < 1.0);
    }

    #[test]
    fn trust_by_tokens() {
        assert_eq!(
            super::trust_by_tokens(&[run("builder", 1, Some(80))]),
            Some(120.0)
        );
    }

    #[test]
    fn cache_alert() {
        assert!(super::cache_alert(Some(79.9)));
        assert!(!super::cache_alert(None));
    }

    #[test]
    fn detects_prefix_drift() {
        let mut run = run("builder", 1, Some(20));
        run.prefix_id = Some("unstable-prefix".into());
        let drift = super::detects_prefix_drift(&[run]).expect("drift");
        assert_eq!(drift.prefix_id, "unstable-prefix");
    }
}
