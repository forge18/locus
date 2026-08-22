DROP TABLE agents.dispatch_preemptions;

ALTER TABLE core.dispatch_policy
DROP COLUMN preemption_enabled;
