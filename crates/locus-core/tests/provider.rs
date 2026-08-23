mod provider {
    use locus_core::provider::{KeychainReference, KeyringKeychain};

    #[test]
    fn keychain_broker() {
        let reference = KeychainReference::new("locus/provider/openai").expect("reference");
        assert_eq!(KeyringKeychain::entry_name(&reference), "locus/provider/openai");
    }
}
