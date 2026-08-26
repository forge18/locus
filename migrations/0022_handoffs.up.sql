ALTER TABLE agents.sessions
ADD COLUMN handed_off_from UUID REFERENCES agents.sessions (id);

CREATE INDEX sessions_handed_off_from_idx
ON agents.sessions (handed_off_from);
