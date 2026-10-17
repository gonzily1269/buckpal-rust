//! SQLxを使ってSQLite上の残高・送金記録を読み書きするアダプター。

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{FromRow, SqlitePool};

use buckpal_application::{
    Dashboard, DashboardAccount, DashboardActivity, DashboardQueryFailure, LoadDashboardPort,
    LoadTransferAccountsPort, RecordTransferPort, TransferPortError,
};
use buckpal_domain::{Account, Transfer};

/// SQLiteデータベースに対する口座・送金リポジトリ。
#[derive(Clone)]
pub struct SqliteRepository {
    /// クエリとトランザクションに共有する`SQLite`接続プール。
    pool: SqlitePool,
}

impl SqliteRepository {
    /// 接続プールを保持するリポジトリを共有可能な形で生成する。
    #[must_use]
    pub fn new(pool: SqlitePool) -> Arc<Self> {
        Arc::new(Self { pool })
    }

    /// 口座残高と直近50件の送金履歴を読み、アプリケーションDTOへ変換する。
    async fn dashboard(&self) -> Result<Dashboard, sqlx::Error> {
        let accounts = sqlx::query_as::<_, AccountRow>(
            "SELECT a.id, a.name, a.opening_balance + COALESCE(SUM(CASE \
             WHEN x.target_account_id = a.id THEN x.amount \
             WHEN x.source_account_id = a.id THEN -x.amount ELSE 0 END), 0) AS balance \
             FROM account a LEFT JOIN activity x ON x.owner_account_id = a.id \
             GROUP BY a.id, a.name, a.opening_balance ORDER BY a.id",
        )
        .fetch_all(&self.pool)
        .await?;
        let accounts = accounts
            .into_iter()
            .map(|account| DashboardAccount {
                id: account.id,
                name: account.name,
                balance: account.balance,
            })
            .collect();

        let activities = sqlx::query_as::<_, ActivityRow>(
            "SELECT id, source_account_id, target_account_id, amount, created_at \
             FROM transfer ORDER BY id DESC LIMIT 50",
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|activity| DashboardActivity {
            id: activity.id,
            source_account_id: activity.source_account_id,
            target_account_id: activity.target_account_id,
            amount: activity.amount,
            created_at: activity.created_at,
        })
        .collect();

        Ok(Dashboard {
            accounts,
            activities,
        })
    }
}

#[derive(Debug, FromRow)]
/// ダッシュボード口座クエリから読み込む内部行。
struct AccountRow {
    /// 口座識別子。
    id: i64,
    /// 画面表示用の口座名。
    name: String,
    /// 取引履歴を反映した現在残高。
    balance: i64,
}

#[async_trait]
impl LoadDashboardPort for SqliteRepository {
    /// `SQLite`のダッシュボード読込をポートのエラー型へ変換して返す。
    async fn load_dashboard(&self) -> Result<Dashboard, DashboardQueryFailure> {
        self.dashboard().await.map_err(|error| {
            tracing::error!(error = ?error, "failed to load dashboard");
            DashboardQueryFailure
        })
    }
}

#[async_trait]
impl LoadTransferAccountsPort for SqliteRepository {
    /// 両口座の現在残高を計算し、ドメイン口座として返す。
    async fn load_accounts(
        &self,
        source_account_id: i64,
        target_account_id: i64,
    ) -> Result<Option<(Account, Account)>, TransferPortError> {
        let source_balance = account_balance_from_pool(&self.pool, source_account_id)
            .await
            .map_err(|error| load_error(&error))?;
        let target_balance = account_balance_from_pool(&self.pool, target_account_id)
            .await
            .map_err(|error| load_error(&error))?;
        let (Some(source_balance), Some(target_balance)) = (source_balance, target_balance) else {
            return Ok(None);
        };
        let source = account_from_storage(source_account_id, source_balance)?;
        let target = account_from_storage(target_account_id, target_balance)?;
        Ok(Some((source, target)))
    }
}

/// `SQLite`の送金行を`SQLx`で読み込むための内部レコード型。
#[derive(Debug, FromRow)]
struct ActivityRow {
    /// 送金履歴の識別子。
    id: i64,
    /// 送金元口座の識別子。
    source_account_id: i64,
    /// 送金先口座の識別子。
    target_account_id: i64,
    /// 送金額。
    amount: i64,
    /// 送金記録日時。
    created_at: String,
}

#[async_trait]
impl RecordTransferPort for SqliteRepository {
    /// SQLiteトランザクション内で競合を検出し、送金と活動記録を原子的に保存する。
    async fn record_transfer(&self, transfer: Transfer) -> Result<(), TransferPortError> {
        let source_account_id = transfer.source_account_id();
        let target_account_id = transfer.target_account_id();
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| record_error(&error))?;

        sqlx::query(
            "UPDATE account SET opening_balance = opening_balance \
             WHERE id IN (?, ?)",
        )
        .bind(source_account_id)
        .bind(target_account_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| record_error(&error))?;

        let source_balance = account_balance(&mut transaction, source_account_id)
            .await
            .map_err(|error| record_error(&error))?;
        let target_balance = account_balance(&mut transaction, target_account_id)
            .await
            .map_err(|error| record_error(&error))?;
        let (Some(source_balance), Some(target_balance)) = (source_balance, target_balance) else {
            return Err(TransferPortError::ConcurrentModification);
        };
        if source_balance != transfer.expected_source_balance()
            || target_balance != transfer.expected_target_balance()
        {
            return Err(TransferPortError::ConcurrentModification);
        }

