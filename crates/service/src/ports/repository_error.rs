use thiserror::Error;

/// リポジトリポートで発生する、技術詳細を隠したエラー。
#[derive(Debug, Error)]
pub enum TransferRepositoryError {
    #[error("ダッシュボード情報を読み込めませんでした")]
    /// ダッシュボード情報の読み込みに失敗した。
    DashboardLoadFailed,
    #[error("口座情報を読み込めませんでした")]
    /// 送金対象口座の読み込みに失敗した。
    LoadFailed,
    #[error("送金を記録できませんでした")]
    /// 成立済み送金の記録に失敗した。
    RecordFailed,
    #[error("口座が見つかりません")]
    /// 送金元または送金先の口座が存在しない。
    AccountNotFound,
    #[error("送金元口座の残高が不足しています")]
    /// 送金元口座の残高が送金額に満たない。
    InsufficientFunds,
    #[error("送金先口座の残高が上限を超えます")]
    /// 入金によって送金先残高が整数の上限を超える。
    BalanceOverflow,
}
