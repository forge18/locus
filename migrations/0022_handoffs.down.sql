DROP INDEX IF EXISTS agents.sessions_handed_off_from_idx;
ALTER TABLE agents.sessions DROP COLUMN IF EXISTS handed_off_from;
