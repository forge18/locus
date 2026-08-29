use std::{fs, path::PathBuf, process::Command};

fn temporary_directory(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "locus-cli-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&path).expect("create temporary directory");
    path
}

#[test]
fn lint_json_flag_is_accepted_and_output_is_compact() {
    let root = temporary_directory("lint-json");
    let linters = root.join("linters");
    let project = root.join("project");
    fs::create_dir_all(&linters).expect("create linter directory");
    fs::create_dir_all(&project).expect("create project directory");
    fs::write(linters.join("format.sh"), "printf clean").expect("write linter");
    fs::write(linters.join("format.md"), "formatting rule").expect("write linter rule");

    let output = Command::new(env!("CARGO_BIN_EXE_locus"))
        .args(["lint", "--only", "format", "--json"])
        .env("LOCUS_LINTER_ROOT", &linters)
        .env("LOCUS_WORKSPACE", &project)
        .output()
        .expect("locus lint --json runs");
    let _ = fs::remove_dir_all(&root);

    assert!(
        output.status.success(),
        "locus lint --json failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert_eq!(stdout.matches('\n').count(), 1, "JSON output is one line");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(stdout.trim_end()).expect("JSON output parses"),
        serde_json::json!({
            "passed": true,
            "results": [{"name": "format", "passed": true, "stdout": "clean"}]
        })
    );
}
