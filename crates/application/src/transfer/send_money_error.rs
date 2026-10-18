use thiserror::Error;

/// 送金ユースケースが返す入力検証・業務・永続化エラー。
#[derive(Debug, Error)]
pub enum SendMoneyError {
    #[error("金額は1以上で指定してください")]
    /// 金額が正の整数ではない。
    InvalidAmount,
    #[error("送金元と送金先には異なる口座を指定してください")]
    /// 送金元と送金先に同じ口座が指定された。
    SameAccount,
    #[error("送金額が設定された上限を超えています")]
    /// 送金額が設定された上限を超えている。
    ThresholdExceeded,
    #[error("送金元または送金先の口座が見つかりません")]
    /// 送金元または送金先の口座が存在しない。
    AccountNotFound,
    #[error("送金元口座の残高が不足しています")]
    /// 送金元口座の残高が送金額に満たない。
    InsufficientFunds,
    #[error("送金先口座の残高が上限を超えます")]
    /// 入金によって送金先残高が整数の上限を超える。
    BalanceOverflow,
    #[error("データの読み込みまたは送金の記録に失敗しました")]
    /// 口座データの読込または送金記録に失敗した。
    PersistenceFailure,
}
