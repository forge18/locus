#[cfg(test)]
mod dispatch_custom_schedule_schema {
    use std::fs;

    #[test]
    fn dispatch_custom_schedule_schema() {
        let sql = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migrations/0019_m07_reconciliation.up.sql"
        ))
        .expect("read M0.7 migration");
        for field in [
            "workflow_def_id DROP NOT NULL",
            "run_mode",
            "prompt",
            "guardrail_overrides",
        ] {
            assert!(sql.contains(field), "migration contains {field}");
        }
    }
}

#[cfg(test)]
mod guardrail_defaults_schema {
    use std::fs;

    #[test]
    fn guardrail_defaults_schema() {
        let sql = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../migrations/0019_m07_reconciliation.up.sql"
        ))
        .expect("read M0.7 migration");
        for field in [
            "kill_and_reassign",
            "change_lines_ceiling",
            "change_files_ceiling",
            "network_tier",
            "block_system_changes",
            "autopilot",
        ] {
            assert!(sql.contains(field), "migration contains {field}");
        }
    }
}
