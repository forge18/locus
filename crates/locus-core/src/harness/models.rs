//! Project-scoped model tier settings.

use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::harness::registry::HarnessDefinition;

/// The tiers a request may fall back through, in order. A missing tier falls back UP,
/// never down: `xhigh` on a harness with no `xhigh` gets nothing, not `high`.
pub fn tier_fallback(requested_tier: &str) -> Result<&'static [&'static str]> {
    Ok(match requested_tier {
        "xhigh" => &["xhigh"],
        "high" => &["high", "xhigh"],
        "medium" => &["medium", "high", "xhigh"],
        "low" => &["low", "medium", "high", "xhigh"],
        tier => bail!("unknown model tier `{tier}`"),
    })
}

/// Build the harness arguments for a resolved model, preserving its default when unset.
pub fn launch_argv(harness: &HarnessDefinition, model_id: Option<&str>) -> Vec<String> {
    let mut argv = harness.launch.argv.clone();
    if let Some(model_id) = model_id {
        argv.push(harness.models.flag.clone());
        argv.push(model_id.into());
    }
    argv
}

/// Discover model ids from a harness, or preserve free-text entry when it cannot enumerate.
///
/// `None` differs from `Some(vec![])`: the former means Settings must offer free text, while the
/// latter means the configured discovery command returned no choices.
pub fn discover_model_ids(harness: &HarnessDefinition) -> Result<Option<Vec<String>>> {
    if harness.models.list_argv.is_empty() {
        return Ok(None);
    }

    let output = Command::new(&harness.binary)
        .args(&harness.models.list_argv)
        .output()
        .with_context(|| format!("run `{}` to discover models", harness.binary))?;
    if !output.status.success() {
        bail!(
            "`{}` model discovery failed with {}: {}",
            harness.binary,
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let stdout = String::from_utf8(output.stdout)
        .with_context(|| format!("`{}` model discovery output is not UTF-8", harness.binary))?;
    Ok(Some(
        stdout
            .lines()
            .map(str::trim)
            .filter(|model_id| !model_id.is_empty())
            .map(str::to_owned)
            .collect(),
    ))
}

#[cfg(test)]
#[test]
fn list_argv_discovery() {
    let mut harness: HarnessDefinition =
        toml::from_str(include_str!("../../../../harnesses/claude.toml"))
            .expect("reference harness definition parses");
    harness.binary = "sh".into();
    harness.models.list_argv = vec![
        "-c".into(),
        "test \"$1\" = models && test \"$2\" = list && printf 'model-low\\nmodel-high\\n'".into(),
        "model-discovery".into(),
        "models".into(),
        "list".into(),
    ];

    assert_eq!(
        discover_model_ids(&harness).expect("discover model ids"),
        Some(vec!["model-low".into(), "model-high".into()]),
        "a configured list_argv runs against the harness and provides combobox choices"
    );

    harness.binary = "not-a-real-harness".into();
    harness.models.list_argv.clear();
    assert_eq!(
        discover_model_ids(&harness).expect("free-text fallback needs no harness process"),
        None,
        "an absent list_argv preserves the free-text fallback rather than treating it as no models"
    );
}

#[cfg(test)]
#[test]
fn unset_uses_harness_default() {
    let mut harness: HarnessDefinition =
        toml::from_str(include_str!("../../../../harnesses/claude.toml"))
            .expect("reference harness definition parses");
    harness.binary = "sh".into();
    harness.launch.argv = vec!["-c".into(), "exit 0".into()];

    let argv = launch_argv(&harness, None);
    assert_eq!(
        argv, harness.launch.argv,
        "an unset model tier preserves the harness's launch arguments"
    );
    assert!(
        !argv.iter().any(|argument| argument == &harness.models.flag),
        "an unset model tier passes no model flag"
    );

    let status = Command::new(&harness.binary)
        .args(argv)
        .status()
        .expect("start the harness with its default model");
    assert!(
        status.success(),
        "the harness starts with its default model"
    );
}
