use std::time::Instant;

use crate::error::AppError;
use crate::models::{LogMessageStatus, DB};
use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
pub async fn logger_middleware(
    State(db): State<DB>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let start = Instant::now();

    let method = request.method().clone();
    let uri = request.uri().clone();
    let response = next.run(request).await;
    let status = response.status();
    tracing::info!(
        "Request: {}: {} | Response: {} | , Duration: {:?}",
        method,
        uri,
        response.status(),
        start.elapsed()
    );

    if let Some(err) = response.extensions().get::<AppError>() {
        db.push_log(
            LogMessageStatus::ERROR,
            format!("{}:{}", err.status_code.as_str(), err.message),
            uri.to_string(),
        )
        .await
    } else {
        db.push_log(LogMessageStatus::OK, format!("{}", status), uri.to_string())
            .await
    }

    response
}
