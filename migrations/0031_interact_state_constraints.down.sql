ALTER TABLE agents.sessions
    DROP CONSTRAINT sessions_interact_state_coherence_check;

ALTER TABLE agents.sessions
    DROP COLUMN IF EXISTS repo_id;
