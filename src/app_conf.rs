use axum_extra::extract::cookie::Key;
use clap::Parser;
use std::sync::OnceLock;
use crate::cmd::Command;

pub mod dev_routes;
pub mod open_routes;
pub mod api_routes;
pub mod ws_routes;
pub mod aws_routes;
pub mod openapi;

/// Application configuration. Every option can be given as a CLI argument or
/// through its environment variable. Defaults target local development.
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Config {
    #[arg(long, env = "DATABASE_URL",
        default_value = "postgres://postgres:password@localhost/postgres?sslmode=disable")]
    pub database_url: String,

    /// Max database connections per worker (sqlx default if unset)
    #[arg(long, env = "MAX_DB_CONNS_WORKER")]
    pub max_db_conns_worker: Option<u32>,

    /// Number of tokio worker threads (one per CPU core if unset)
    #[arg(long, env = "MAX_NB_WORKERS")]
    pub max_nb_workers: Option<usize>,

    #[arg(long, env = "LISTEN_ADDRESS", default_value = "127.0.0.1:8080")]
    pub listen_address: String,

    /// Enables POST /api-open/dev-login to log in as any user without Steam.
    /// Never enable in production.
    #[arg(long, env = "DEV_LOGIN", default_value_t = false)]
    pub dev_login: bool,

    /// Tracing filter directives
    #[arg(long, env = "RUST_LOG", default_value = "rigidity_application=debug,tower_http=debug")]
    pub log_filter: String,

    #[arg(long, env = "SECRET_KEY", hide_env_values = true)]
    pub secret_key: String,

    #[arg(long, env = "STEAM_SECRET_ACCESS_KEY", hide_env_values = true)]
    pub steam_secret_access_key: String,

    #[arg(long, env = "AWS_ACCESS_KEY_ID", hide_env_values = true)]
    pub aws_access_key_id: String,

    #[arg(long, env = "AWS_SECRET_ACCESS_KEY", hide_env_values = true)]
    pub aws_secret_access_key: String,

    #[command(subcommand)]
    pub command: Option<Command>,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

/// Parses the configuration from CLI args and env variables, then sets up tracing.
/// Must be called once at startup, before any call to `config()`.
pub fn init() -> &'static Config {
    let config = CONFIG.get_or_init(Config::parse);

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(&config.log_filter))
        .init();

    config
}

pub fn config() -> &'static Config {
    CONFIG.get().expect("app_conf::init() must be called before app_conf::config()")
}

pub fn cookie_key() -> Key {
    Key::from(config().secret_key.as_bytes())
}
