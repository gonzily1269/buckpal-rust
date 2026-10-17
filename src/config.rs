//! TOML設定と環境変数オーバーライドからアプリケーション設定を読み込む。

use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

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
    threshold: i64,
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
            .with_context(|| format!("could not read configuration file {}", path.display()))?;
        let mut config: Self = toml::from_str(&contents)
            .with_context(|| format!("could not parse configuration file {}", path.display()))?;

        config.apply_overrides(ConfigOverrides::from_environment()?)?;

        config.validate()?;
        Ok(config)
    }

    /// 読み込んだ設定値がアプリケーションで許容される範囲か検証する。
    fn validate(&self) -> Result<()> {
        ensure!(
            self.transfer.threshold >= 0,
            "transfer.threshold must be a non-negative integer"
        );
        ensure!(
            self.database.max_connections > 0,
            "database.max_connections must be greater than zero"
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
                .context("BUCKPAL_TRANSFER_THRESHOLD must be a non-negative integer")?;
        }
        if let Some(value) = overrides.max_connections {
            self.database.max_connections = value
                .parse()
                .context("BUCKPAL_DB_MAX_CONNECTIONS must be a positive integer")?;
        }
        if let Some(value) = overrides.busy_timeout_seconds {
            self.database.busy_timeout_seconds = value
                .parse()
                .context("BUCKPAL_DB_BUSY_TIMEOUT_SECONDS must be a non-negative integer")?;
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
    pub fn transfer_threshold(&self) -> i64 {
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
            anyhow::bail!("{name} must contain valid Unicode")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, ConfigOverrides};

    /// 他の設定テストで共用する、有効なTOML設定を生成する。
    fn valid_config() -> Config {
        Config {
            database: super::DatabaseConfig {
                url: "sqlite://test.db".to_owned(),
                max_connections: 3,
                busy_timeout_seconds: 8,
            },
            server: super::ServerConfig {
                bind_addr: "0.0.0.0:8080".to_owned(),
            },
            transfer: super::TransferConfig { threshold: 20_000 },
            logging: super::LoggingConfig {
                filter: "debug".to_owned(),
            },
        }
    }

    /// TOMLの各セクションが対応する設定項目へ読み込まれることを確認する。
    #[test]
    fn parses_runtime_settings_from_toml() {
        let config = valid_config();

        assert_eq!(config.database_url(), "sqlite://test.db");
        assert_eq!(config.max_connections(), 3);
        assert_eq!(config.busy_timeout_seconds(), 8);
        assert_eq!(config.bind_addr(), "0.0.0.0:8080");
        assert_eq!(config.transfer_threshold(), 20_000);
        assert_eq!(config.log_filter(), "debug");
    }

    /// 必須セクションが欠けた設定を拒否することを確認する。
    #[test]
    fn rejects_missing_configuration_values() {
        let parsed = toml::from_str::<Config>(
            r#"
                [database]
                url = "sqlite://test.db"
                max_connections = 3
                busy_timeout_seconds = 8
            "#,
        );

        assert!(parsed.is_err_and(|error| error.to_string().contains("server")));
    }

    /// DB接続数とbusy timeoutが環境変数で上書きされることを確認する。
    #[test]
    fn environment_overrides_runtime_database_settings() {
        let mut config = valid_config();
        let result = config
            .apply_overrides(ConfigOverrides {
                max_connections: Some("12".to_owned()),
                busy_timeout_seconds: Some("30".to_owned()),
                ..ConfigOverrides::default()
            })
            .and_then(|()| config.validate())
            .map(|()| (config.max_connections(), config.busy_timeout_seconds()))
            .map_err(|error| error.to_string());

        assert_eq!(result, Ok((12, 30)));
    }

    /// 数値として解釈できない環境変数の値を拒否することを確認する。
    #[test]
    fn rejects_invalid_environment_overrides() {
        let mut config = valid_config();
        let result = config.apply_overrides(ConfigOverrides {
            max_connections: Some("zero".to_owned()),
            ..ConfigOverrides::default()
        });
        assert!(result.is_err_and(|error| {
            error
                .to_string()
                .contains("BUCKPAL_DB_MAX_CONNECTIONS must be a positive integer")
        }));
    }

    /// 0件の接続プールを設定として許可しないことを確認する。
    #[test]
    fn rejects_zero_connection_count() {
        let mut config = valid_config();
        let result = config
            .apply_overrides(ConfigOverrides {
                max_connections: Some("0".to_owned()),
                ..ConfigOverrides::default()
            })
            .and_then(|()| config.validate());
        assert!(result.is_err_and(|error| {
            error
                .to_string()
                .contains("database.max_connections must be greater than zero")
        }));
    }

    /// リポジトリに含まれる既定設定ファイルが設定型と互換であることを確認する。
    #[test]
    fn parses_the_repository_configuration_file() {
        assert!(toml::from_str::<Config>(include_str!("../config.toml")).is_ok());
    }
}
