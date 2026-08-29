use std::process::Command;

#[test]
fn validates_the_registered_harnesses() {
    let output = Command::new(env!("CARGO_BIN_EXE_locus"))
        .args(["harness", "lint"])
        .output()
        .expect("locus harness lint runs");

    assert!(
        output.status.success(),
        "locus harness lint failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is UTF-8"),
        "11 harness definitions passed validation\n"
    );
}

#[test]
fn json_output_is_compact_and_structured() {
    let output = Command::new(env!("CARGO_BIN_EXE_locus"))
        .args(["harness", "lint", "--json"])
        .output()
        .expect("locus harness lint --json runs");

    assert!(
        output.status.success(),
        "locus harness lint --json failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert_eq!(stdout.matches('\n').count(), 1, "JSON output is one line");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(stdout.trim_end()).expect("JSON output parses"),
        serde_json::json!({"passed": true, "harnesses": 11})
    );
}
