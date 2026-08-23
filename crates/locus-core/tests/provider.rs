mod provider {
    use anyhow::Result;
    use locus_core::provider::{KeychainReference, KeyringKeychain, OsKeychain, ProviderBroker, ProviderReference};
    use uuid::Uuid;

    struct TestKeychain;

    impl OsKeychain for TestKeychain {
        fn read_secret(&self, _: &KeychainReference) -> Result<String> { Ok("secret".into()) }
        fn write_secret(&self, _: &KeychainReference, _: &str) -> Result<()> { Ok(()) }
        fn delete_secret(&self, _: &KeychainReference) -> Result<()> { Ok(()) }
    }

    #[test]
    fn container_has_no_secret() {
        let proxy = locus_core::sandbox::CredentialProxy::new("provider-secret", "api_key");
        let environment = proxy.container_environment("run-nonce");
        assert!(locus_core::sandbox::no_long_lived_secret("provider-secret", &environment, &[]));
    }

    #[test]
    fn broker_only_access() {
        let reference = KeychainReference::new("locus/provider/openai").expect("reference");
        let provider = ProviderReference::new(Uuid::nil(), "openai", reference).expect("provider");
        let broker = ProviderBroker::new(TestKeychain);
        broker.with_host_egress(&provider, |_| Ok(())).expect("host broker");
    }

    #[test]
    fn keychain_broker() {
        let reference = KeychainReference::new("locus/provider/openai").expect("reference");
        assert_eq!(KeyringKeychain::entry_name(&reference), "locus/provider/openai");
    }
}
