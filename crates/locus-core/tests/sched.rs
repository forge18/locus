mod sched {
    use locus_core::services::schedule::CronExpression;

    #[test]
    fn table() {
        let migration = include_str!("../../../migrations/0006_workflows_schema.up.sql");
        assert!(migration.contains("CREATE TABLE workflows.schedules"));
        assert!(migration.contains("cron_expression TEXT NOT NULL"));
        CronExpression::parse("0 2 * * *").expect("schedule table accepts cron syntax");
    }
}
