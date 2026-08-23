mod routing {
    use std::collections::BTreeMap;
    use locus_core::routing::{AutoroutingPolicy, ComplexityBand, RoutingBand};

    #[test]
    fn six_bands() {
        let bands = [ComplexityBand::XtraLow, ComplexityBand::Low, ComplexityBand::Medium, ComplexityBand::High, ComplexityBand::XtraHigh, ComplexityBand::Max]
            .into_iter().map(|band| (band, RoutingBand { model_id: Some("model".into()), effort: "medium".into(), approval_required: false, when_to_use: "work".into() })).collect::<BTreeMap<_, _>>();
        assert_eq!(AutoroutingPolicy { enabled: true, bands }.bands.len(), 6);
    }
}
