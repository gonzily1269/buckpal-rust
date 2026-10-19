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

use buckpal_persistence::sqlite::SqliteRepository;
use buckpal_service::{
    GetDashboardService, GetDashboardUseCase, SendMoneyService, SendMoneyUseCase,
    TransferRepository,
};
use buckpal_web::AppState;
use config::Config;

#[tokio::main]
/// データベースとHTTPサーバーを初期化し、アプリケーションを起動する。
async fn main() -> Result<()> {
    let config = Config::load()?;
    any_spawner::Executor::init_tokio().context("Leptosの実行環境を初期化できませんでした")?;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_new(config.log_filter()).context("ログフィルターの設定が無効です")?,
        )
        .init();

    let options = SqliteConnectOptions::from_str(config.database_url())
        .context("DATABASE_URLには有効なSQLite接続URLを指定してください")?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(config.busy_timeout_seconds()));
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections())
        .connect_with(options)
        .await
        .context("SQLiteに接続できませんでした")?;

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
        .with_context(|| format!("{}で待ち受けを開始できませんでした", config.bind_addr()))?;
    tracing::info!(address = %listener.local_addr()?, "Buckpalを起動しました");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("HTTPサーバーでエラーが発生しました")?;
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
                            tracing::error!(error = %error, "Ctrl-Cシグナルを待機できませんでした");
                        }
                    }
                    _ = terminate.recv() => {}
                }
            }
            Err(error) => {
                tracing::error!(error = %error, "SIGTERMハンドラーを登録できませんでした");
                if let Err(error) = tokio::signal::ctrl_c().await {
                    tracing::error!(error = %error, "Ctrl-Cシグナルを待機できませんでした");
                }
            }
        }
    }

    #[cfg(not(unix))]
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(error = %error, "Ctrl-Cシグナルを待機できませんでした");
    }

    tracing::info!("終了シグナルを受信しました。サーバーを正常に終了します");
}

/// 埋め込み済みのSQLxマイグレーションを実行してデータベースを初期化する。
async fn initialize_database(pool: &SqlitePool) -> Result<()> {
    sqlx::migrate!("../migrations")
        .run(pool)
        .await
        .context("データベースのマイグレーションに失敗しました")?;
    Ok(())
}
