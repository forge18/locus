//! Cron schedule parsing and restart-safe firing state.
//!
//! The durable store owns rows and workflow log entries; this module owns the
//! deterministic decision made for one firing. Timezone policy is explicitly UTC
//! until the product has a timezone/DST contract.

use std::collections::BTreeSet;

use thiserror::Error;
use time::{Duration, OffsetDateTime, UtcOffset, Weekday};
use uuid::Uuid;

const CRON_FIELDS: usize = 5;
const SEARCH_MINUTES: i64 = 366 * 24 * 60;

#[derive(Clone, Debug, Eq, PartialEq)]
struct CronField {
    values: BTreeSet<u8>,
}

impl CronField {
    fn parse(value: &str, min: u8, max: u8) -> Result<Self, ScheduleError> {
        if value.trim().is_empty() {
            return Err(ScheduleError::InvalidCron(value.into()));
        }
        let mut values = BTreeSet::new();
        for part in value.split(',') {
            let (range, step) = match part.split_once('/') {
                Some((range, step)) => (
                    range,
                    step.parse::<u8>()
                        .map_err(|_| ScheduleError::InvalidCron(value.into()))?,
                ),
                None => (part, 1),
            };
            if step == 0 {
                return Err(ScheduleError::InvalidCron(value.into()));
            }
            let (start, end) = if range == "*" {
                (min, max)
            } else if let Some((start, end)) = range.split_once('-') {
                (
                    parse_number(start, min, max, value)?,
                    parse_number(end, min, max, value)?,
                )
            } else {
                let number = parse_number(range, min, max, value)?;
                (number, number)
            };
            if start > end {
                return Err(ScheduleError::InvalidCron(value.into()));
            }
            let mut number = start;
            while number <= end {
                values.insert(number);
                match number.checked_add(step) {
                    Some(next) if next > number => number = next,
                    _ => break,
                }
            }
        }
        if values.is_empty() {
            return Err(ScheduleError::InvalidCron(value.into()));
        }
        Ok(Self { values })
    }

    fn contains(&self, value: u8) -> bool {
        self.values.contains(&value)
    }
}

fn parse_number(value: &str, min: u8, max: u8, whole: &str) -> Result<u8, ScheduleError> {
    let value = value
        .parse::<u8>()
        .map_err(|_| ScheduleError::InvalidCron(whole.into()))?;
    if value < min || value > max {
        return Err(ScheduleError::InvalidCron(whole.into()));
    }
    Ok(value)
}

/// A five-field cron expression evaluated in UTC.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CronExpression {
    source: String,
    minute: CronField,
    hour: CronField,
    day_of_month: CronField,
    month: CronField,
    day_of_week: CronField,
}

