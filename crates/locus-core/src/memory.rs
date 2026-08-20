//! Two tiers: a bounded `core` injected at run start, and a `store` recalled on demand.
//!
//! An over-cap write to the core tier returns an error rather than evicting silently —
//! the agent consolidates in the same turn. The core block is frozen at run start to
//! preserve the model's prefix cache.
