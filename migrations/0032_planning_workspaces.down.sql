DROP TABLE IF EXISTS core.planning_workspace_materializations;
DROP TABLE IF EXISTS core.planning_workspace_events;
DROP TABLE IF EXISTS core.planning_workspace_sessions;
DROP TABLE IF EXISTS core.planning_workspace_specs;
ALTER TABLE core.planning_workspaces
    DROP CONSTRAINT IF EXISTS planning_workspaces_approved_revision_fk;
DROP TABLE IF EXISTS core.planning_workspace_revisions;
DROP TABLE IF EXISTS core.planning_workspaces;
