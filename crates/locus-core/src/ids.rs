//! Typed identifiers.
//!
//! Every id was a bare `Uuid` or `String`, so passing a session id where a run id belongs
//! compiled silently. That matters most on the nested-invocation path, which threads a
//! parent run id, a child run id, and a session id through the same signatures — and
//! `.specs/agent-cli` bounds that at depth 3 / fan-out 4 because depth 4 is 85 containers.
//!
//! `run_id` alone had five production spellings before this: `Uuid`, `&str`, `String`,
//! `impl Into<String>`, and `u128`, with `to_string()` and `Uuid::from_u128` bridging
//! between them.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! typed_id {
    ($name:ident, $what:literal) => {
        #[doc = concat!("Identifies one ", $what, ".")]
        #[derive(
            Clone,
            Copy,
            Debug,
            Default,
            Deserialize,
            Eq,
            Hash,
            Ord,
            PartialEq,
            PartialOrd,
            Serialize,
            sqlx::Type,
        )]
        #[serde(transparent)]
        #[sqlx(transparent)]
        pub struct $name(Uuid);

        impl $name {
            pub const fn new(value: Uuid) -> Self {
                Self(value)
            }

            /// A fresh identifier. The only place one is minted.
            pub fn generate() -> Self {
                Self(Uuid::new_v4())
            }

            pub const fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self(value)
            }
        }

        impl From<$name> for Uuid {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                value.parse().map(Self)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

typed_id!(ProjectId, "project");
typed_id!(SessionId, "agent session");
typed_id!(RunId, "run: one container lifetime inside a session");
typed_id!(TurnId, "turn: one prompt and its response within a run");
typed_id!(TaskId, "board task");
typed_id!(ArtifactId, "run artifact");
typed_id!(AgentDefId, "agent definition");
typed_id!(EventId, "persisted event");
typed_id!(CommentId, "artifact comment");
