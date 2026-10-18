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
            .contains("BUCKPAL_DB_MAX_CONNECTIONS は1以上の整数で指定してください")
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
            .contains("database.max_connections は1以上で指定してください")
    }));
}

/// リポジトリに含まれる既定設定ファイルが設定型と互換であることを確認する。
#[test]
fn parses_the_repository_configuration_file() {
    assert!(toml::from_str::<Config>(include_str!("../../../../config.toml")).is_ok());
}