        let created_at = chrono::Utc::now().to_rfc3339();
        let transfer_id = sqlx::query(
            "INSERT INTO transfer (source_account_id, target_account_id, amount, created_at) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(source_account_id)
        .bind(target_account_id)
        .bind(transfer.amount().amount())
        .bind(&created_at)
        .execute(&mut *transaction)
        .await
        .map_err(|error| record_error(&error))?
        .last_insert_rowid();

        for owner_account_id in [source_account_id, target_account_id] {
            sqlx::query(
                "INSERT INTO activity \
                 (transfer_id, owner_account_id, source_account_id, target_account_id, amount, created_at) \
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(transfer_id)
            .bind(owner_account_id)
            .bind(source_account_id)
            .bind(target_account_id)
            .bind(transfer.amount().amount())
            .bind(&created_at)
            .execute(&mut *transaction)
            .await
            .map_err(|error| record_error(&error))?;
        }

        transaction
            .commit()
            .await
            .map_err(|error| record_error(&error))
    }
}

/// 接続プールから指定口座の初期残高と取引履歴を集計して現在残高を返す。
///
/// 口座が存在しない場合は`None`を返し、クエリの失敗はエラーとして返す。
async fn account_balance_from_pool(
    pool: &SqlitePool,
    account_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT a.opening_balance + COALESCE(SUM(CASE \
         WHEN x.target_account_id = a.id THEN x.amount \
         WHEN x.source_account_id = a.id THEN -x.amount ELSE 0 END), 0) \
         FROM account a LEFT JOIN activity x ON x.owner_account_id = a.id \
         WHERE a.id = ? GROUP BY a.id, a.opening_balance",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
}

/// トランザクション内で指定口座の初期残高と取引履歴から現在残高を計算する。
///
/// 口座が存在しない場合は `None` を返し、クエリの失敗はエラーとして返す。
async fn account_balance(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    account_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT a.opening_balance + COALESCE(SUM(CASE \
         WHEN x.target_account_id = a.id THEN x.amount \
         WHEN x.source_account_id = a.id THEN -x.amount ELSE 0 END), 0) \
         FROM account a LEFT JOIN activity x ON x.owner_account_id = a.id \
         WHERE a.id = ? GROUP BY a.id, a.opening_balance",
    )
    .bind(account_id)
    .fetch_optional(&mut **transaction)
    .await
}

/// SQLxエラーを、口座読込ポートの技術詳細を隠したエラーへ変換する。
fn load_error(error: &sqlx::Error) -> TransferPortError {
    tracing::error!(error = ?error, "failed to load transfer accounts");
    TransferPortError::LoadFailed
}

/// SQLxエラーを、送金記録ポートの技術詳細を隠したエラーへ変換する。
fn record_error(error: &sqlx::Error) -> TransferPortError {
    tracing::error!(error = ?error, "failed to record transfer");
    TransferPortError::RecordFailed
}

/// 指定口座の永続化データをドメインの不変条件で検証して生成する。
fn account_from_storage(id: i64, balance: i64) -> Result<Account, TransferPortError> {
    Account::new(id, balance).map_err(|error| {
        tracing::error!(account_id = id, balance, error = ?error, "invalid account state in storage");
        TransferPortError::LoadFailed
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use buckpal_application::{
        LoadTransferAccountsPort, RecordTransferPort, SendMoneyError, SendMoneyService,
        TransferPortError,
    };
    use buckpal_domain::Money;
    use sqlx::sqlite::SqlitePoolOptions;

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
            let accounts_port: Arc<dyn LoadTransferAccountsPort> = repository.clone();
            let transfer_port: Arc<dyn RecordTransferPort> = repository.clone();
            let service = SendMoneyService::new(accounts_port, transfer_port, 10_000);

            service
                .send_money(1, 2, 500)
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
            let accounts_port: Arc<dyn LoadTransferAccountsPort> = repository.clone();
            let transfer_port: Arc<dyn RecordTransferPort> = repository.clone();
            let service = SendMoneyService::new(accounts_port, transfer_port, 10_000);

            let insufficient_funds = matches!(
                service.send_money(1, 2, 5_001).await,
                Err(SendMoneyError::InsufficientFunds)
            );
            let threshold_exceeded = matches!(
                service.send_money(1, 2, 10_001).await,
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

    /// 読込後に残高が更新された送金を拒否し、競合を検出することを確認する。
    #[tokio::test]
    async fn rejects_a_transfer_when_balances_changed_after_loading() {
        let result = async {
            let repository = repository().await.map_err(|error| error.to_string())?;
            let (source, target) = repository
                .load_accounts(1, 2)
                .await
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "seed accounts were not found".to_owned())?;
            let amount = Money::new(100).map_err(|error| error.to_string())?;
            let stale_transfer =
                Transfer::new(source, target, amount).map_err(|error| error.to_string())?;

            let accounts_port: Arc<dyn LoadTransferAccountsPort> = repository.clone();
            let transfer_port: Arc<dyn RecordTransferPort> = repository.clone();
            let service = SendMoneyService::new(accounts_port, transfer_port, 10_000);
            service
                .send_money(1, 2, 500)
                .await
                .map_err(|error| error.to_string())?;

            let concurrent_update = matches!(
                repository.record_transfer(stale_transfer).await,
                Err(TransferPortError::ConcurrentModification)
            );
            let activity_count = repository
                .load_dashboard()
                .await
                .map_err(|error| error.to_string())?
                .activities
                .len();
            Ok::<_, String>((concurrent_update, activity_count))
        }
        .await;

        assert_eq!(result, Ok((true, 1)));
    }
}
