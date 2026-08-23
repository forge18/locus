mod project {
    #[test]
    fn lifecycle_preserves_history() {
        let project =
            locus_core::services::project::ProjectLifecycle::new("alpha").expect("project");
        let archived = project.rename("beta").archive();
        assert_eq!(archived.name(), "beta");
        assert!(archived.is_archived());
        assert!(archived.preserves_history());
    }

    use locus_core::{
        harness::materialize::ProjectExtensionScope,
        services::{
            project::{ProjectAnalytics, ProjectRepo, ProjectRunAnalytics, ProjectSettings},
            tools::ProjectToolScope,
        },
    };

    #[test]
    fn settings_roundtrip() {
        let settings = ProjectSettings::new();
        let stored = settings.to_stored_value().expect("settings serialize");

        assert_eq!(
            ProjectSettings::from_stored_value(stored).expect("settings deserialize"),
            settings
        );
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
    fn analytics() {
        let analytics = ProjectAnalytics::from_runs([
            ProjectRunAnalytics::new("claude", 100, 80, 3),
            ProjectRunAnalytics::new("claude", 50, 40, 2),
        ]);

        assert_eq!(analytics.model("claude").expect("model").tokens, 150);
        assert_eq!(
            analytics.model("claude").expect("model").cache_read_tokens,
            120
        );
        assert_eq!(analytics.model("claude").expect("model").spend_micros, 5);
    }

    #[test]
    fn tool_scope() {
        let scope = ProjectToolScope::new(["psql"]);
        let settings = ProjectSettings::new().with_tool_scope(scope.clone());

        assert_eq!(settings.tool_scope(), &scope);
    }

    #[test]
    fn extension_overrides() {
        let mut overrides = ProjectExtensionScope::default();
        overrides.disable_extension("hooks");
        overrides.disable_entry("skills", "review");
        let settings = ProjectSettings::new().with_extension_overrides(overrides.clone());

        assert_eq!(settings.extension_overrides(), &overrides);
    }

    #[test]
    fn base_context() {
        let settings = ProjectSettings::new()
            .with_base_context("Use cargo test.", 1_500)
            .expect("set base context");

        assert_eq!(settings.base_context(), Some("Use cargo test."));
        assert_eq!(settings.base_context_token_budget(), Some(1_500));
    }

    #[test]
    fn repos() {
        let repo = ProjectRepo::new("core", "/work/core").expect("valid repo");

        assert_eq!(repo.name, "core");
        assert_eq!(repo.working_copy_path, "/work/core");
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
