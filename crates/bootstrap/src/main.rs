//! Leptos SSR と SQLite で口座間送金を行う Buckpal アプリケーション。
//!
//! 起動時に設定を読み込み、SQLite接続とマイグレーションを準備してから
//! AxumのHTTPサーバーを開始する。

mod config;

use std::{str::FromStr, time::Duration};

use anyhow::{Context, Result};
use axum::{Router, routing::get};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use tracing_subscriber::EnvFilter;

use buckpal_adapters::sqlite::SqliteRepository;
use buckpal_application::{
    GetDashboardService, GetDashboardUseCase, SendMoneyService, SendMoneyUseCase,
    TransferRepository,
};
use buckpal_web::AppState;
use config::Config;

#[tokio::main]
/// データベースとHTTPサーバーを初期化し、アプリケーションを起動する。
async fn main() -> Result<()> {
    let config = Config::load()?;
    any_spawner::Executor::init_tokio().context("could not initialize the Leptos executor")?;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_new(config.log_filter()).context("invalid log filter configuration")?,
        )
        .init();

    let options = SqliteConnectOptions::from_str(config.database_url())
        .context("DATABASE_URL must be a valid SQLite connection URL")?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(config.busy_timeout_seconds()));
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections())
        .connect_with(options)
        .await
        .context("could not connect to SQLite")?;

    initialize_database(&pool).await?;

    let repository = SqliteRepository::new(pool);
    let transfer_repository: std::sync::Arc<dyn TransferRepository> = repository.clone();
    let service = SendMoneyService::new(transfer_repository, config.transfer_threshold());
    let send_money: std::sync::Arc<dyn SendMoneyUseCase> = std::sync::Arc::new(service);
    let dashboard_service = GetDashboardService::new(repository);
    let get_dashboard: std::sync::Arc<dyn GetDashboardUseCase> =
        std::sync::Arc::new(dashboard_service);
    let state = AppState::new(send_money, get_dashboard);
    let app = Router::new()
        .route(
            "/",
            get(buckpal_web::home).post(buckpal_web::submit_transfer),
        )
        .route("/health", get(|| async { "ok" }))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(config.bind_addr())
        .await
        .with_context(|| format!("could not bind to {}", config.bind_addr()))?;
    tracing::info!(address = %listener.local_addr()?, "Buckpal is running");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("HTTP server failed")?;
    Ok(())
}

/// Ctrl-CまたはUnixのSIGTERMを待ち、サーバーへgraceful shutdownを通知する。
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let terminate = signal(SignalKind::terminate());
        match terminate {
            Ok(mut terminate) => {
                tokio::select! {
                    result = tokio::signal::ctrl_c() => {
                        if let Err(error) = result {
                            tracing::error!(error = %error, "failed to listen for Ctrl-C");
                        }
                    }
                    _ = terminate.recv() => {}
                }
            }
            Err(error) => {
                tracing::error!(error = %error, "failed to register SIGTERM handler");
                if let Err(error) = tokio::signal::ctrl_c().await {
                    tracing::error!(error = %error, "failed to listen for Ctrl-C");
                }
            }
        }
    }

    #[cfg(not(unix))]
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(error = %error, "failed to listen for Ctrl-C");
    }

    tracing::info!("shutdown signal received; gracefully stopping server");
}

/// 埋め込み済みのSQLxマイグレーションを実行してデータベースを初期化する。
async fn initialize_database(pool: &SqlitePool) -> Result<()> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .context("database migrations failed")?;
    Ok(())
}
