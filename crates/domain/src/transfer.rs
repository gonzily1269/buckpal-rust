//! ドメインルールを満たした口座間送金を表す。

use std::num::NonZeroU64;

use crate::{Account, transfer_error::TransferError};

/// 口座残高の事前状態とドメイン検証済みの送金内容。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transfer {
    source_account: Account,
    target_account: Account,
    amount: NonZeroU64,
    resulting_source_balance: u64,
    resulting_target_balance: u64,
}

impl Transfer {
    /// 口座の残高ルールを検証し、送金記録に必要な不変な値を作る。
    ///
    /// # Errors
    ///
    /// 同一口座間の送金、残高不足、または送金先残高のオーバーフローの場合にエラーを返す。
    pub fn new(
        source_account: Account,
        target_account: Account,
        amount: NonZeroU64,
    ) -> Result<Self, TransferError> {
        if source_account.id() == target_account.id() {
            return Err(TransferError::SameAccount);
        }

        let mut resulting_source = source_account.clone();
        if !resulting_source.withdraw(amount) {
            return Err(TransferError::InsufficientFunds);
        }
        let mut resulting_target = target_account.clone();
        if resulting_target.deposit(amount).is_none() {
            return Err(TransferError::BalanceOverflow);
        }

        Ok(Self {
            source_account,
            target_account,
            amount,
            resulting_source_balance: resulting_source.balance(),
            resulting_target_balance: resulting_target.balance(),
        })
    }

    /// 送金元口座の識別子を返す。
    #[must_use]
    pub fn source_account_id(&self) -> NonZeroU64 {
        self.source_account.id()
    }

    /// 送金先口座の識別子を返す。
    #[must_use]
    pub fn target_account_id(&self) -> NonZeroU64 {
        self.target_account.id()
    }

    /// 送金額を正の金額型として返す。
    #[must_use]
    pub fn amount(&self) -> NonZeroU64 {
        self.amount
    }

    /// 送金成立前に読み込んだ送金元残高を返す。
    #[must_use]
    pub fn expected_source_balance(&self) -> u64 {
        self.source_account.balance()
    }

    /// 送金成立前に読み込んだ送金先残高を返す。
    #[must_use]
    pub fn expected_target_balance(&self) -> u64 {
        self.target_account.balance()
    }

    /// 送金後に期待される送金元残高を返す。
    #[must_use]
    pub fn resulting_source_balance(&self) -> u64 {
        self.resulting_source_balance
    }

    /// 送金後に期待される送金先残高を返す。
    #[must_use]
    pub fn resulting_target_balance(&self) -> u64 {
        self.resulting_target_balance
    }
}
