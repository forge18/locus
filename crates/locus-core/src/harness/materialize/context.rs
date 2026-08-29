//! The context assembly policy shared by every harness materializer.
//!
//! This is a derivation table, not a second memory store. Knowledge kind and lifetime
//! describe the source; task class selects recall depth where that matters. Constitution
//! and rules remain in the host-owned never-drop plane.

use crate::services::memory::{EvictionClass, TaskClass};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum KnowledgeKind {
    Constitution,
    Rule,
    BaseContext,
    Skill,
    Memory,
    Plan,
    Evidence,
    ToolOutput,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Lifetime {
    Working,
    Probation,
    LongTerm,
    Written,
    Run,
    Turn,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PlacementZone {
    FrozenHead,
    MutableTail,
    OnDemand,
    ToolBoundary,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum InjectionMechanism {
    AlwaysOn,
    Catalog,
    Recall,
    Recitation,
    ArtifactHandle,
    Compaction,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ContextDerivation {
    pub placement: PlacementZone,
    pub injection: InjectionMechanism,
    pub eviction_class: EvictionClass,
}

impl ContextDerivation {
    pub const fn never_dropped(self) -> bool {
        matches!(
            (self.placement, self.eviction_class),
            (PlacementZone::FrozenHead, EvictionClass::Sticky)
        )
    }
}

/// Short aliases keep call sites readable without introducing a second taxonomy.
pub type KnowledgeLifetime = Lifetime;
pub type ContextPolicy = ContextDerivation;

/// Pure policy derivation. No filesystem, clock, harness name, or model call can
/// affect the result.
pub const fn derive_context(
    kind: KnowledgeKind,
    lifetime: Lifetime,
    task_class: TaskClass,
) -> ContextDerivation {
    match kind {
        KnowledgeKind::Constitution | KnowledgeKind::Rule | KnowledgeKind::BaseContext => {
            ContextDerivation {
                placement: PlacementZone::FrozenHead,
                injection: InjectionMechanism::AlwaysOn,
                eviction_class: EvictionClass::Sticky,
            }
        }
        KnowledgeKind::Skill => match lifetime {
            Lifetime::Written | Lifetime::LongTerm => ContextDerivation {
                placement: PlacementZone::OnDemand,
                injection: InjectionMechanism::ArtifactHandle,
                eviction_class: EvictionClass::Standard,
            },
            _ => ContextDerivation {
                placement: PlacementZone::FrozenHead,
                injection: InjectionMechanism::AlwaysOn,
                eviction_class: EvictionClass::Sticky,
            },
        },
        KnowledgeKind::Memory => match lifetime {
            Lifetime::Working | Lifetime::Probation => ContextDerivation {
                placement: PlacementZone::MutableTail,
                injection: InjectionMechanism::Recall,
                eviction_class: EvictionClass::Standard,
            },
            Lifetime::Written => ContextDerivation {
                placement: PlacementZone::OnDemand,
                injection: InjectionMechanism::ArtifactHandle,
                eviction_class: EvictionClass::Standard,
            },
            Lifetime::LongTerm => ContextDerivation {
                placement: PlacementZone::FrozenHead,
                injection: InjectionMechanism::Catalog,
                eviction_class: EvictionClass::Standard,
            },
            Lifetime::Run | Lifetime::Turn => ContextDerivation {
                placement: PlacementZone::MutableTail,
                injection: if matches!(task_class, TaskClass::Research) {
                    InjectionMechanism::Recall
                } else {
                    InjectionMechanism::Catalog
                },
                eviction_class: EvictionClass::Standard,
            },
        },
        KnowledgeKind::Plan => ContextDerivation {
            placement: PlacementZone::MutableTail,
            injection: InjectionMechanism::Recitation,
            eviction_class: EvictionClass::Sticky,
        },
        KnowledgeKind::Evidence => ContextDerivation {
            placement: PlacementZone::OnDemand,
            injection: InjectionMechanism::Recall,
            eviction_class: EvictionClass::Standard,
        },
        KnowledgeKind::ToolOutput => ContextDerivation {
            placement: PlacementZone::ToolBoundary,
            injection: InjectionMechanism::Compaction,
            eviction_class: EvictionClass::Standard,
        },
    }
}

pub const fn derive_policy(
    kind: KnowledgeKind,
    lifetime: Lifetime,
    task_class: TaskClass,
) -> ContextDerivation {
    derive_context(kind, lifetime, task_class)
}

/// Stable assembly of named fragments. The caller can put the returned head before
/// any mutable tail; sorting here makes the same policy byte-identical across runs.
pub fn assemble_frozen_head<'a>(fragments: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
    let mut fragments = fragments.into_iter().collect::<Vec<_>>();
    fragments.sort_by(|left, right| left.0.cmp(right.0));
    fragments
        .into_iter()
        .map(|(_, body)| body)
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod materializer {
    use super::*;

    #[test]
    fn context_derivation_table() {
        let inputs = [
            (
                KnowledgeKind::Constitution,
                Lifetime::Written,
                TaskClass::Code,
            ),
            (KnowledgeKind::Memory, Lifetime::LongTerm, TaskClass::Code),
            (KnowledgeKind::Memory, Lifetime::Run, TaskClass::Research),
            (KnowledgeKind::ToolOutput, Lifetime::Turn, TaskClass::Code),
        ];
        let first = inputs
            .iter()
            .map(|(kind, lifetime, task)| derive_context(*kind, *lifetime, *task))
            .collect::<Vec<_>>();
        let second = inputs
            .iter()
            .map(|(kind, lifetime, task)| derive_context(*kind, *lifetime, *task))
            .collect::<Vec<_>>();
        assert_eq!(first, second);
        assert_eq!(first[1].injection, InjectionMechanism::Catalog);
        assert_eq!(first[2].injection, InjectionMechanism::Recall);
    }

    #[test]
    fn derivation_keeps_constitution_head() {
        for kind in [KnowledgeKind::Constitution, KnowledgeKind::Rule] {
            let policy = derive_context(kind, Lifetime::Written, TaskClass::Research);
            assert_eq!(policy.placement, PlacementZone::FrozenHead);
            assert_eq!(policy.injection, InjectionMechanism::AlwaysOn);
            assert_eq!(policy.eviction_class, EvictionClass::Sticky);
            assert!(policy.never_dropped());
        }
        assert_eq!(
            assemble_frozen_head([("z", "rules"), ("a", "constitution")]),
            "constitution\n\nrules"
        );
    }
}
