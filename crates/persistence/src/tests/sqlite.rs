use super::sqlite::SqliteRepository;
use std::{num::NonZeroU64, sync::Arc};

use buckpal_service::{SendMoneyError, SendMoneyService, TransferRepository};
use sqlx::sqlite::SqlitePoolOptions;

fn account_id(id: u64) -> NonZeroU64 {
    NonZeroU64::new(id).unwrap_or(NonZeroU64::MIN)
}

fn amount(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap_or(NonZeroU64::MIN)
}

/// インメモリ`SQLite`データベースへマイグレーションを適用したリポジトリを作成する。
async fn repository() -> anyhow::Result<Arc<SqliteRepository>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    Ok(SqliteRepository::new(pool))
}

/// 成功した送金が両口座の残高と取引履歴に反映されることを確認する。
#[tokio::test]
async fn transfer_updates_both_balances_and_records_one_transfer() {
    let result = async {
        let repository = repository().await.map_err(|error| error.to_string())?;
        let repository_port: Arc<dyn TransferRepository> = repository.clone();
        let service = SendMoneyService::new(repository_port, 10_000);

        service
            .send_money(NonZeroU64::MIN, account_id(2), amount(500))
            .await
            .map_err(|error| error.to_string())?;

        let dashboard = repository
            .load_dashboard()
            .await
            .map_err(|error| error.to_string())?;
        let balances = dashboard
            .accounts
            .iter()
            .map(|account| account.balance)
            .collect::<Vec<_>>();
        let amounts = dashboard
            .activities
            .iter()
            .map(|activity| activity.amount)
            .collect::<Vec<_>>();
        Ok::<_, String>((balances, amounts))
    }
    .await;

    assert_eq!(result, Ok((vec![4_500, 3_000], vec![500])));
}

/// 残高不足または上限超過の送金が残高・履歴を変更しないことを確認する。
#[tokio::test]
async fn failed_transfers_leave_balances_and_history_unchanged() {
    let result = async {
        let repository = repository().await.map_err(|error| error.to_string())?;
        let repository_port: Arc<dyn TransferRepository> = repository.clone();
        let service = SendMoneyService::new(repository_port, 10_000);

        let insufficient_funds = matches!(
            service
                .send_money(NonZeroU64::MIN, account_id(2), amount(5_001))
                .await,
            Err(SendMoneyError::InsufficientFunds)
        );
        let threshold_exceeded = matches!(
            service
                .send_money(NonZeroU64::MIN, account_id(2), amount(10_001))
                .await,
            Err(SendMoneyError::ThresholdExceeded)
        );

        let dashboard = repository
            .load_dashboard()
            .await
            .map_err(|error| error.to_string())?;
        let balances = dashboard
            .accounts
            .iter()
            .map(|account| account.balance)
            .collect::<Vec<_>>();
        Ok::<_, String>((
            insufficient_funds,
            threshold_exceeded,
            balances,
            dashboard.activities.len(),
        ))
    }
    .await;

    assert_eq!(result, Ok((true, true, vec![5_000, 2_500], 0)));
}
