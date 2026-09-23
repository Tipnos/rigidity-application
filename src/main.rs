use actix_web::{web, HttpResponse, web::Data, App, HttpServer};
use rigidity_application::{
    cmd::interpret_args,
    services::aws::get_gamelift_client,
    app_conf,
    database,
    new_websocket_lobby};
use actix_identity::IdentityMiddleware;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    app_conf::set_env();
    let mut args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        return start_server().await
    } else {
        args.drain(0..1);
        interpret_args(args).await;
    }

    Ok(())
}

async fn start_server() -> std::io::Result<()> {
    let db_pool = database::connect_database().await;
    let ws_srv = new_websocket_lobby(db_pool.clone()); //important if clone in closure ref not properly tracked
    let gamelift = get_gamelift_client().await;

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(gamelift.to_owned()))
            .app_data(Data::new(db_pool.clone()))
            .app_data(Data::new(ws_srv.clone()))
            .wrap(IdentityMiddleware::default())
            .wrap(app_conf::middleware_cookie_session())
            .wrap(app_conf::middleware_logger())
            .route("/ws", app_conf::ws_routes::get())
            .service(app_conf::open_routes::get_all())
            .service(app_conf::api_routes::get_all())
            .service(app_conf::aws_routes::get_all())
            .service(app_conf::static_routes::get_all())
            .default_service(web::to(|| HttpResponse::NotFound())) // 404
    });

    if let Some(max_nb_workers) = app_conf::nb_worker() {
        http_server
            .workers(max_nb_workers as usize)
            .bind(app_conf::get_listen_address())?
            .run()
            .await
    } else {
        http_server.bind(app_conf::get_listen_address())?
            .run()
            .await
    }
}
