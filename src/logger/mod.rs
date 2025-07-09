use async_trait::async_trait;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Extension,
};
use chrono::{DateTime, Utc};
use mongodb::Collection;
use serde::Serialize;
use std::{sync::Arc, time::Instant};
use thiserror::Error;
use tracing::{error, info};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

use crate::{error::ApiError, AppState};

#[derive(Error, Debug)]
pub enum LoggerError {
    #[error("File logger error: {0}")]
    FileLogger(#[from] std::io::Error),
    #[error("Database logger error: {0}")]
    MongoDbError(#[source] mongodb::error::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[async_trait]
pub trait DatabaseLogger: Send + Sync {
    async fn log(
        &self,
        status_code: StatusCode,
        message: String,
        duration: u64,
        uri: String,
    ) -> Result<(), LoggerError>;
}

#[derive(Serialize, Debug)]
pub struct DatabaseLog {
    message: String,
    status_code: u16,
    time: DateTime<Utc>,
    duration: u64,
    uri: String,
}
pub struct MognoDBLogger {
    collection: Collection<DatabaseLog>,
}

#[async_trait]
impl DatabaseLogger for MognoDBLogger {
    async fn log(
        &self,
        status_code: StatusCode,
        message: String,
        duration: u64,
        uri: String,
    ) -> Result<(), LoggerError> {
        let log = DatabaseLog {
            time: Utc::now(),
            message,
            status_code: status_code.as_u16(),
            duration,
            uri,
        };
        match self.collection.insert_one(log).await {
            Ok(_res) => return Ok(()),
            Err(err) => return Err(LoggerError::MongoDbError(err)),
        };
    }
}

impl MognoDBLogger {
    pub fn new(collection: Collection<DatabaseLog>) -> Self {
        Self { collection }
    }
}

pub struct FileLogger {
    _guard: Arc<WorkerGuard>,
}

impl FileLogger {
    pub fn init_logger(path: String) -> Self {
        let file_appender = RollingFileAppender::new(Rotation::DAILY, &path, "internal_logs.log");

        let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(fmt::Layer::new().with_writer(file_writer));

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set logger subscriber as default");

        info!("Logger initialized");

        Self {
            _guard: Arc::new(guard),
        }
    }
    pub fn flush(&self) {
        //lazy drop
        let _ = std::thread::spawn({
            let guard = Arc::clone(&self._guard);
            move || drop(guard)
        })
        .join();
    }
}

pub struct LoggerState {
    pub file_logger: Arc<FileLogger>,
    database_logger: Arc<dyn DatabaseLogger>,
}

impl LoggerState {
    pub fn new(file_log_path: String, log_collection: MognoDBLogger) -> Self {
        Self {
            file_logger: Arc::new(FileLogger::init_logger(file_log_path)),
            database_logger: Arc::new(log_collection),
        }
    }
}

pub async fn logger_middleware(
    State(app_state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let income_time = Instant::now();
    let uri = request.uri().clone();

    let res = next.run(request).await;
    let duration = income_time.elapsed().as_secs();
    match res.extensions().get::<ApiError>() {
        Some(err) => {
            let _res = app_state
                .logger
                .database_logger
                .log(res.status(), err.to_string(), duration, uri.to_string())
                .await
                .map_err(|err| error!("{}", err));
        }
        None => {
            let _res = app_state
                .logger
                .database_logger
                .log(
                    res.status(),
                    "Request done sucefully".to_string(),
                    duration,
                    uri.to_string(),
                )
                .await
                .map_err(|err| error!("{}", err));
        }
    };
    res
}
