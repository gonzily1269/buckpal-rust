//! ドメインルールを満たした口座間送金を表す。

use thiserror::Error;

use crate::{Account, Money};

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

/// 口座残高の事前状態とドメイン検証済みの送金内容。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transfer {
    source_account: Account,
    target_account: Account,
    amount: Money,
    resulting_source_balance: i64,
    resulting_target_balance: i64,
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
        amount: Money,
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
    pub fn source_account_id(&self) -> i64 {
        self.source_account.id()
    }

    /// 送金先口座の識別子を返す。
    #[must_use]
    pub fn target_account_id(&self) -> i64 {
        self.target_account.id()
    }

    /// 送金額を正の金額型として返す。
    #[must_use]
    pub fn amount(&self) -> Money {
        self.amount
    }

    /// 送金成立前に読み込んだ送金元残高を返す。
    #[must_use]
    pub fn expected_source_balance(&self) -> i64 {
        self.source_account.balance()
    }

    /// 送金成立前に読み込んだ送金先残高を返す。
    #[must_use]
    pub fn expected_target_balance(&self) -> i64 {
        self.target_account.balance()
    }

    /// 送金後に期待される送金元残高を返す。
    #[must_use]
    pub fn resulting_source_balance(&self) -> i64 {
        self.resulting_source_balance
    }

    /// 送金後に期待される送金先残高を返す。
    #[must_use]
    pub fn resulting_target_balance(&self) -> i64 {
        self.resulting_target_balance
    }
}

#[cfg(test)]
mod tests {
    use super::{Transfer, TransferError};
    use crate::{Account, Money};

    /// 送金前後の残高と送金額を成立済み送金へ保持することを確認する。
    #[test]
    fn creates_transfer_with_expected_and_resulting_balances() {
        type TransferDetails = (i64, i64, i64, i64, i64, i64, i64);
        let result = (|| -> Result<TransferDetails, String> {
            let source = Account::new(1, 1_000).map_err(|error| error.to_string())?;
            let target = Account::new(2, 500).map_err(|error| error.to_string())?;
            let amount = Money::new(300).map_err(|error| error.to_string())?;
            let transfer =
                Transfer::new(source, target, amount).map_err(|error| error.to_string())?;
            Ok((
                transfer.source_account_id(),
                transfer.target_account_id(),
                transfer.expected_source_balance(),
                transfer.expected_target_balance(),
                transfer.resulting_source_balance(),
                transfer.resulting_target_balance(),
                transfer.amount().amount(),
            ))
        })();

        assert_eq!(result, Ok((1, 2, 1_000, 500, 700, 800, 300)));
    }

    /// 同一口座、残高不足、残高超過を送金成立前に拒否することを確認する。
    #[test]
    fn rejects_transfers_that_violate_account_rules() {
        let result = (|| -> Result<(bool, bool, bool), String> {
            let source = Account::new(1, 100).map_err(|error| error.to_string())?;
            let target = Account::new(2, i64::MAX).map_err(|error| error.to_string())?;
            let one = Money::new(1).map_err(|error| error.to_string())?;
            let too_much = Money::new(101).map_err(|error| error.to_string())?;
            Ok((
                matches!(
                    Transfer::new(source.clone(), source.clone(), one),
                    Err(TransferError::SameAccount)
                ),
                matches!(
                    Transfer::new(source.clone(), target.clone(), too_much),
                    Err(TransferError::InsufficientFunds)
                ),
                matches!(
                    Transfer::new(source, target, one),
                    Err(TransferError::BalanceOverflow)
                ),
            ))
        })();

        assert_eq!(result, Ok((true, true, true)));
    }
}
