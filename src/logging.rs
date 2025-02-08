use std::time::Instant;

use axum::{body::Body, http::{request, Request}, middleware::Next, response::Response};



pub async fn loggin_middleware(request: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    tracing::info!(
        "Request: {}, {}", request.method(), request.uri()
    );

    let response = next.run(request).await;
    
    tracing::info!("Response: {}, Duration: {:?}", response.status(), start.elapsed());
  
    response
}