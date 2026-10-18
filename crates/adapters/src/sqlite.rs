//! SQLxを使ってSQLite上の残高・送金記録を読み書きするアダプター。

use std::num::NonZeroU64;
use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{FromRow, SqliteConnection, SqlitePool};

use buckpal_application::{
    Dashboard, DashboardAccount, DashboardActivity, TransferRepository, TransferRepositoryError,
};
use buckpal_domain::{Account, Transfer, TransferError};

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
            .map(|account| {
                let id = storage_id(account.id, "口座ID")?;
                let balance = storage_amount(account.balance, "口座残高")?;
                Ok(DashboardAccount {
                    id,
                    name: account.name,
                    balance,
                })
            })
            .collect::<Result<_, sqlx::Error>>()?;

        let activities = sqlx::query_as::<_, ActivityRow>(
            "SELECT id, source_account_id, target_account_id, amount, created_at \
             FROM transfer ORDER BY id DESC LIMIT 50",
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(activity_from_storage)
        .collect::<Result<_, _>>()?;

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

fn activity_from_storage(activity: ActivityRow) -> Result<DashboardActivity, sqlx::Error> {
    let id = storage_id(activity.id, "送金ID")?;
    let source_account_id = storage_id(activity.source_account_id, "送金元口座ID")?;
    let target_account_id = storage_id(activity.target_account_id, "送金先口座ID")?;
    let amount = storage_amount(activity.amount, "送金額")?;

    Ok(DashboardActivity {
        id,
        source_account_id,
        target_account_id,
        amount,
        created_at: activity.created_at,
    })
}

#[async_trait]
impl TransferRepository for SqliteRepository {
    /// `SQLite`のダッシュボード読込をリポジトリのエラー型へ変換して返す。
    async fn load_dashboard(&self) -> Result<Dashboard, TransferRepositoryError> {
        self.dashboard().await.map_err(|error| {
            tracing::error!(error = ?error, "ダッシュボード情報の読み込みに失敗しました");
            TransferRepositoryError::DashboardLoadFailed
        })
    }

    /// 書き込みロック下で残高を読み、ドメイン検証と送金記録を一括して行う。
    async fn transfer(
        &self,
        source_account_id: NonZeroU64,
        target_account_id: NonZeroU64,
        amount: NonZeroU64,
    ) -> Result<(), TransferRepositoryError> {
        let source_account_id = database_id_for_record(source_account_id)?;
        let target_account_id = database_id_for_record(target_account_id)?;
        let mut connection = self
            .pool
            .acquire()
            .await
            .map_err(|error| record_error(&error))?;

        // SQLiteには行単位の`SELECT FOR UPDATE`がないため、書き込みトランザクションを
        // 先に開始して、残高の読込から記録まで他の書き込み処理を待機させる。
        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut *connection)
            .await
            .map_err(|error| record_error(&error))?;

        let result = Self::transfer_in_transaction(
            &mut connection,
            source_account_id,
            target_account_id,
            amount,
        )
        .await;

        match result {
            Ok(()) => sqlx::query("COMMIT")
                .execute(&mut *connection)
                .await
                .map(|_| ())
                .map_err(|error| record_error(&error)),
            Err(error) => {
                if let Err(rollback_error) = sqlx::query("ROLLBACK").execute(&mut *connection).await
                {
                    tracing::error!(error = ?rollback_error, "送金のロールバックに失敗しました");
                }
                Err(error)
            }
        }
    }
}

impl SqliteRepository {
    async fn transfer_in_transaction(
        connection: &mut SqliteConnection,
        source_account_id: i64,
        target_account_id: i64,
        amount: NonZeroU64,
    ) -> Result<(), TransferRepositoryError> {
        let source_balance = account_balance(connection, source_account_id)
            .await
            .map_err(|error| record_error(&error))?;
        let target_balance = account_balance(connection, target_account_id)
            .await
            .map_err(|error| record_error(&error))?;

        let (Some(source_balance), Some(target_balance)) = (source_balance, target_balance) else {
            return Err(TransferRepositoryError::AccountNotFound);
        };
        let source = account_from_storage(source_account_id, source_balance)?;
        let target = account_from_storage(target_account_id, target_balance)?;
        let transfer = Transfer::new(source, target, amount).map_err(|error| match error {
            TransferError::SameAccount => TransferRepositoryError::RecordFailed,
            TransferError::InsufficientFunds => TransferRepositoryError::InsufficientFunds,
            TransferError::BalanceOverflow => TransferRepositoryError::BalanceOverflow,
        })?;

        let amount = database_amount(transfer.amount().get())?;
        let created_at = chrono::Utc::now().to_rfc3339();
        let transfer_id = sqlx::query(
            "INSERT INTO transfer (source_account_id, target_account_id, amount, created_at) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(source_account_id)
        .bind(target_account_id)
        .bind(amount)
        .bind(&created_at)
        .execute(&mut *connection)
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
            .bind(amount)
            .bind(&created_at)
            .execute(&mut *connection)
            .await
            .map_err(|error| record_error(&error))?;
        }

        Ok(())
    }
}

/// 接続プールから指定口座の初期残高と取引履歴を集計して現在残高を返す。
///
/// 口座が存在しない場合は`None`を返し、クエリの失敗はエラーとして返す。
/// トランザクション内で指定口座の初期残高と取引履歴から現在残高を計算する。
///
/// 口座が存在しない場合は `None` を返し、クエリの失敗はエラーとして返す。
async fn account_balance(
    connection: &mut SqliteConnection,
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
    .fetch_optional(&mut *connection)
    .await
}

/// SQLxエラーを、送金リポジトリの技術詳細を隠したエラーへ変換する。
fn record_error(error: &sqlx::Error) -> TransferRepositoryError {
    tracing::error!(error = ?error, "送金の記録に失敗しました");
    TransferRepositoryError::RecordFailed
}

/// 指定口座の永続化データをドメインの不変条件で検証して生成する。
fn account_from_storage(id: i64, balance: i64) -> Result<Account, TransferRepositoryError> {
    let id = repository_id(id)?;
    let balance = repository_amount(balance)?;
    Ok(Account::new(id, balance))
}

fn storage_id(value: i64, name: &str) -> Result<NonZeroU64, sqlx::Error> {
    NonZeroU64::new(u64::try_from(value).map_err(|_| invalid_storage_value(name))?)
        .ok_or_else(|| invalid_storage_value(name))
}

fn storage_amount(value: i64, name: &str) -> Result<u64, sqlx::Error> {
    u64::try_from(value).map_err(|_| invalid_storage_value(name))
}

fn invalid_storage_value(name: &str) -> sqlx::Error {
    sqlx::Error::Protocol(format!("{name} が許容範囲を超えています"))
}

fn database_id_for_record(value: NonZeroU64) -> Result<i64, TransferRepositoryError> {
    i64::try_from(value.get()).map_err(|_| TransferRepositoryError::RecordFailed)
}

fn database_amount(value: u64) -> Result<i64, TransferRepositoryError> {
    i64::try_from(value).map_err(|_| TransferRepositoryError::RecordFailed)
}

fn repository_id(value: i64) -> Result<NonZeroU64, TransferRepositoryError> {
    NonZeroU64::new(u64::try_from(value).map_err(|_| TransferRepositoryError::LoadFailed)?)
        .ok_or(TransferRepositoryError::LoadFailed)
}

fn repository_amount(value: i64) -> Result<u64, TransferRepositoryError> {
    u64::try_from(value).map_err(|_| TransferRepositoryError::LoadFailed)
}
