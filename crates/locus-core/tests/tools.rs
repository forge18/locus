mod tools {
    #[test]
    fn trusted_keys() {
        let keys = locus_core::tools::TrustedKeyStore::from_public_keys([]).expect("empty trust store");
        assert!(std::mem::size_of_val(&keys) > 0);
    }
}
