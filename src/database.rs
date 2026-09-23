use sqlx::postgres::PgPoolOptions;
use crate::app_conf::config;

pub mod custom_rooms;
pub mod custom_room_slots;
pub mod users;

pub type DbPool = sqlx::PgPool;
pub type DbResult<R> = Result<R, sqlx::Error>;

// Any query added/changed in this module's submodules requires re-running
// `cargo sqlx prepare` (with DATABASE_URL pointed at a migrated database)
// and committing the resulting `.sqlx/*.json` files.

pub async fn connect_database() -> DbPool {
    let config = config();
    let mut options = PgPoolOptions::new();
    if let Some(max_size) = config.max_db_conns_worker {
        options = options.max_connections(max_size);
    }

    options
        .connect(&config.database_url)
        .await
        .expect("Failed to create sqlx pool.")
}
