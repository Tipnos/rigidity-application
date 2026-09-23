use axum::{extract::Path, http::{header, StatusCode}, response::IntoResponse, routing::get, Router};
use tower_http::services::ServeDir;
use std::fs;
use crate::AppState;

const AUTHORIZED_STATIC_PATHS: & [&str] = &[
    "ask_password_reset.html", 
    "login.html",
    "reset_password.html",
    "email_confirmation.html",
];

pub fn get_all() -> Router<AppState> {
    Router::new().nest("/static", Router::new()
        .route("/{html_file_path}", get(static_file_http_response))
        .nest_service("/assets", ServeDir::new("static/assets")))
}

async fn static_file_http_response(Path(html_file_path): Path<String>) -> impl IntoResponse {
    // check if it's an authorized path
    let path = html_file_path.to_string();
    let mut find = false;
    for p in AUTHORIZED_STATIC_PATHS {
        if p.to_string() == path {
            find = true;
            break;
        }
    }

    let error_closure = move || {
        let error_message = format!("Unknown path: {}", path);

        (StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, String::from("text/html; charset=utf-8"))],
            error_message)
    };

    if !find {
        error_closure()
    } else {
        let extension = html_file_path.split(".").last().unwrap_or_default();

        if extension.len() == 0  {
            return error_closure();
        }
        let full_path = format!("static/html/{}", html_file_path);

        match fs::read_to_string(full_path) {
            Ok(contents) => {
                let mut content_type = "text/".to_owned();

                if extension != "js" {
                    content_type.push_str(&extension);
                } else {
                    content_type.push_str("javascript");
                }
                
                content_type.push_str("; charset=utf-8");
                let content_type = content_type;

                (StatusCode::OK,
                    [(header::CONTENT_TYPE, content_type)],
                    contents)
            }, Err(e) => {
                println!("{}", e);
                error_closure()
            }
        }      
    }
}