//! Assertions and deterministic run preflights shared by integration tests.

use std::{
    env, fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::{bail, Context, Result};
use serde_json::json;

use crate::{
    materialize::{materialize, ExtensionEntry, ExtensionSet, PluginHost},
    registry::{HarnessDefinition, HarnessRegistry},
    telemetry::{Event, EventVerb},
};

const CANARY_SKILL: &str = include_str!("../../../tests/canary/skill.md");
const CANARY_RULE: &str = include_str!("../../../tests/canary/rule.md");
const CANARY_SKILL_MARKER: &str = "LOCUS_CI_CANARY_SKILL";
const CANARY_RULE_MARKER: &str = "LOCUS_CI_CANARY_RULE";
static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

/// Returns the fixture extensions that every harness smoke test must expose.
pub fn canary_extensions() -> ExtensionSet {
    let mut extensions = ExtensionSet::default();
    extensions.insert(
        "skills",
        vec![ExtensionEntry::new(
            "canary/skill.md",
            json!({"name": "ci-canary-skill"}),
            CANARY_SKILL,
        )],
    );
    extensions.insert(
        "rules",
        vec![ExtensionEntry::new(
            "canary/rule.md",
            json!({"paths": ["**/*"]}),
            CANARY_RULE,
        )],
    );
    extensions
}

/// A deterministic substitute for a live harness run.
///
/// It materializes the exact configuration supplied at run start and records the resulting
/// configuration as the run's event stream. Live Docker/harness verification remains deferred
/// until registered images and launch paths exist.
pub fn run_canary_smoke(harness: &HarnessDefinition) -> Result<Vec<Event>> {
    let root = smoke_root(&harness.name);
    let extensions = canary_extensions();
    let plugin = harness
        .layout
        .named_entries()
        .iter()
        .any(|(_, entry)| entry.via == "plugin")
        .then(plugin_host);
    let result = materialize(harness, &extensions, &root, plugin.as_ref())
        .with_context(|| format!("{}: materialization failed", harness.name))
        .and_then(|(tree, _)| {
            let visible = tree
                .files()
                .map(|file| file.content.as_str())
                .collect::<String>();
            if !visible.contains(CANARY_SKILL_MARKER) {
                bail!("{}: canary skill is not visible", harness.name);
            }
            if !visible.contains(CANARY_RULE_MARKER) {
                bail!("{}: canary rule is not visible", harness.name);
            }
            Ok(vec![
                event(&harness.name, 0, EventVerb::SessionStart, None),
                event(&harness.name, 1, EventVerb::Assistant, Some(visible)),
                event(&harness.name, 2, EventVerb::SessionEnd, None),
            ])
        });
    let _ = fs::remove_dir_all(root);
    result
}

/// Run the deterministic preflight for every definition before accepting a registry.
pub fn smoke_registry(registry: &HarnessRegistry) -> Result<()> {
    for harness in registry.iter() {
        run_canary_smoke(harness)?;
    }
    Ok(())
}

/// Assert that a normalized event stream contains the requested verb.
pub fn assert_event(events: &[Event], verb: EventVerb) -> Result<()> {
    if events.iter().any(|event| event.verb == verb) {
        return Ok(());
    }
    bail!("event stream did not contain `{verb}`")
}

/// Assert that a normalized event stream contains text from an event of the requested verb.
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

fn event(run_id: &str, seq: u64, verb: EventVerb, text: Option<String>) -> Event {
    Event {
        run_id: run_id.into(),
        seq,
        ts: "1970-01-01T00:00:00Z".into(),
        verb,
        text,
        tool: None,
        args: None,
        usage: None,
        raw: json!({"source": "ci-canary-smoke"}),
    }
}

fn plugin_host() -> PluginHost {
    PluginHost {
        program: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses/pi/materialize"),
        args: Vec::new(),
    }
}

fn smoke_root(harness: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "locus-ci-smoke-{harness}-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed),
    ))
}

#[cfg(test)]
#[test]
fn event_assertions() {
    let events = vec![event(
        "run",
        0,
        EventVerb::Assistant,
        Some("materialized canary".into()),
    )];
    assert_event(&events, EventVerb::Assistant).expect("assistant event exists");
    assert_event_text(&events, EventVerb::Assistant, "canary").expect("event text exists");
    assert!(assert_event(&events, EventVerb::SessionEnd).is_err());
}
