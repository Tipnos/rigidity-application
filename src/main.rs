use axum::{routing::get, Router};
use rigidity_application::{
    cmd::interpret_args,
    services::aws::get_gamelift_client,
    app_conf,
    database,
    new_websocket_lobby,
    AppState};
use tower_http::trace::TraceLayer;
use std::env;

fn main() -> std::io::Result<()> {
    app_conf::set_env();

    let mut runtime = tokio::runtime::Builder::new_multi_thread();
    if let Some(max_nb_workers) = app_conf::nb_worker() {
        runtime.worker_threads(max_nb_workers as usize);
    }
    let runtime = runtime.enable_all().build()?;

    runtime.block_on(async {
        let mut args: Vec<String> = env::args().collect();

        if args.len() == 1 {
            return start_server().await
        } else {
            args.drain(0..1);
            interpret_args(args).await;
        }

        Ok(())
    })
}

async fn start_server() -> std::io::Result<()> {
    let db_pool = database::connect_database().await;
    let state = AppState {
        ws: new_websocket_lobby(db_pool.clone()),
        pool: db_pool,
        gamelift: get_gamelift_client().await,
        cookie_key: app_conf::cookie_key(),
    };

    let app = Router::new()
        .route("/ws", get(app_conf::ws_routes::get()))
        .merge(app_conf::open_routes::get_all())
        .merge(app_conf::api_routes::get_all())
        .merge(app_conf::aws_routes::get_all())
        .merge(app_conf::static_routes::get_all())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(app_conf::get_listen_address()).await?;
    axum::serve(listener, app).await
}
