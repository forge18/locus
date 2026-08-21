//! Fixture for Spike 2. Small enough that rust-analyzer indexes it in seconds,
//! and shaped so that every LSP feature the editor needs has something to bite:
//! a symbol to complete, a doc comment to hover, a definition to jump to, a
//! second use site to find as a reference, and one deliberate type error.

/// Sums a slice. The doc comment is here so `hover` has something to return.
pub fn total(values: &[i64]) -> i64 {
    values.iter().sum()
}

pub fn average(values: &[i64]) -> i64 {
    // A second call site, so `find references` has more than one answer.
    total(values) / values.len() as i64
}

fn main() {
    let values = vec![1, 2, 3];

    // Deliberately wrong. rust-analyzer flags this natively — it does not need
    // `cargo check` to have run — so `textDocument/publishDiagnostics` has
    // something real to push. This fixture is not meant to compile.
    let _mismatch: i64 = "not a number";

    println!("{} {}", total(&values), average(&values));
}
