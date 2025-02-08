use auth::Keys;
use axum::{
    middleware, routing::{get, post}, Router
};
use dotenv::dotenv;
use logging::loggin_middleware;
use models::{DatabaseModel, NoteInfo, UserInfo};
use mongodb::Client;
use tracing_subscriber::fmt;
use std::sync::{Arc, LazyLock};

mod auth;
pub(crate) mod models;
mod routes;
mod logging;

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
        .route("/check", get(routes::token_check::protected));

    let note_routes = Router::new()
        .route("/create", post(routes::notes::create_note))
        .route("/", get(routes::notes::get_all_notes_info))
        .route(
            "/id/{id}",
            get(routes::notes::get_note_by_id)
                .patch(routes::notes::update_note_by_id)
                .delete(routes::notes::delete_note),
        );

    let app = Router::new()
        .route("/", get(root))
        .nest("/auth", auth_routes)
        .nest("/notes", note_routes)
        .with_state(db_model)
        .layer(middleware::from_fn(loggin_middleware));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello world!"
}
