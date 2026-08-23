mod provider {
    use anyhow::Result;
    use locus_core::provider::{
        KeychainReference, KeyringKeychain, OsKeychain, ProviderBroker, ProviderReference,
    };
    use uuid::Uuid;

    struct TestKeychain;

    impl OsKeychain for TestKeychain {
        fn read_secret(&self, _: &KeychainReference) -> Result<String> {
            Ok("secret".into())
        }
        fn write_secret(&self, _: &KeychainReference, _: &str) -> Result<()> {
            Ok(())
        }
        fn delete_secret(&self, _: &KeychainReference) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn model_catalog() {
        let provider_id = Uuid::nil();
        let model = locus_core::provider::ProviderModel::new(provider_id, "gpt-5").expect("model");
        assert_eq!(model.provider_id, provider_id);
        assert_eq!(model.model_id, "gpt-5");
    }

    #[test]
    fn verification_metadata() {
        let metadata = locus_core::provider::ProviderVerificationMetadata::new(
            "2026-08-23T00:00:00Z", 3, locus_core::provider::VerificationStatus::Failed,
        ).expect("metadata");
        assert_eq!(metadata.model_count, 3);
        assert_eq!(metadata.status, locus_core::provider::VerificationStatus::Failed);
    }

    #[test]
    fn verifies_connection() {
        let reference = KeychainReference::new("locus/provider/openai").expect("reference");
        let provider = ProviderReference::new(Uuid::nil(), "openai", reference).expect("provider");
        let metadata = ProviderBroker::new(TestKeychain)
            .verify_connection(&provider, "2026-08-23T00:00:00Z", |_| Ok(3))
            .expect("verification");
        assert_eq!(metadata.model_count, 3);
    }

    #[test]
    fn connection_config() {
        let config = locus_core::provider::ProviderConnectionConfig::new("oauth", Some("https://api.example.test".into())).expect("config");
        assert_eq!(config.authentication_method(), "oauth");
        assert_eq!(config.base_url(), Some("https://api.example.test"));
    }

    #[test]
    fn migration_versions_are_unique() {
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations");
        let mut versions = std::collections::BTreeSet::new();
        for entry in std::fs::read_dir(directory).expect("migrations") {
            let name = entry.expect("migration entry").file_name().into_string().expect("utf8 name");
            if name.ends_with(".up.sql") {
                let version = name.split('_').next().expect("version");
                assert!(versions.insert(version.to_owned()), "duplicate migration version {version}");
            }
        }
    }

    #[test]
    fn container_has_no_secret() {
        let proxy = locus_core::sandbox::CredentialProxy::new("provider-secret", "api_key");
        let environment = proxy.container_environment("run-nonce");
        assert!(locus_core::sandbox::no_long_lived_secret(
            "provider-secret",
            &environment,
            &[]
        ));
    }

    #[test]
    fn broker_only_access() {
        let reference = KeychainReference::new("locus/provider/openai").expect("reference");
        let provider = ProviderReference::new(Uuid::nil(), "openai", reference).expect("provider");
        let broker = ProviderBroker::new(TestKeychain);
        broker
            .with_host_egress(&provider, |_| Ok(()))
            .expect("host broker");
    }

    #[test]
    fn keychain_broker() {
        let reference = KeychainReference::new("locus/provider/openai").expect("reference");
        assert_eq!(
            KeyringKeychain::entry_name(&reference),
            "locus/provider/openai"
        );
    }
}
