DROP TABLE core.stop_all_schedule_snapshots;
DROP TABLE core.stop_all_autorun_snapshots;
DROP TABLE core.stop_all_run_snapshots;
DROP TABLE core.stop_all_snapshots;
DROP TABLE core.project_autorun;

ALTER TABLE agents.runs DROP CONSTRAINT runs_status_check;
ALTER TABLE agents.runs ADD CONSTRAINT runs_status_check CHECK (
    status IN ('queued', 'running', 'paused', 'completed', 'aborted', 'cancelled')
);
