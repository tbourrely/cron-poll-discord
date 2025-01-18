use lazy_static::lazy_static;
use rusqlite_migration::Migrations;
use include_dir::{include_dir, Dir};
use rusqlite::Connection;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

// Define migrations. These are applied atomically.
lazy_static! {
    static ref MIGRATIONS: Migrations<'static> =
        Migrations::from_directory(&MIGRATIONS_DIR).unwrap();
}

pub fn init_db() -> Result<Connection, Box<dyn std::error::Error>> {
    let mut conn = Connection::open(crate::DATABASE)?;

    // Update the database schema, atomically
    MIGRATIONS.to_latest(&mut conn)?;

    Ok(conn)
}
