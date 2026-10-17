//! 送金に必要な口座読込と、検証済み送金の記録を行う出力ポート。

use async_trait::async_trait;
use thiserror::Error;

use buckpal_domain::{Account, Transfer};

/// 送金の読み書きで発生する、技術詳細を隠した境界エラー。
#[derive(Debug, Error)]
pub enum TransferPortError {
    #[error("account data could not be loaded")]
    /// 送金対象口座の読み込みに失敗した。
    LoadFailed,
    #[error("transfer could not be recorded")]
    /// 成立済み送金の記録に失敗した。
    RecordFailed,
    #[error("account balances changed before the transfer was recorded")]
    /// 読み込み後に口座残高が変わり、送金内容が古くなった。
    ConcurrentModification,
}

/// 送金する口座の現時点の状態を読み込む出力ポート。
#[async_trait]
pub trait LoadTransferAccountsPort: Send + Sync {
    /// 2口座を読み込む。いずれかが存在しない場合は `None` を返す。
    ///
    /// ストレージ障害は`TransferPortError`として返す。
    async fn load_accounts(
        &self,
        source_account_id: i64,
        target_account_id: i64,
    ) -> Result<Option<(Account, Account)>, TransferPortError>;
}

/// ユースケースがドメインルールを適用した送金を記録する出力ポート。
#[async_trait]
pub trait RecordTransferPort: Send + Sync {
    /// 読み込み時の残高が変わっていない場合に、送金を一括して記録する。
    ///
    /// 実装は永続化の原子性と競合検出を担い、業務上の可否判断は行わない。
    ///
    /// 読み込み後の残高が変わっていれば`ConcurrentModification`を返す。
    async fn record_transfer(&self, transfer: Transfer) -> Result<(), TransferPortError>;
}
