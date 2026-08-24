ALTER TABLE workflows.schedules DROP CONSTRAINT IF EXISTS schedules_custom_shape_check;
ALTER TABLE agents.runs DROP CONSTRAINT IF EXISTS runs_routing_decision_check;
ALTER TABLE agents.runs
    ADD CONSTRAINT runs_routing_decision_check CHECK (
        (
            routing_requested_band IS NULL
            AND routing_selected_band IS NULL
            AND routing_effort IS NULL
            AND routing_approval_required IS NULL
        )
        OR (
            routing_requested_band IN (
                'xtra-low', 'low', 'medium', 'high', 'xtra-high', 'max'
            )
            AND (
                routing_selected_band IS NULL
                OR routing_selected_band IN (
                    'xtra-low', 'low', 'medium', 'high', 'xtra-high', 'max'
                )
            )
            AND routing_effort IS NOT NULL
            AND btrim(routing_effort) <> ''
            AND routing_approval_required IS NOT NULL
        )
    );
DROP TABLE IF EXISTS core.plan_stage_history;
DROP TABLE IF EXISTS core.plan_requirements;
DROP TABLE IF EXISTS core.plans;
DROP TABLE IF EXISTS core.harness_adapter_configs;
ALTER TABLE core.providers DROP COLUMN IF EXISTS verification_expires_at, DROP COLUMN IF EXISTS verification_stale_after;
ALTER TABLE workflows.schedules
    DROP COLUMN IF EXISTS guardrail_overrides, DROP COLUMN IF EXISTS prompt, DROP COLUMN IF EXISTS spec_id,
    DROP COLUMN IF EXISTS project_id, DROP COLUMN IF EXISTS harness, DROP COLUMN IF EXISTS agent_def_id,
    DROP COLUMN IF EXISTS run_mode;
ALTER TABLE workflows.schedules
    ALTER COLUMN workflow_def_id SET NOT NULL;
DROP TABLE IF EXISTS memory.retrieval_feedback;
DROP TABLE IF EXISTS memory.fact_revisions;
ALTER TABLE memory.store DROP COLUMN IF EXISTS current_revision, DROP COLUMN IF EXISTS confidence_state;
ALTER TABLE agents.runs DROP COLUMN IF EXISTS resolved_guardrails, DROP COLUMN IF EXISTS verify_status, DROP COLUMN IF EXISTS failed_iterations;
DROP INDEX IF EXISTS sessions_interact_state_idx;
ALTER TABLE agents.sessions DROP COLUMN IF EXISTS interact_state;
DROP TABLE IF EXISTS core.qa_findings;
DROP TABLE IF EXISTS core.qa_check_runs;
DROP TABLE IF EXISTS core.qa_schedules;
DROP TABLE IF EXISTS core.guardrail_defaults;
DROP TABLE IF EXISTS core.project_autorun_policy;
DROP TABLE IF EXISTS core.stop_all_handoffs;
ALTER TABLE core.stop_all_autorun_snapshots DROP COLUMN IF EXISTS state;
ALTER TABLE core.project_autorun DROP COLUMN IF EXISTS state;
ALTER TABLE core.projects DROP COLUMN IF EXISTS archived_at;
DROP TABLE IF EXISTS core.repo_project_history;
ALTER TABLE core.repos DROP CONSTRAINT IF EXISTS repos_working_copy_path_global_key;
