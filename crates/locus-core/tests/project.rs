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
        harness::materialize::extensions::ProjectExtensionScope,
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
    fn workflows() {
        let execution_id = uuid::Uuid::new_v4();
        let payload = locus_core::services::workflow::ExecutionEntryPayload {
            execution_id,
            workflow_def_id: uuid::Uuid::new_v4(),
            schedule_id: None,
            status: "running".into(),
            scheduled_for: None,
            started_at: None,
            ended_at: None,
        };
        let entry = locus_core::services::workflow::WorkflowEntry::new(
            locus_core::ids::ProjectId::generate(),
            1,
            locus_core::services::workflow::WorkflowEntryKind::Execution,
            1,
            serde_json::to_value(payload).expect("payload"),
            "system",
            None,
        );
        let projection = locus_core::services::workflow::WorkflowsProjection::rebuild([entry])
            .expect("workflow projection");
        assert_eq!(
            projection.execution(execution_id).unwrap().status,
            "running"
        );
    }

    #[test]
    fn board() {
        let task = locus_core::services::board::BoardTask::new(
            locus_core::ids::ProjectId::generate(),
            locus_core::ids::TaskId::generate(),
            "board task",
            Some("cargo test".into()),
        );
        let mut projection = locus_core::services::board::BoardProjection::default();
        projection
            .apply(locus_core::services::board::BoardEvent::Created {
                task: Box::new(task.clone()),
            })
            .expect("project board task");
        assert_eq!(projection.task(task.id).unwrap().summary, "board task");
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
