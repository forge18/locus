mod project {
    use locus_core::project::ProjectSettings;

    #[test]
    fn settings_roundtrip() {
        let settings = ProjectSettings::new();
        let stored = settings.to_stored_value().expect("settings serialize");

        assert_eq!(ProjectSettings::from_stored_value(stored).expect("settings deserialize"), settings);
    }

    #[test]
    fn harness_allow_list() {
        let settings = ProjectSettings::new()
            .with_harness_allow_list(["claude", "codex"])
            .expect("set allow-list");

        assert!(settings.permits_harness("claude"));
        assert!(settings.permits_harness("codex"));
        assert!(!settings.permits_harness("gemini"));
    }

    #[test]
    fn one_agent_default() {
        let settings = ProjectSettings::new()
            .with_harness_allow_list(["claude", "codex"])
            .expect("set allow-list")
            .with_agent_default("codex")
            .expect("set default");

        assert_eq!(settings.agent_default(), Some("codex"));
        assert!(settings.with_agent_default("gemini").is_err());
    }
}
