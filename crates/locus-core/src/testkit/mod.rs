//! Shared test fixtures: event assertions, and an isolated Postgres per test.
//!
//! Behind the `testkit` feature so it does not ship in an ordinary build. The canary
//! smoke test is NOT here — `harness::canary` gates registration on it, which makes it
//! production code.

pub mod postgres;

use anyhow::{bail, Result};

use crate::services::telemetry::{Event, EventVerb};

/// Assert that a normalized event stream contains the requested verb.
pub fn assert_event(events: &[Event], verb: EventVerb) -> Result<()> {
    if events.iter().any(|event| event.verb == verb) {
        return Ok(());
    }
    bail!("event stream did not contain `{verb}`")
}

/// Assert that a normalized event stream contains the requested verb carrying `text`.
pub fn assert_event_text(events: &[Event], verb: EventVerb, text: &str) -> Result<()> {
    if events.iter().any(|event| {
        event.verb == verb
            && event
                .text
                .as_deref()
                .is_some_and(|value| value.contains(text))
    }) {
        return Ok(());
    }
    bail!("event stream did not contain `{verb}` text `{text}`")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::RunId;
    use serde_json::json;

    fn event(verb: EventVerb, text: Option<String>) -> Event {
        Event {
            run_id: RunId::generate(),
            seq: 0,
            ts: "1970-01-01T00:00:00Z".into(),
            verb,
            text,
            tool: None,
            args: None,
            usage: None,
            raw: json!({"source": "testkit"}),
        }
    }

    #[test]
    fn event_assertions() {
        let events = vec![event(
            EventVerb::Assistant,
            Some("materialized canary".into()),
        )];
        assert_event(&events, EventVerb::Assistant).expect("assistant event exists");
        assert_event_text(&events, EventVerb::Assistant, "canary").expect("event text exists");
        assert!(assert_event(&events, EventVerb::SessionEnd).is_err());
    }
}
