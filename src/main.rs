use crate::logger::{logger_middleware, LoggerState};
use auth::{auth_middleware, Keys};
use axum::{
    http::{header, HeaderValue, Method},
    middleware,
    routing::{get, patch, post},
    Router,
};
use database::Database;
use dotenv::dotenv;
use std::sync::{Arc, LazyLock};
use tower_http::cors::CorsLayer;
use tracing::info;

mod auth;
mod database;
mod error;
mod logger;
mod models;
mod repository;
mod routes;
mod services;

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

static MONGO_URL: LazyLock<String> =
    LazyLock::new(|| std::env::var("DB_URL").expect("DB_URL must be set"));

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<Database>,
    pub logger: Arc<LoggerState>,
}

impl AppState {
    pub async fn new() -> Self {
        let db_state = Arc::new(Database::new().await);
        Self {
            database: db_state.clone(),
            logger: Arc::new(LoggerState::new(
                "./logs/".to_string(),
                db_state.logs_repo(),
            )),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let app_state = AppState::new().await;

    let port: String = std::env::var("BACKEND_PORT").unwrap_or("3001".to_string());

    info!("Server is starting");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_credentials(true);

    let auth_routes = Router::new()
        .route("/login", post(routes::auth::authorize))
        .route("/register", post(routes::auth::register))
        .route("/refresh", post(routes::auth::refresh_token))
        .route(
            "/check",
            get(routes::auth::check_auth).layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_middleware,
            )),
        )
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            logger_middleware,
        ));

    let todo_list_route = Router::new()
        .route(
            "/",
            post(routes::todos::create_todo_list).get(routes::todos::get_all_todo_lists),
        )
        .route("/id", get(routes::todos::get_all_todos_by_id))
        .route(
            "/id/{todo_list_id}",
            patch(routes::todos::rename_todo_list)
                .delete(routes::todos::delete_todo_list)
                .post(routes::todos::create_todo),
        )
        .route(
            "/id/{todo_list_id}/todo/id/{todo_id}",
            patch(routes::todos::modify_todo).delete(routes::todos::delete_todo),
        )
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            logger_middleware,
        ));

    let note_routes = Router::new()
        .route("/create", post(routes::notes::create_note))
        .route("/", get(routes::notes::get_all_notes_info))
        .route(
            "/id/{id}",
            get(routes::notes::get_note_by_id)
                .patch(routes::notes::update_note_by_id)
                .delete(routes::notes::delete_note),
        )
        .route(
            "/id/{id}/pin/{todo_list_id}",
            patch(routes::notes::pin_todo_list).delete(routes::notes::unpin_todo_list),
        )
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            logger_middleware,
        ));

    let app = Router::new()
        .nest("/auth", auth_routes)
        .nest("/notes", note_routes)
        .nest("/todos", todo_list_route)
        .with_state(app_state.clone())
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Server listen on 0.0.0.0:{}", port);
    axum::serve(listener, app).await.unwrap();

    //enforce to lazy drop of file logger state idk if this is good aproach but works...
    app_state.logger.file_logger.flush();

    Ok(())
}
