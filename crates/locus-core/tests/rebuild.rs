mod rebuild {
    use locus_core::ids::{ProjectId, RunId};
    use locus_core::services::wiki::{
        PageKind, RevisionActor, WikiEmbedding, WikiEvent, WikiPage, WikiProjection, WikiRevision,
    };

    #[test]
    fn wiki_attribution_survives() {
        let run_id = RunId::generate();
        let page = WikiPage {
            id: "page".into(),
            project_id: ProjectId::generate(),
            slug: "page".into(),
            kind: PageKind::Source,
            title: "README".into(),
            body: String::new(),
            revision: 0,
            links_out: vec![],
            provenance: vec![],
            assertion_count: 0,
            source_count: 0,
        };
        let projection = WikiProjection::from_events([
            WikiEvent::PageCreated { page },
            WikiEvent::RevisionAdded {
                revision: WikiRevision {
                    id: "revision".into(),
                    page_id: "page".into(),
                    number: 1,
                    body: "agent edit".into(),
                    summary: "ingest".into(),
                    actor: RevisionActor::Agent { run_id },
                },
            },
        ])
        .unwrap();
        assert!(matches!(
            projection.revisions[0].actor,
            RevisionActor::Agent { run_id: seen } if seen == run_id
        ));
    }

    #[test]
    fn wiki_vectors_untouched() {
        let embedding = WikiEmbedding {
            id: "embedding".into(),
            project_id: ProjectId::generate(),
            revision_id: "revision".into(),
            source_page_id: "page".into(),
            statement: "statement".into(),
            vector: vec![0.25, 0.5, 0.75],
            model: "test".into(),
            carve_out: true,
        };
        let projection = WikiProjection::from_events([WikiEvent::EmbeddingAdded {
            embedding: embedding.clone(),
        }])
        .unwrap();
        assert_eq!(projection.embeddings[0].vector, embedding.vector);
        assert!(projection.embeddings[0].carve_out);
    }
}
