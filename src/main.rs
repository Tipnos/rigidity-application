use axum::routing::get;
use rigidity_application::{
    cmd,
    app_conf,
    database,
    new_websocket_lobby,
    AppState};
use tower_http::trace::TraceLayer;
use utoipa_swagger_ui::SwaggerUi;

fn main() -> std::io::Result<()> {
    let config = app_conf::init();

    let mut runtime = tokio::runtime::Builder::new_multi_thread();
    if let Some(max_nb_workers) = config.max_nb_workers {
        runtime.worker_threads(max_nb_workers);
    }
    let runtime = runtime.enable_all().build()?;

    runtime.block_on(async {
        match &config.command {
            Some(command) => {
                cmd::run(command).await;
                Ok(())
            },
            None => start_server().await,
        }
    })
}

async fn start_server() -> std::io::Result<()> {
    let db_pool = database::connect_database().await;
    let state = AppState {
        ws: new_websocket_lobby(db_pool.clone()),
        pool: db_pool,
        // TODO: a GameLift client was built here from the AWS credentials
        // (region eu-west-1) and shared through `AppState`.
        cookie_key: app_conf::cookie_key(),
    };

    let (api_router, api) = app_conf::openapi::router().split_for_parts();
    let mut app = api_router
        .route("/ws", get(app_conf::ws_routes::get()))
        // TODO: `POST /aws/sns` was merged here. It received AWS SNS
        // notifications (subscription confirmation plus FlexMatch events) and
        // passed them on to `custom_room::matchmaking_succeeded` /
        // `custom_room::matchmaking_failed`.
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api));

    if app_conf::config().dev_login {
        tracing::warn!("Dev login enabled: anyone can log in as any user via POST /api-open/dev-login");
        app = app.merge(app_conf::dev_routes::get_all());
    }

    let app = app
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&app_conf::config().listen_address).await?;
    axum::serve(listener, app).await
}
