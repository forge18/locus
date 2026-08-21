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
        "12 harness definitions passed validation\n"
    );
}
