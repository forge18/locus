mod sandbox {
    use locus_core::{
        sandbox::image::agent_image_tag_for_scopes,
        services::tools::{
            ImageTool, ProjectToolScope, RoleToolScope, ToolCatalog, TrustedKeyStore,
        },
    };

    #[test]
    fn tool_set_rebuild() {
        let current = vec![ImageTool::new("git", "2")];
        assert!(!locus_core::sandbox::image::tool_set_requires_rebuild(
            &current, &current
        ));
    }

    #[test]
    fn role_tools_absent() {
        let mut catalog = ToolCatalog::new(TrustedKeyStore::default());
        catalog
            .add_builtin(ImageTool::new("git", "2.49"))
            .expect("git");
        catalog
            .add_builtin(ImageTool::new("sqlx", "0.8"))
            .expect("sqlx");

        let tag = agent_image_tag_for_scopes(
            "base",
            &catalog,
            &ProjectToolScope::default(),
            &RoleToolScope::new(["sqlx"]),
        );
        let git_only = agent_image_tag_for_scopes(
            "base",
            &catalog,
            &ProjectToolScope::new(["sqlx"]),
            &RoleToolScope::default(),
        );
        assert_eq!(tag, git_only);
    }
}
