//! Linter discovery and execution. Linters are Locus-owned rather than harness hooks.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Context, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Linters apply to their materialized directory; path-glob scopes are intentionally unsupported.
    DirectoryOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Linter {
    pub name: String,
    pub check: PathBuf,
    pub rule: PathBuf,
    pub scope: Scope,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LintRequest {
    pub only: Option<String>,
    pub changed: bool,
    pub changed_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LintResult {
    pub name: String,
    pub stdout: String,
    pub passed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LintReport {
    pub results: Vec<LintResult>,
}

impl LintReport {
    pub fn passed(&self) -> bool {
        self.results.iter().all(|result| result.passed)
    }

    /// The exact stdout preserved as board-transition evidence.
    pub fn evidence(&self) -> String {
        self.results
            .iter()
            .map(|result| result.stdout.as_str())
            .collect::<Vec<_>>()
            .join("")
    }
}

/// Discover `<name>.sh` and its required `<name>.md` rule in one materialized linters directory.
pub fn discover(root: impl AsRef<Path>) -> Result<Vec<Linter>> {
    let root = root.as_ref();
    let mut linters = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("read linters `{}`", root.display()))? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("sh") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .context("linter filename is not UTF-8")?
            .to_owned();
        let rule = root.join(format!("{name}.md"));
        if !rule.is_file() {
            bail!("linter `{name}` is missing rule file `{}`", rule.display());
        }
        linters.push(Linter {
            name,
            check: path,
            rule,
            scope: Scope::DirectoryOnly,
        });
    }
    linters.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(linters)
}

/// Validates authored linter filenames before materialization. A linter is a check plus its rule.
pub fn validate_filenames(names: impl IntoIterator<Item = impl AsRef<str>>) -> Result<()> {
    let names = names
        .into_iter()
        .map(|name| name.as_ref().to_owned())
        .collect::<BTreeSet<_>>();
    for check in names.iter().filter(|name| name.ends_with(".sh")) {
        let rule = format!("{}.md", check.trim_end_matches(".sh"));
        if !names.contains(&rule) {
            bail!("linter `{check}` is missing rule file `{rule}`");
        }
    }
    Ok(())
}

/// Run all selected linters from a project directory. `--changed` passes only run-diff paths.
pub fn run(
    linters_root: impl AsRef<Path>,
    project_root: impl AsRef<Path>,
    request: &LintRequest,
) -> Result<LintReport> {
    let project_root = project_root.as_ref();
    let linters = discover(linters_root)?;
    if let Some(name) = &request.only {
        if !linters.iter().any(|linter| &linter.name == name) {
            bail!("unknown linter `{name}`");
        }
    }

    let mut results = Vec::new();
    for linter in linters.iter().filter(|linter| {
        request
            .only
            .as_deref()
            .is_none_or(|name| name == linter.name)
    }) {
        let mut command = Command::new("sh");
        command.arg(&linter.check).current_dir(project_root);
        if request.changed {
            command.args(&request.changed_paths);
        }
        let output = command
            .output()
            .with_context(|| format!("run linter `{}`", linter.name))?;
        let mut stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if !output.status.success() {
            if !stdout.ends_with('\n') && !stdout.is_empty() {
                stdout.push('\n');
            }
            stdout.push_str(
                &fs::read_to_string(&linter.rule)
                    .with_context(|| format!("read rule for linter `{}`", linter.name))?,
            );
            if !stdout.ends_with('\n') {
                stdout.push('\n');
            }
        }
        results.push(LintResult {
            name: linter.name.clone(),
            stdout,
            passed: output.status.success(),
        });
    }
    Ok(LintReport { results })
}

/// A workflow Verify node consumes this direct exit-code projection.
pub fn verify(report: &LintReport) -> Result<()> {
    if report.passed() {
        Ok(())
    } else {
        bail!("linter failed")
    }
}

#[cfg(test)]
fn root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("locus-lint-{label}-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).expect("create root");
    root
}

#[cfg(test)]
fn linter(root: &Path, name: &str, body: &str, rule: Option<&str>) {
    fs::write(root.join(format!("{name}.sh")), body).expect("write check");
    if let Some(rule) = rule {
        fs::write(root.join(format!("{name}.md")), rule).expect("write rule");
    }
}

#[test]
fn format() {
    let root = root("format");
    linter(&root, "format", "exit 0", Some("format rules"));
    assert_eq!(discover(&root).expect("discover").len(), 1);
    fs::remove_dir_all(root).expect("remove root");
}

#[test]
fn rule_file_required() {
    let root = root("rule-required");
    linter(&root, "format", "exit 0", None);
    assert!(discover(&root)
        .expect_err("missing rule fails")
        .to_string()
        .contains("format.md"));
    fs::remove_dir_all(root).expect("remove root");
}

#[test]
fn runs_all() {
    let root = root("runs-all");
    linter(&root, "first", "printf first", Some("first rule"));
    linter(&root, "second", "printf second", Some("second rule"));
    let report = run(&root, &root, &LintRequest::default()).expect("run linters");
    assert_eq!(report.results.len(), 2);
    assert!(report.passed());
    fs::remove_dir_all(root).expect("remove root");
}

#[test]
fn only() {
    let root = root("only");
    linter(&root, "first", "printf first", Some("first rule"));
    linter(&root, "second", "printf second", Some("second rule"));
    let report = run(
        &root,
        &root,
        &LintRequest {
            only: Some("second".into()),
            ..LintRequest::default()
        },
    )
    .expect("run selected linter");
    assert_eq!(
        report
            .results
            .iter()
            .map(|result| &result.name)
            .collect::<Vec<_>>(),
        ["second"]
    );
    fs::remove_dir_all(root).expect("remove root");
}

#[test]
fn changed() {
    let root = root("changed");
    linter(&root, "args", "printf '%s' \"$1\"", Some("args rule"));
    let report = run(
        &root,
        &root,
        &LintRequest {
            changed: true,
            changed_paths: vec![PathBuf::from("changed.rs")],
            ..LintRequest::default()
        },
    )
    .expect("run changed linter");
    assert_eq!(report.results[0].stdout, "changed.rs");
    fs::remove_dir_all(root).expect("remove root");
}

#[test]
fn exit_code() {
    let root = root("exit-code");
    linter(
        &root,
        "fails",
        "printf failure; exit 1",
        Some("why this matters"),
    );
    let report = run(&root, &root, &LintRequest::default()).expect("run linter");
    assert!(!report.passed());
    fs::remove_dir_all(root).expect("remove root");
}

#[test]
fn prints_the_rule() {
    let root = root("rule-output");
    linter(
        &root,
        "fails",
        "printf failure; exit 1",
        Some("why this matters"),
    );
    let report = run(&root, &root, &LintRequest::default()).expect("run linter");
    assert!(report.results[0].stdout.contains("why this matters"));
    fs::remove_dir_all(root).expect("remove root");
}

#[test]
fn verify_can_gate() {
    let root = root("verify");
    linter(&root, "fails", "exit 1", Some("rule"));
    assert!(verify(&run(&root, &root, &LintRequest::default()).expect("run")).is_err());
    fs::remove_dir_all(root).expect("remove root");
}

#[test]
fn output_is_evidence() {
    let root = root("evidence");
    linter(&root, "report", "printf evidence", Some("rule"));
    let report = run(&root, &root, &LintRequest::default()).expect("run");
    assert_eq!(report.evidence(), "evidence");
    fs::remove_dir_all(root).expect("remove root");
}

#[test]
fn scoping() {
    assert_eq!(Scope::DirectoryOnly, Scope::DirectoryOnly);
    assert!(validate_filenames(["check.sh", "check.md"]).is_ok());
}
