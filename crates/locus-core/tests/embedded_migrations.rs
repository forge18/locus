//! The migrations a deployed binary carries.
//!
//! `sqlx::migrate!` embeds `migrations/` at compile time, so this test failing to compile
//! is the point: a malformed or missing migration is a build error, not a boot error.

#[test]
fn every_migration_on_disk_is_embedded() {
    let embedded = sqlx::migrate!("../../migrations");
    let on_disk = std::fs::read_dir(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
    )
    .expect("read migrations directory")
    .filter_map(Result::ok)
    .filter(|entry| entry.file_name().to_string_lossy().ends_with(".up.sql"))
    .count();

    // `migrate!` embeds both directions of a reversible pair; count the up half.
    let embedded_up = embedded
        .migrations
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .count();

    assert_eq!(
        embedded_up, on_disk,
        "a migration on disk is not embedded in the binary"
    );
    assert!(on_disk > 0, "no migrations found");
}
