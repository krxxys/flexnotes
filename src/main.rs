use auth::{auth_middleware, Keys};
use axum::{
    http::{header, HeaderValue, Method},
    middleware,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use jsonwebtoken::Header;
use logging::logging_middleware;
use models::{DatabaseModel, NoteInfo, UserInfo};
use mongodb::{
    bson::{DateTime, Timestamp},
    Client,
};
use std::{
    sync::{Arc, LazyLock},
    vec,
};
use tower_http::cors::{AllowOrigin, Cors, CorsLayer};
use tracing_subscriber::{fmt, layer};

mod auth;
mod logging;
pub(crate) mod models;
mod routes;

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

static MONGO_URL: LazyLock<String> =
    LazyLock::new(|| std::env::var("DB_URL").expect("DB_URL must be set"));

#[tokio::main]
async fn main() {
    dotenv().ok();

    fmt::fmt().pretty().init();

    let refresh_store = match sled::open("/tmp/refresh_store") {
        Ok(store) => store,
        Err(_) => panic!("Failed to open refresh token db"),
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_credentials(true);

    let client = Client::with_uri_str(MONGO_URL.clone())
        .await
        .expect("Failed to connect to the mongodb server");
    let users_collection = client.database("flexnote").collection::<UserInfo>("users");
    let notes_collection = client.database("flexnote").collection::<NoteInfo>("notes");
    let db_model = Arc::new(DatabaseModel {
        notes: notes_collection,
        users: users_collection,
    });

    let auth_routes = Router::new()
        .route("/login", post(routes::auth::authorize))
        .route("/register", post(routes::auth::register))
        .route("/check", get(routes::token_check::protected))
        .route("/refresh", post(routes::auth::refresh_token));

    let note_routes = Router::new()
        .route("/create", post(routes::notes::create_note))
        .route("/", get(routes::notes::get_all_notes_info))
        .route(
            "/id/{id}",
            get(routes::notes::get_note_by_id)
                .patch(routes::notes::update_note_by_id)
                .delete(routes::notes::delete_note),
        )
        .layer(middleware::from_fn(auth_middleware));

    let app = Router::new()
        .route("/", get(root))
        .nest("/auth", auth_routes)
        .nest("/notes", note_routes)
        .with_state(db_model)
        .with_state(Arc::new(refresh_store))
        .layer(middleware::from_fn(logging_middleware))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello world!"
}
