mod routing {
    use std::collections::BTreeMap;
    use locus_core::routing::{AutoroutingPolicy, ComplexityBand, RoutingBand};

    #[test]
    fn falls_up() {
        let mut bands = BTreeMap::new();
        bands.insert(ComplexityBand::High, RoutingBand { model_id: Some("opus".into()), effort: "high".into(), approval_required: false, when_to_use: "hard".into() });
        let decision = AutoroutingPolicy { enabled: true, bands }.route(ComplexityBand::Medium, &locus_core::routing::RoutingDefaults { model_id: "sonnet".into(), effort: "medium".into() }).expect("fallback");
        assert_eq!(decision.selected_band, Some(ComplexityBand::High));
    }

    #[test]
    fn six_bands() {
        let bands = [ComplexityBand::XtraLow, ComplexityBand::Low, ComplexityBand::Medium, ComplexityBand::High, ComplexityBand::XtraHigh, ComplexityBand::Max]
            .into_iter().map(|band| (band, RoutingBand { model_id: Some("model".into()), effort: "medium".into(), approval_required: false, when_to_use: "work".into() })).collect::<BTreeMap<_, _>>();
        assert_eq!(AutoroutingPolicy { enabled: true, bands }.bands.len(), 6);
    }
}
