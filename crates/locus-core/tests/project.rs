mod project {
    use locus_core::project::ProjectSettings;

    #[test]
    fn settings_roundtrip() {
        let settings = ProjectSettings::new();
        let stored = settings.to_stored_value().expect("settings serialize");

        assert_eq!(ProjectSettings::from_stored_value(stored).expect("settings deserialize"), settings);
    }
}
