DROP TABLE IF EXISTS agents.permission_gates;
DROP TABLE IF EXISTS agents.checkpoints;
ALTER TABLE agents.runs DROP COLUMN IF EXISTS permission_posture;
