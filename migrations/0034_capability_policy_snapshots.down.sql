ALTER TABLE agents.runs
DROP COLUMN IF EXISTS capability_snapshot,
DROP COLUMN IF EXISTS capability_policy_revision;
DROP TABLE IF EXISTS core.project_capability_policies;
