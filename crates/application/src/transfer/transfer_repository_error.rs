use thiserror::Error;

/// 送金リポジトリで発生する、技術詳細を隠したエラー。
#[derive(Debug, Error)]
pub enum TransferRepositoryError {
    #[error("dashboard data could not be loaded")]
    /// ダッシュボード情報の読み込みに失敗した。
    DashboardLoadFailed,
    #[error("account data could not be loaded")]
    /// 送金対象口座の読み込みに失敗した。
    LoadFailed,
    #[error("transfer could not be recorded")]
    /// 成立済み送金の記録に失敗した。
    RecordFailed,
    #[error("account does not exist")]
    /// 送金元または送金先の口座が存在しない。
    AccountNotFound,
    #[error("source account has insufficient funds")]
    /// 送金元口座の残高が送金額に満たない。
    InsufficientFunds,
    #[error("target account balance exceeds the supported range")]
    /// 入金によって送金先残高が整数の上限を超える。
    BalanceOverflow,
}
