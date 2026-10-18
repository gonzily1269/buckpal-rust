use thiserror::Error;

/// 送金ユースケースが返す入力検証・業務・永続化エラー。
#[derive(Debug, Error)]
pub enum SendMoneyError {
    #[error("amount must be greater than zero")]
    /// 金額が正の整数ではない。
    InvalidAmount,
    #[error("source and target accounts must be different")]
    /// 送金元と送金先に同じ口座が指定された。
    SameAccount,
    #[error("transfer exceeds the configured limit")]
    /// 送金額が設定された上限を超えている。
    ThresholdExceeded,
    #[error("account does not exist")]
    /// 送金元または送金先の口座が存在しない。
    AccountNotFound,
    #[error("source account has insufficient funds")]
    /// 送金元口座の残高が送金額に満たない。
    InsufficientFunds,
    #[error("target account balance exceeds the supported range")]
    /// 入金によって送金先残高が整数の上限を超える。
    BalanceOverflow,
    #[error("transfer could not be completed because storage failed")]
    /// 口座データの読込または送金記録に失敗した。
    PersistenceFailure,
}
