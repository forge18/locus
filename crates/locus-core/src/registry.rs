#[cfg(test)]
#[test]
fn schema_parses() {
    const DEFINITIONS: &[&str] = &[
        include_str!("../../../harnesses/aider.toml"),
        include_str!("../../../harnesses/antigravity.toml"),
        include_str!("../../../harnesses/claude.toml"),
        include_str!("../../../harnesses/codex.toml"),
        include_str!("../../../harnesses/copilot.toml"),
        include_str!("../../../harnesses/cursor.toml"),
        include_str!("../../../harnesses/dsh.toml"),
        include_str!("../../../harnesses/gemini.toml"),
        include_str!("../../../harnesses/hermes.toml"),
        include_str!("../../../harnesses/omp.toml"),
        include_str!("../../../harnesses/opencode.toml"),
        include_str!("../../../harnesses/pi.toml"),
    ];

    for definition in DEFINITIONS {
        let definition: HarnessDefinition =
            toml::from_str(definition).expect("harness definition matches the schema");
        assert!(!definition.name.is_empty());
        assert!(!definition.binary.is_empty());
    }
}
