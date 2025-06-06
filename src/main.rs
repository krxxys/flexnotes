use crate::{error::AppError, models::DB};
use auth::{auth_middleware, Keys};
use axum::{
    extract::State,
    http::{header, HeaderValue, Method, StatusCode},
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use dotenv::dotenv;
use logger::logger_middleware;
use models::{DatabaseModel, LogMessage, NoteInfo, UserInfo};
use mongodb::Client;
use std::sync::{Arc, LazyLock};
use tower_http::cors::CorsLayer;
use tracing_subscriber::fmt;

mod auth;
mod error;
mod logger;
mod models;
mod routes;

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

static MONGO_URL: LazyLock<String> =
    LazyLock::new(|| std::env::var("DB_URL").expect("DB_URL must be set"));

pub async fn setup_database() -> Arc<DatabaseModel> {
    let mongo_client = Client::with_uri_str(MONGO_URL.clone())
        .await
        .expect("Failed to connect to the mongodb server");

    let users_collection = mongo_client
        .database("flexnote")
        .collection::<UserInfo>("users");

    let notes_collection = mongo_client
        .database("flexnote")
        .collection::<NoteInfo>("notes");

    let logs_collection = mongo_client
        .database("flexnote")
        .collection::<LogMessage>("logs");

    Arc::new(DatabaseModel {
        notes: notes_collection,
        users: users_collection,
        logs: logs_collection,
        client: Arc::new(mongo_client),
    })
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    fmt::fmt().pretty().init();

    let database = setup_database().await;

    let port: String = std::env::var("BACKEND_PORT").unwrap_or("3001".to_string());

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
                database.clone(),
                auth_middleware,
            )),
        );

    let todo_nested_route = Router::new()
        .route(
            "/id/{id}/todos/{todo_id}",
            delete(routes::todos::delete_todo_by_id),
        )
        .route(
            "/id/{id}/todos/{todo_id}",
            patch(routes::todos::update_todo_by_id),
        )
        .route("/id/{id}/todos", post(routes::todos::create_todo))
        .route("/id/{id}/todos", get(routes::todos::get_todos_by_note_id));

    let note_routes = Router::new()
        .route("/create", post(routes::notes::create_note))
        .route("/", get(routes::notes::get_all_notes_info))
        .route(
            "/id/{id}",
            get(routes::notes::get_note_by_id)
                .patch(routes::notes::update_note_by_id)
                .delete(routes::notes::delete_note),
        )
        .merge(todo_nested_route)
        .layer(middleware::from_fn_with_state(
            database.clone(),
            auth_middleware,
        ));

    let app = Router::new()
        .nest("/auth", auth_routes)
        .nest("/notes", note_routes)
        .with_state(database.clone())
        .layer(middleware::from_fn_with_state(
            database.clone(),
            logger_middleware,
        ))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect(format!("Failed to listen on port: {}", port).as_str());
    axum::serve(listener, app).await.unwrap();
}
