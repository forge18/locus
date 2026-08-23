mod adapter {
    use locus_core::adapter::{AdapterRegistry, AdapterVersion};

    #[test]
    fn registry() {
        let registry = AdapterRegistry::from([AdapterVersion::new("claude-acp", "3").expect("adapter")]);
        assert!(registry.contains("claude-acp", "3"));
    }
}
