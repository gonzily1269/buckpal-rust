use thiserror::Error;

/// 送金を成立させられない業務上の理由。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TransferError {
    #[error("source and target accounts must be different")]
    /// 送金元と送金先が同じ口座である。
    SameAccount,
    #[error("source account has insufficient funds")]
    /// 送金額に対して送金元の残高が不足している。
    InsufficientFunds,
    #[error("target account balance exceeds the supported range")]
    /// 入金すると送金先の残高が整数型の範囲を超える。
    BalanceOverflow,
}
