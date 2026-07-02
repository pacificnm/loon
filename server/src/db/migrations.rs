//! Embedded SQL migrations.

use nest_data::{Migration, SqlMigration};

/// Returns Loon database migrations in apply order.
pub fn loon_migrations() -> Vec<Box<dyn Migration>> {
    vec![Box::new(SqlMigration::new(
        "001_initial",
        include_str!("../../migrations/001_initial.sql"),
        "-- rollback 001_initial",
    ))]
}
