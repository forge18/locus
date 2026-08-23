mod tools {
    #[test]
    fn group_toggles() {
        let state = locus_core::services::tools::ToolGroupEnablement::from_tools([true, false]);
        assert_eq!(
            state,
            locus_core::services::tools::ToolGroupEnablement::Mixed
        );
    }

    #[test]
    fn trusted_keys() {
        let keys = locus_core::services::tools::TrustedKeyStore::from_public_keys([])
            .expect("empty trust store");
        assert!(std::mem::size_of_val(&keys) > 0);
    }
}
