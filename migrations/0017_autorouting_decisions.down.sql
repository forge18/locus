ALTER TABLE agents.runs
DROP CONSTRAINT runs_routing_decision_check,
DROP COLUMN routing_approval_required,
DROP COLUMN routing_effort,
DROP COLUMN routing_selected_band,
DROP COLUMN routing_requested_band;
