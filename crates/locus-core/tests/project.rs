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
}
