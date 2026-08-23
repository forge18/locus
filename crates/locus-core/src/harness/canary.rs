//! The per-harness canary smoke test, run on registration and in CI.
//!
//! PLAN.md §Risks calls this the mitigation for a harness surface that changes silently:
//! materialize a canary skill and a canary rule, then assert the agent can see both. That
//! turns a silent non-load into a failing test, which is the only reason the `emits` and
//! `via` declarations are worth writing down.
//!
//! Production, not test scaffolding — `register_from_directory` gates on it.

use std::{env, fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    harness::{
        materialize::{
            extensions::ExtensionEntry, extensions::ExtensionSet, materialize, plugin::PluginHost,
        },
        registry::{HarnessDefinition, HarnessRegistry, Via},
    },
    ids::RunId,
    services::telemetry::{Event, EventVerb},
};

const CANARY_SKILL: &str = include_str!("../../../../tests/canary/skill.md");
const CANARY_RULE: &str = include_str!("../../../../tests/canary/rule.md");
const CANARY_SKILL_MARKER: &str = "LOCUS_CI_CANARY_SKILL";
const CANARY_RULE_MARKER: &str = "LOCUS_CI_CANARY_RULE";

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
        .any(|(_, entry)| entry.via == Via::Plugin)
        .then(|| plugin_host(harness))
        .flatten();
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
                event(
                    canary_run_id(&harness.name),
                    0,
                    EventVerb::SessionStart,
                    None,
                ),
                event(
                    canary_run_id(&harness.name),
                    1,
                    EventVerb::Assistant,
                    Some(visible),
                ),
                event(canary_run_id(&harness.name), 2, EventVerb::SessionEnd, None),
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

fn event(run_id: RunId, seq: u64, verb: EventVerb, text: Option<String>) -> Event {
    Event {
        run_id,
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

fn plugin_host(harness: &HarnessDefinition) -> Option<PluginHost> {
    harness.materializer_program().map(|program| PluginHost {
        program,
        args: Vec::new(),
    })
}

/// The canary never starts a container, so it has no real run. A deterministic id
/// derived from the harness name keeps the fabricated event stream well-typed without
/// pretending a run happened.
fn canary_run_id(harness: &str) -> RunId {
    let digest = Sha256::digest(harness.as_bytes());
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    RunId::new(uuid::Uuid::from_bytes(bytes))
}

fn smoke_root(harness: &str) -> PathBuf {
    env::temp_dir().join(format!("locus-ci-smoke-{harness}-{}", uuid::Uuid::new_v4(),))
}
