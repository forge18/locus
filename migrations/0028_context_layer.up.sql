-- Context-layer memory policy. The label is foldable metadata; vectors and decay state remain
-- the carve-outs documented by the memory contract.
ALTER TABLE memory.store
    ADD COLUMN IF NOT EXISTS eviction_class TEXT NOT NULL DEFAULT 'standard'
        CHECK (eviction_class IN ('sticky', 'standard'));

ALTER TABLE memory.probation
    ADD COLUMN IF NOT EXISTS eviction_class TEXT NOT NULL DEFAULT 'standard'
        CHECK (eviction_class IN ('sticky', 'standard'));

COMMENT ON COLUMN memory.store.eviction_class IS
    'Foldable eviction policy: sticky records survive standard overflow; standard records may be dropped by strength.';
COMMENT ON COLUMN memory.probation.eviction_class IS
    'Derived from capture origin; declared promotions and unresolved errors are sticky.';