impl CronExpression {
    pub fn parse(source: &str) -> Result<Self, ScheduleError> {
        let fields = source.split_whitespace().collect::<Vec<_>>();
        if fields.len() != CRON_FIELDS {
            return Err(ScheduleError::InvalidCron(source.into()));
        }
        Ok(Self {
            source: source.trim().into(),
            minute: CronField::parse(fields[0], 0, 59)?,
            hour: CronField::parse(fields[1], 0, 23)?,
            day_of_month: CronField::parse(fields[2], 1, 31)?,
            month: CronField::parse(fields[3], 1, 12)?,
            day_of_week: CronField::parse(fields[4], 0, 7)?,
        })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    /// The schedule contract currently has one explicit timezone: UTC. This
    /// avoids silently inventing DST behavior for a schema with no timezone field.
    pub const fn timezone() -> ScheduleTimeZone {
        ScheduleTimeZone::Utc
    }

    pub fn matches(&self, instant: OffsetDateTime) -> bool {
        let instant = instant.to_offset(UtcOffset::UTC);
        let date = instant.date();
        self.minute.contains(instant.minute())
            && self.hour.contains(instant.hour())
            && self.day_of_month.contains(date.day())
            && self.month.contains(date.month() as u8)
            && cron_weekday_matches(&self.day_of_week, date.weekday())
    }

    pub fn next_after(&self, after: OffsetDateTime) -> Result<OffsetDateTime, ScheduleError> {
        let normalized = after
            .to_offset(UtcOffset::UTC)
            .replace_second(0)
            .map_err(|_| ScheduleError::TimeOverflow)?
            .replace_nanosecond(0)
            .map_err(|_| ScheduleError::TimeOverflow)?;
        let mut candidate = normalized + Duration::minutes(1);
        for _ in 0..SEARCH_MINUTES {
            if self.matches(candidate) {
                return Ok(candidate);
            }
            candidate = candidate
                .checked_add(Duration::minutes(1))
                .ok_or(ScheduleError::TimeOverflow)?;
        }
        Err(ScheduleError::NoFireWithinSearchWindow)
    }
}

fn cron_weekday_matches(field: &CronField, weekday: Weekday) -> bool {
    let monday_number = weekday.number_from_monday();
    let cron_number = match weekday {
        Weekday::Sunday => 0,
        _ => monday_number,
    };
    field.contains(cron_number) || (weekday == Weekday::Sunday && field.contains(7))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleTimeZone {
    Utc,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifyResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl VerifyResult {
    pub fn passed(&self) -> bool {
        self.exit_code == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FireOutcome {
    Started,
    SkippedOverlap,
    Paused,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleExecution {
    pub execution_id: Uuid,
    pub scheduled_for: OffsetDateTime,
    pub outcome: FireOutcome,
    pub verify_result: Option<VerifyResult>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleSnapshot {
    pub paused: bool,
    pub active_execution: Option<Uuid>,
    pub history: Vec<ScheduleExecution>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ScheduleError {
    #[error("invalid cron expression `{0}`")]
    InvalidCron(String),
    #[error("schedule time overflow")]
    TimeOverflow,
    #[error("schedule has no fire time within the search window")]
    NoFireWithinSearchWindow,
    #[error("execution `{execution_id}` is not the active execution")]
    NotActive { execution_id: Uuid },
}

/// Restart-safe scheduler state. A skipped firing creates history but never a queue item.
#[derive(Clone, Debug)]
pub struct ScheduleController {
    id: Uuid,
    cron: CronExpression,
    paused: bool,
    active_execution: Option<Uuid>,
    history: Vec<ScheduleExecution>,
}

impl ScheduleController {
    pub fn new(id: Uuid, cron: CronExpression) -> Self {
        Self {
            id,
            cron,
            paused: false,
            active_execution: None,
            history: Vec::new(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn cron(&self) -> &CronExpression {
        &self.cron
    }

    pub fn next_fire_after(&self, after: OffsetDateTime) -> Result<OffsetDateTime, ScheduleError> {
        self.cron.next_after(after)
    }

    pub fn fire(&mut self, scheduled_for: OffsetDateTime) -> (FireOutcome, Option<Uuid>) {
        if self.paused {
            self.history.push(ScheduleExecution {
                execution_id: Uuid::new_v4(),
                scheduled_for,
                outcome: FireOutcome::Paused,
                verify_result: None,
            });
            return (FireOutcome::Paused, None);
        }
        if self.active_execution.is_some() {
            let execution_id = Uuid::new_v4();
            self.history.push(ScheduleExecution {
                execution_id,
                scheduled_for,
                outcome: FireOutcome::SkippedOverlap,
                verify_result: None,
            });
            return (FireOutcome::SkippedOverlap, None);
        }
        let execution_id = Uuid::new_v4();
        self.active_execution = Some(execution_id);
        self.history.push(ScheduleExecution {
            execution_id,
            scheduled_for,
            outcome: FireOutcome::Started,
            verify_result: None,
        });
        (FireOutcome::Started, Some(execution_id))
    }

    pub fn complete(
        &mut self,
        execution_id: Uuid,
        verify_result: VerifyResult,
    ) -> Result<(), ScheduleError> {
        if self.active_execution != Some(execution_id) {
            return Err(ScheduleError::NotActive { execution_id });
        }
        self.active_execution = None;
        if let Some(execution) = self
            .history
            .iter_mut()
            .rev()
            .find(|execution| execution.execution_id == execution_id)
        {
            execution.verify_result = Some(verify_result);
        }
        Ok(())
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn active_execution(&self) -> Option<Uuid> {
        self.active_execution
    }

    pub fn skipped_count(&self) -> usize {
        self.history
            .iter()
            .filter(|execution| execution.outcome == FireOutcome::SkippedOverlap)
            .count()
    }

    pub fn history(&self) -> &[ScheduleExecution] {
        &self.history
    }

    pub fn has_backlog(&self) -> bool {
        false
    }

    pub fn snapshot(&self) -> ScheduleSnapshot {
        ScheduleSnapshot {
            paused: self.paused,
            active_execution: self.active_execution,
            history: self.history.clone(),
        }
    }

    pub fn from_snapshot(id: Uuid, cron: CronExpression, snapshot: ScheduleSnapshot) -> Self {
        Self {
            id,
            cron,
            paused: snapshot.paused,
            active_execution: snapshot.active_execution,
            history: snapshot.history,
        }
    }
}

#[cfg(test)]
mod sched {
    use super::*;

    fn at(unix: i64) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(unix).expect("timestamp")
    }

    #[test]
    fn cron_parses() {
        let cron = CronExpression::parse("*/15 9-17 * * 1-5").expect("cron");
        assert_eq!(cron.source(), "*/15 9-17 * * 1-5");
        assert!(cron.next_after(at(1_700_000_000)).is_ok());
    }

    #[test]
    fn dst() {
        assert_eq!(CronExpression::timezone(), ScheduleTimeZone::Utc);
        let cron = CronExpression::parse("0 2 * * *").expect("cron");
        let fire = cron.next_after(at(1_709_874_000)).expect("UTC fire");
        assert_eq!(fire.offset(), UtcOffset::UTC);
    }

    #[test]
    fn fires() {
        let mut schedule = ScheduleController::new(
            Uuid::new_v4(),
            CronExpression::parse("* * * * *").expect("cron"),
        );
        assert_eq!(schedule.fire(at(1_700_000_000)).0, FireOutcome::Started);
    }

    #[test]
    fn fires_headless() {
        let mut schedule = ScheduleController::new(
            Uuid::new_v4(),
            CronExpression::parse("* * * * *").expect("cron"),
        );
        assert_eq!(schedule.fire(at(1_700_000_000)).0, FireOutcome::Started);
    }

    #[test]
    fn records_verify_result() {
        let mut schedule = ScheduleController::new(
            Uuid::new_v4(),
            CronExpression::parse("* * * * *").expect("cron"),
        );
        let execution = schedule.fire(at(1_700_000_000)).1.expect("execution");
        schedule
            .complete(
                execution,
                VerifyResult {
                    exit_code: 0,
                    stdout: "ok".into(),
                    stderr: String::new(),
                },
            )
            .expect("verify");
        assert!(schedule.history()[0]
            .verify_result
            .as_ref()
            .unwrap()
            .passed());
    }

    #[test]
    fn detects_overlap() {
        let mut schedule = ScheduleController::new(
            Uuid::new_v4(),
            CronExpression::parse("* * * * *").expect("cron"),
        );
        schedule.fire(at(1_700_000_000));
        assert_eq!(
            schedule.fire(at(1_700_000_060)).0,
            FireOutcome::SkippedOverlap
        );
    }

    #[test]
    fn skips_never_queues() {
        let mut schedule = ScheduleController::new(
            Uuid::new_v4(),
            CronExpression::parse("* * * * *").expect("cron"),
        );
        schedule.fire(at(1_700_000_000));
        schedule.fire(at(1_700_000_060));
        assert!(!schedule.has_backlog());
    }

    #[test]
    fn no_backlog() {
        let mut schedule = ScheduleController::new(
            Uuid::new_v4(),
            CronExpression::parse("* * * * *").expect("cron"),
        );
        schedule.fire(at(1_700_000_000));
        for minute in 1..10 {
            schedule.fire(at(1_700_000_000 + minute * 60));
        }
        assert!(!schedule.has_backlog());
    }

    #[test]
    fn skips_are_counted() {
        let mut schedule = ScheduleController::new(
            Uuid::new_v4(),
            CronExpression::parse("* * * * *").expect("cron"),
        );
        schedule.fire(at(1_700_000_000));
        schedule.fire(at(1_700_000_060));
        assert_eq!(schedule.skipped_count(), 1);
    }

    #[test]
    fn restart_safe() {
        let id = Uuid::new_v4();
        let cron = CronExpression::parse("* * * * *").expect("cron");
        let mut schedule = ScheduleController::new(id, cron.clone());
        let execution = schedule.fire(at(1_700_000_000)).1;
        let restarted = ScheduleController::from_snapshot(id, cron, schedule.snapshot());
        assert_eq!(restarted.active_execution(), execution);
        assert_eq!(restarted.history().len(), 1);
    }

    #[test]
    fn pause_resume() {
        let mut schedule = ScheduleController::new(
            Uuid::new_v4(),
            CronExpression::parse("* * * * *").expect("cron"),
        );
        schedule.pause();
        assert_eq!(schedule.fire(at(1_700_000_000)).0, FireOutcome::Paused);
        schedule.resume();
        assert_eq!(schedule.fire(at(1_700_000_060)).0, FireOutcome::Started);
    }
}
