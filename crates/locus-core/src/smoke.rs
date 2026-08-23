//! CI canary smoke checks for the registered harness surface.
//!
//! These tests are intentionally deterministic until the sandbox owns verified images and launch
//! paths. They verify the materialized configuration and its normalized event stream, not a live
//! model response.

#[cfg(test)]
use std::{fs, path::PathBuf};

#[cfg(test)]
use crate::{
    harness::registry::{load_from_directory, register_from_directory},
    services::telemetry::EventVerb,
    testkit::{assert_event, assert_event_text, run_canary_smoke, smoke_registry},
};

#[cfg(test)]
fn registry() -> crate::harness::registry::HarnessRegistry {
    load_from_directory(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses"))
        .expect("registry loads")
}

/// Kept as an explicit ignored target so CI distinguishes this preflight from the future live
/// Docker test it replaces. The workflow invokes it with `--ignored`.
#[cfg(test)]
#[test]
#[ignore = "requires Docker for the future live-harness smoke; deterministic preflight runs now"]
fn canary_visible() {
    let events = run_canary_smoke(registry().by_name("claude").expect("claude exists"))
        .expect("canaries are materialized");
    assert_event(&events, EventVerb::SessionStart).expect("run starts");
    assert_event_text(&events, EventVerb::Assistant, "LOCUS_CI_CANARY_SKILL")
        .expect("skill is visible");
    assert_event_text(&events, EventVerb::Assistant, "LOCUS_CI_CANARY_RULE")
        .expect("rule is visible");
    assert_event(&events, EventVerb::SessionEnd).expect("run ends");
}

#[cfg(test)]
#[test]
#[ignore = "requires Docker for the future live-harness smoke; deterministic preflight runs now"]
fn all_registered_harnesses() {
    let registry = registry();
    for harness in registry.iter() {
        let events =
            run_canary_smoke(harness).unwrap_or_else(|error| panic!("{}: {error}", harness.name));
        assert_event_text(&events, EventVerb::Assistant, "LOCUS_CI_CANARY_SKILL")
            .unwrap_or_else(|error| panic!("{}: {error}", harness.name));
        assert_event_text(&events, EventVerb::Assistant, "LOCUS_CI_CANARY_RULE")
            .unwrap_or_else(|error| panic!("{}: {error}", harness.name));
    }
}

#[cfg(test)]
#[test]
#[ignore = "requires Docker for the future live-harness smoke; deterministic preflight runs now"]
fn isolates_failure() {
    let root = copy_registry();
    break_claude_rules(&root.join("claude.toml"));

    let registry = load_from_directory(&root).expect("broken registry still parses");
    let failures = registry
        .iter()
        .filter_map(|harness| {
            run_canary_smoke(harness)
                .err()
                .map(|_| harness.name.as_str())
        })
        .collect::<Vec<_>>();
    let _ = fs::remove_dir_all(root);
    assert_eq!(failures, ["claude"]);
}

#[cfg(test)]
#[test]
fn gates_registration() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses");
    let registry =
        register_from_directory(&source).expect("registered harnesses pass canary smoke");
    smoke_registry(&registry).expect("registration smoke remains independently reusable");

    let root = copy_registry();
    break_claude_rules(&root.join("claude.toml"));
    let error = register_from_directory(&root).expect_err("failed smoke rejects registration");
    let _ = fs::remove_dir_all(root);
    assert!(error
        .to_string()
        .contains("harness `claude` failed its canary smoke test"));
}

#[cfg(test)]
fn break_claude_rules(path: &std::path::Path) {
    let mut definition: toml::Value = fs::read_to_string(path)
        .expect("claude definition reads")
        .parse()
        .expect("claude definition parses");
    let rules = definition["layout"]["rules"]
        .as_table_mut()
        .expect("claude rules are a table");
    rules.clear();
    rules.insert("via".into(), toml::Value::String("core-driven".into()));
    rules.insert(
        "weaker_than_native".into(),
        toml::Value::String("CI fixture deliberately drops rules".into()),
    );
    fs::write(
        path,
        toml::to_string(&definition).expect("broken definition serializes"),
    )
    .expect("broken definition writes");
}

#[cfg(test)]
fn copy_registry() -> PathBuf {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses");
    let root = std::env::temp_dir().join(format!(
        "locus-smoke-registry-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(&root).expect("fixture registry directory exists");
    for entry in fs::read_dir(source).expect("source registry reads") {
        let entry = entry.expect("registry entry reads");
        let path = entry.path();
        if path.is_dir() {
            let target = root.join(path.file_name().expect("plugin directory name"));
            fs::create_dir(&target).expect("plugin directory copies");
            for child in fs::read_dir(path).expect("plugin reads") {
                let child = child.expect("plugin entry reads");
                fs::copy(child.path(), target.join(child.file_name()))
                    .expect("plugin entry copies");
            }
        } else if path
            .extension()
            .is_some_and(|extension| extension == "toml")
        {
            fs::copy(&path, root.join(path.file_name().expect("definition name")))
                .expect("definition copies");
        }
    }
    root
}
