DROP TABLE IF EXISTS bots.routine_executions;
DROP TABLE IF EXISTS bots.routines;
DROP TABLE IF EXISTS bots.bots;
DROP SCHEMA IF EXISTS bots;
DROP INDEX IF EXISTS runs_agent_def_id_idx;
ALTER TABLE agents.runs DROP COLUMN IF EXISTS agent_def_id;
