//! `.specs/agent-cli` acceptance 1: "Every verb is a socket round trip; a test asserts
//! the CLI computes no domain answer locally."
//!
//! This is that test. It pins the exact set of commands `main` answers without the
//! socket, so adding a new one is a deliberate edit here rather than a quiet drift.

use std::{fs, path::PathBuf};

/// Commands `main` dispatches locally, and why each is not an agent verb.
///
/// `hook` is the harness callback shim, not something an agent types.
/// `backup` and `harness` are host-side operator and CI commands: they run outside any
/// container, against paths and a database an agent cannot reach.
///
/// `lint` is the exception, and it is a real conflict: it is an agent verb, it is in the
/// socket allowlist, and `main` still answers it locally — so the allowlist entry is
/// unreachable. It stays local only until the daemon that would serve it exists.
/// `ralph` is a local workflow helper and is not an agent-facing socket verb.
const LOCAL_COMMANDS: &[&str] = &["backup", "harness", "hook", "lint", "ralph"];

fn main_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
    fs::read_to_string(path).expect("read locus-cli main.rs")
}

#[test]
fn only_the_declared_commands_are_answered_locally() {
    let source = main_source();
    let dispatch = source
        .split("fn main()")
        .nth(1)
        .expect("main exists")
        .split("fn dispatch(")
        .next()
        .expect("main body ends before dispatch");

    let mut local = dispatch
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("Some(\"")?;
            rest.split('"').next()
        })
        .collect::<Vec<_>>();
    local.sort_unstable();
    local.dedup();

    let mut expected = LOCAL_COMMANDS.to_vec();
    expected.sort_unstable();

    assert_eq!(
        local, expected,
        "locus answers a command locally that this test does not account for; \
         every verb an agent types must be a socket round trip"
    );
}

#[test]
fn lint_is_the_one_command_that_is_both_local_and_allowlisted() {
    // Recorded so the conflict is visible rather than latent: the socket allowlist
    // carries `lint`, but `main` intercepts it first, so that entry never runs.
    let source = main_source();
    assert!(
        source.contains("Some(\"lint\") => lint("),
        "if `lint` stopped being answered locally, drop it from LOCAL_COMMANDS \
         and delete this test — the socket allowlist entry becomes live"
    );
}
