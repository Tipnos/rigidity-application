use sqlx::postgres::PgPoolOptions;

pub mod users;

pub type DbPool = sqlx::PgPool;
pub type DbResult<R> = Result<R, sqlx::Error>;

// Any query added/changed in this module's submodules requires re-running
// `cargo sqlx prepare` (with DATABASE_URL pointed at a migrated database)
// and committing the resulting `.sqlx/*.json` files.

#[cfg(debug_assertions)]
pub async fn connect_database() -> DbPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to create sqlx pool.")
}

#[cfg(not(debug_assertions))]
pub async fn connect_database() -> DbPool {
    let database_url = std::env::var("POSTGRESQL_ADDON_URI").expect("POSTGRESQL_ADDON_URI must be set");
    let max_size: u32 = std::env::var("MAX_DB_CONNS_WORKER")
        .expect("MAX_DB_CONNS_WORKER must be set")
        .parse()
        .unwrap();

    PgPoolOptions::new()
        .max_connections(max_size)
        .connect(&database_url)
        .await
        .expect("Failed to create sqlx pool.")
}
