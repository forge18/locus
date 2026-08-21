# calibration-loop — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Retro agent definition with a bounded job | — | `cargo test -p locus-core calibrate::retro_agent` |
| 2 | Watermark: read only classifications since the last pass | 1 | `cargo test -p locus-core calibrate::watermark` |
| 3 | Cluster recurring classes across tasks | 2 | `cargo test -p locus-core calibrate::clusters` |
| 4 | One proposal per recurring cluster, not one per task | 3 | `cargo test -p locus-core calibrate::one_proposal_per_cluster` |
| 5 | Bug → regression-set proposal | 3 | `cargo test -p locus-core calibrate::bug_proposal` |
| 6 | Spec gap → specialization-record clause proposal | 3 | `cargo test -p locus-core calibrate::spec_gap_proposal` |
| 7 | Noise → recalibrate or quarantine proposal | 3 | `cargo test -p locus-core calibrate::noise_proposal` |
| 8 | Ambiguity → interview topic or reduction rule proposal | 3 | `cargo test -p locus-core calibrate::ambiguity_proposal` |
| 9 | Assert no fifth proposal type can be produced | 5,6,7,8 | `cargo test -p locus-core calibrate::exactly_four_types` |
| 10 | Reflection review queue holding every proposal | 4 | `cargo test -p locus-core calibrate::review_queue` |
| 11 | Assert no proposal applies without acceptance, per type | 10 | `cargo test -p locus-core calibrate::nothing_auto_applies` |
| 12 | Accept a bug proposal into the regression set | 5,10 | `cargo test -p locus-core calibrate::accept_bug` |
| 13 | Accept a spec-gap proposal onto a wiki `concept` page | 6,10 | `cargo test -p locus-core calibrate::accept_spec_gap` |
| 14 | Assert specialization records are wiki pages, not a new store | 13 | `cargo test -p locus-core calibrate::no_fourth_tier` |
| 15 | Accept a noise proposal, recalibrating the check | 7,10 | `cargo test -p locus-core calibrate::accept_noise` |
| 16 | Accept an ambiguity proposal into the interview or reduction pass | 8,10 | `cargo test -p locus-core calibrate::accept_ambiguity` |
| 17 | Record a rejection so it is not re-proposed identically | 10 | `cargo test -p locus-core calibrate::rejection_sticks` |
| 18 | Confidence threshold gating record injection into synthesis | 13 | `cargo test -p locus-core calibrate::threshold_gates_injection` |
| 19 | Below-threshold record is not injected | 18 | `cargo test -p locus-core calibrate::below_threshold_not_injected` |
| 20 | Review queue UI, shared with memory promotions | 10 | `pnpm -C apps/desktop test -- inbox/reflection-queue` |
