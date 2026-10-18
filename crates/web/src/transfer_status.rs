use serde::Deserialize;

/// 送金処理の結果として画面に表示する状態。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransferStatus {
    /// 送金が完了した。
    Success,
    /// 送金額が無効だった。
    InvalidAmount,
    /// 送金元と送金先が同じだった。
    SameAccount,
    /// 送金上限を超過した。
    Threshold,
    /// 指定された口座が見つからなかった。
    AccountNotFound,
    /// 送金元の残高が不足していた。
    InsufficientFunds,
    /// 送金後の残高が表現可能な上限を超える。
    BalanceOverflow,
    /// 口座残高が同時更新されていた。
    ConcurrentUpdate,
}

impl TransferStatus {
    /// リダイレクトURLに使う、互換性のある結果コードを返す。
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::InvalidAmount => "invalid-amount",
            Self::SameAccount => "same-account",
            Self::Threshold => "threshold",
            Self::AccountNotFound => "account-not-found",
            Self::InsufficientFunds => "insufficient-funds",
            Self::BalanceOverflow => "balance-overflow",
            Self::ConcurrentUpdate => "concurrent-update",
        }
    }
}
