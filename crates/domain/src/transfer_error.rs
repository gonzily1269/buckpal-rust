use thiserror::Error;

/// 送金を成立させられない業務上の理由。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TransferError {
    #[error("送金元と送金先には異なる口座を指定してください")]
    /// 送金元と送金先が同じ口座である。
    SameAccount,
    #[error("送金元口座の残高が不足しています")]
    /// 送金額に対して送金元の残高が不足している。
    InsufficientFunds,
    #[error("送金先口座の残高が上限を超えます")]
    /// 入金すると送金先の残高が整数型の範囲を超える。
    BalanceOverflow,
}
