ALTER TABLE agents.sessions
    ADD COLUMN IF NOT EXISTS repo_id UUID REFERENCES core.repos (id) ON DELETE SET NULL;

ALTER TABLE agents.sessions
ADD constraint sessions_interact_state_coherence_check
CHECK (
    (interact_state = 'open' AND board_task_id IS NULL)
    OR (interact_state = 'promoted' AND board_task_id IS NOT NULL)
    OR (interact_state = 'discarded' AND board_task_id IS NULL)
);
