use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use actix_web::{middleware};
use actix_identity::{CookieIdentityPolicy, IdentityService};
use time::Duration;
use super::{Pool};

pub mod static_routes;
pub mod open_routes;
pub mod api_routes;

lazy_static::lazy_static! {
    pub static ref SECRET_KEY: String = std::env::var("SECRET_KEY").unwrap_or_else(|_| "0123".repeat(8));
}

#[cfg(debug_assertions)]
pub fn nb_worker() -> Option<u32> {
    None
}

#[cfg(debug_assertions)]
pub fn middleware_logger() -> middleware::Logger {
    middleware::Logger::default()
}

#[cfg(debug_assertions)]
pub fn middleware_identity_service() -> IdentityService<CookieIdentityPolicy> {
    IdentityService::new(
        CookieIdentityPolicy::new(SECRET_KEY.as_bytes())
            .name("auth")
            .path("/api")
            .domain(get_domain().as_str())
            .max_age_time(Duration::days(1))
            .secure(false), // this can only be true if you have https
    )
}

#[cfg(debug_assertions)]
pub fn set_env() {
    dotenv::dotenv().ok();
    std::env::set_var(
        "RUST_LOG",
        "rigidity-application=debug,actix_web=info,actix_server=info",
    );
    env_logger::init();
}

#[cfg(debug_assertions)]
pub fn connect_database() -> Pool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // create db connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

#[cfg(debug_assertions)]
pub fn get_listen_address() -> String {
    let mut domain = get_domain();
    domain.push_str(":8080");

    domain
}

#[cfg(debug_assertions)]
pub fn get_base_url() -> String {
    format!("http://{}", get_listen_address())
}

fn get_domain() -> String {
    std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string())
}

#[cfg(not(debug_assertions))]
pub fn nb_worker() -> Option<u32> {
    let max_nb_workers: u32 = std::env::var("MAX_NB_WORKERS")
        .expect("MAX_NB_WORKER must be set")
        .parse()
        .unwrap();
    Some(max_nb_workers)
}

#[cfg(not(debug_assertions))]
pub fn middleware_logger() -> middleware::Logger {
    middleware::Logger::default()
}

#[cfg(not(debug_assertions))]
pub fn middleware_identity_service() -> IdentityService<CookieIdentityPolicy> {
    IdentityService::new(
        CookieIdentityPolicy::new(SECRET_KEY.as_bytes())
            .name("auth")
            .path("/api")
            .domain(get_domain().as_str())
            .max_age_time(Duration::days(1))
            .secure(true), // this can only be true if you have https
    )
}

#[cfg(not(debug_assertions))]
pub fn set_env() {
    //check postgre URI
    std::env::var("POSTGRESQL_ADDON_URI").expect("Missing POSTGRESQL_ADDON_URI env variable.");
    std::env::var("DOMAIN").expect("Missing DOMAIN env variable.");
    std::env::var("MAILGUN_DOMAIN").expect("Missing MAILGUN_DOMAIN env variable.");
    std::env::var("MAILGUN_KEY").expect("Missing MAILGUN_KEY env variable.");
    std::env::var("MAILGUN_MAIL_ADDRESS").expect("Missing MAILGUN_MAIL_ADDRESS env variable.");
    std::env::var("MAX_NB_WORKERS").expect("Missing MAX_NB_WORKERS env variable.");
    std::env::var("MAX_DB_CONNS_WORKER").expect("Missing MAX_DB_CONNS_WORKER env variable.");
    std::env::var("AWS_ACCESS_KEY_ID").expect("Missing AWS_ACCESS_KEY_ID env variable.");
    std::env::var("AWS_SECRET_ACCESS_KEY").expect("Missing AWS_SECRET_ACCESS_KEY env variable.");

    env_logger::init();
}

#[cfg(not(debug_assertions))]
pub fn connect_database() -> Pool {
    let database_url = std::env::var("POSTGRESQL_ADDON_URI").expect("POSTGRESQL_ADDON_URI must be set");
    let max_size: u32 = std::env::var("MAX_DB_CONNS_WORKER")
        .expect("MAX_DB_CONNS_WORKER must be set")
        .parse()
        .unwrap();
    // create db connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .max_size(max_size)
        .build(manager)
        .expect("Failed to create pool.")
}

#[cfg(not(debug_assertions))]
pub fn get_listen_address() -> String {
    String::from("0.0.0.0:8080")
}

#[cfg(not(debug_assertions))]
pub fn get_base_url() -> String {
    format!("https://{}", get_domain())
}
