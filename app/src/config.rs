//! TOML設定と環境変数オーバーライドからアプリケーション設定を読み込む。

use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;

/// アプリケーション起動時に使用する設定。
#[derive(Debug, Deserialize)]
pub struct Config {
    database: DatabaseConfig,
    server: ServerConfig,
    transfer: TransferConfig,
    logging: LoggingConfig,
}

/// SQLite接続先と接続プールの設定。
#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
    busy_timeout_seconds: u64,
}

/// HTTPサーバーの待受設定。
#[derive(Debug, Deserialize)]
struct ServerConfig {
    bind_addr: String,
}

/// 送金ユースケースの業務設定。
#[derive(Debug, Deserialize)]
struct TransferConfig {
    threshold: u64,
}

/// アプリケーションのログ出力設定。
#[derive(Debug, Deserialize)]
struct LoggingConfig {
    filter: String,
}

impl Config {
    /// config.tomlを読み、対応する環境変数で上書きした設定を検証して返す。
    pub fn load() -> Result<Self> {
        let path = env::var_os("BUCKPAL_CONFIG")
            .map_or_else(|| PathBuf::from("config.toml"), PathBuf::from);
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("設定ファイルを読み込めません: {}", path.display()))?;
        let mut config: Self = toml::from_str(&contents)
            .with_context(|| format!("設定ファイルを解析できません: {}", path.display()))?;

        config.apply_overrides(ConfigOverrides::from_environment()?)?;

        config.validate()?;
        Ok(config)
    }

    /// 読み込んだ設定値がアプリケーションで許容される範囲か検証する。
    fn validate(&self) -> Result<()> {
        ensure!(
            self.database.max_connections > 0,
            "database.max_connections は1以上で指定してください"
        );
        Ok(())
    }

    /// 環境変数から取得した値を設定へ反映し、数値項目を解析する。
    fn apply_overrides(&mut self, overrides: ConfigOverrides) -> Result<()> {
        if let Some(value) = overrides.database_url {
            self.database.url = value;
        }
        if let Some(value) = overrides.bind_addr {
            self.server.bind_addr = value;
        }
        if let Some(value) = overrides.transfer_threshold {
            self.transfer.threshold = value
                .parse()
                .context("BUCKPAL_TRANSFER_THRESHOLD は0以上の整数で指定してください")?;
        }
        if let Some(value) = overrides.max_connections {
            self.database.max_connections = value
                .parse()
                .context("BUCKPAL_DB_MAX_CONNECTIONS は1以上の整数で指定してください")?;
        }
        if let Some(value) = overrides.busy_timeout_seconds {
            self.database.busy_timeout_seconds = value
                .parse()
                .context("BUCKPAL_DB_BUSY_TIMEOUT_SECONDS は0以上の整数で指定してください")?;
        }
        if let Some(value) = overrides.log_filter {
            self.logging.filter = value;
        }
        Ok(())
    }

    /// SQLiteへの接続URLを返す。
    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    /// SQLite接続プールの最大接続数を返す。
    pub fn max_connections(&self) -> u32 {
        self.database.max_connections
    }

    /// SQLiteのロック取得を待つ最大秒数を返す。
    pub fn busy_timeout_seconds(&self) -> u64 {
        self.database.busy_timeout_seconds
    }

    /// HTTPサーバーの待受アドレスを返す。
    pub fn bind_addr(&self) -> &str {
        &self.server.bind_addr
    }

    /// 1回あたりに許可する送金上限を整数単位で返す。
    pub fn transfer_threshold(&self) -> u64 {
        self.transfer.threshold
    }

    /// `tracing`で使用するログフィルター文字列を返す。
    pub fn log_filter(&self) -> &str {
        &self.logging.filter
    }
}

#[derive(Default)]
/// ファイル設定を上書きする環境変数の値を保持する。
struct ConfigOverrides {
    database_url: Option<String>,
    bind_addr: Option<String>,
    transfer_threshold: Option<String>,
    max_connections: Option<String>,
    busy_timeout_seconds: Option<String>,
    log_filter: Option<String>,
}

impl ConfigOverrides {
    /// 対応する環境変数を読み取り、設定上書き値を作成する。
    fn from_environment() -> Result<Self> {
        Ok(Self {
            database_url: optional_env("DATABASE_URL")?,
            bind_addr: optional_env("BIND_ADDR")?,
            transfer_threshold: optional_env("BUCKPAL_TRANSFER_THRESHOLD")?,
            max_connections: optional_env("BUCKPAL_DB_MAX_CONNECTIONS")?,
            busy_timeout_seconds: optional_env("BUCKPAL_DB_BUSY_TIMEOUT_SECONDS")?,
            log_filter: optional_env("RUST_LOG")?,
        })
    }
}

/// 環境変数を任意のUTF-8文字列として取得し、未設定と不正値を区別する。
fn optional_env(name: &str) -> Result<Option<String>> {
    match env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => {
            anyhow::bail!("{name} の値は有効なUnicode文字列で指定してください")
        }
    }
}
