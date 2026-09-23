use actix_web::{cookie::Key, middleware};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};

pub mod static_routes;
pub mod open_routes;
pub mod api_routes;
pub mod ws_routes;
pub mod aws_routes;

lazy_static::lazy_static! {
    pub static ref SECRET_KEY: String = std::env::var("SECRET_KEY").unwrap_or_else(|_| "0123".repeat(16));
}

pub fn middleware_cookie_session() -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::new(
        CookieSessionStore::default(), 
        Key::from(SECRET_KEY.as_bytes()))
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
pub fn set_env() {
    dotenv::dotenv().ok();
    std::env::set_var(
        "RUST_LOG",
        "rigidity-application=debug,actix_web=info,actix_server=info",
    );
    env_logger::init();
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
pub fn set_env() {
    //check postgre URI
    std::env::var("POSTGRESQL_ADDON_URI").expect("Missing POSTGRESQL_ADDON_URI env variable.");
    std::env::var("DOMAIN").expect("Missing DOMAIN env variable.");
    std::env::var("EMAIL_DOMAIN").expect("Missing EMAIL_DOMAIN env variable.");
    std::env::var("EMAIL_KEY").expect("Missing EMAIL_KEY env variable.");
    std::env::var("EMAIL_DEFAULT_ADDRESS").expect("Missing EMAIL_DEFAULT_ADDRESS env variable.");
    std::env::var("MAX_NB_WORKERS").expect("Missing MAX_NB_WORKERS env variable.");
    std::env::var("MAX_DB_CONNS_WORKER").expect("Missing MAX_DB_CONNS_WORKER env variable.");
    std::env::var("AWS_ACCESS_KEY_ID").expect("Missing AWS_ACCESS_KEY_ID env variable.");
    std::env::var("AWS_SECRET_ACCESS_KEY").expect("Missing AWS_SECRET_ACCESS_KEY env variable.");
    std::env::var("SECRET_KEY").expect("Missing SECRET_KEY env variable.");
    std::env::var("STEAM_SECRET_ACCESS_KEY").expect("Missing STEAM_SECRET_ACCESS_KEY env variable");

    env_logger::init();
}

#[cfg(not(debug_assertions))]
pub fn get_listen_address() -> String {
    String::from("0.0.0.0:8080")
}

#[cfg(not(debug_assertions))]
pub fn get_base_url() -> String {
    format!("https://{}", get_domain())
}
