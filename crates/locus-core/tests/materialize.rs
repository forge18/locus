mod materialize {
    use std::{fs, path::PathBuf};

    use locus_core::harness::{
        materialize::{
            extensions::ExtensionEntry, extensions::ExtensionSet,
            extensions::ProjectExtensionScope, materialize_project,
        },
        registry::load_from_directory,
    };
    use serde_json::json;

    #[test]
    fn disabled_extensions_absent() {
        let mut extensions = ExtensionSet::default();
        extensions.insert(
            "rules",
            vec![ExtensionEntry::new("secret.md", json!({}), "never commit")],
        );
        let mut scope = ProjectExtensionScope::default();
        scope.disable_extension("rules");
        let root = PathBuf::from(format!("/tmp/locus-materialize-{}", std::process::id()));
        let registry = load_from_directory(concat!(env!("CARGO_MANIFEST_DIR"), "/../../harnesses"))
            .expect("registry");

        let (tree, _) = materialize_project(
            registry.by_name("claude").expect("claude"),
            &extensions,
            &scope,
            &root,
            None,
        )
        .expect("materialize");
        assert!(tree.file("rules/secret.md").is_none());
        let _ = fs::remove_dir_all(root);
    }
}
