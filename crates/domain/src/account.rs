use thiserror::Error;

use crate::Money;

/// 口座の生成に失敗した理由。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AccountError {
    #[error("account id must be greater than zero")]
    /// 口座IDに0以下が指定された。
    InvalidId,
    #[error("account balance cannot be negative")]
    /// 初期残高に負の値が指定された。
    NegativeBalance,
}

/// 口座IDと残高を保持し、入出金ルールを適用するドメインモデル。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// 口座を識別する一意なID。
    id: i64,
    /// 現在の残高。整数の通貨単位で保持する。
    balance: i64,
}

impl Account {
    /// 正のIDと非負の初期残高から口座を生成する。
    ///
    /// # Errors
    ///
    /// IDが正でない場合、または初期残高が負の場合はエラーを返す。
    pub fn new(id: i64, balance: i64) -> Result<Self, AccountError> {
        if id <= 0 {
            return Err(AccountError::InvalidId);
        }
        if balance < 0 {
            return Err(AccountError::NegativeBalance);
        }
        Ok(Self { id, balance })
    }

    /// この口座のIDを返す。
    #[must_use]
    pub fn id(&self) -> i64 {
        self.id
    }

    /// この口座の残高を整数単位で返す。
    #[must_use]
    pub fn balance(&self) -> i64 {
        self.balance
    }

    /// 残高不足でなければ指定額を引き出し、成功したかを返す。
    ///
    /// 残高不足の場合は口座の状態を変更しない。
    pub fn withdraw(&mut self, amount: Money) -> bool {
        if self.balance < amount.amount() {
            return false;
        }
        self.balance -= amount.amount();
        true
    }

    /// 残高に指定額を加算する。
    ///
    /// `i64` の上限を超える場合は `None` を返し、残高を変更しない。
    pub fn deposit(&mut self, amount: Money) -> Option<()> {
        self.balance = self.balance.checked_add(amount.amount())?;
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Money;

    use super::Account;

    /// 残高を超える引き出しを拒否し、残高がゼロになる引き出しを許可する。
    #[test]
    fn account_withdrawal_cannot_overdraw() {
        let result = (|| -> Result<(bool, bool, bool, i64), String> {
            let mut account = Account::new(1, 10).map_err(|error| error.to_string())?;
            Ok((
                account.withdraw(Money::new(11).map_err(|error| error.to_string())?),
                account.withdraw(Money::new(10).map_err(|error| error.to_string())?),
                account.withdraw(Money::new(1).map_err(|error| error.to_string())?),
                account.balance(),
            ))
        })();

        assert_eq!(result, Ok((false, true, false, 0)));
    }

    /// 入金後の残高が、その全額を引き出せる値になっていることを確認する。
    #[test]
    fn account_deposit_updates_balance() {
        let result = (|| -> Result<(Option<()>, bool, bool, i64), String> {
            let mut account = Account::new(1, 10).map_err(|error| error.to_string())?;
            let amount = Money::new(5).map_err(|error| error.to_string())?;
            let deposited = account.deposit(amount);
            let full_balance = Money::new(15).map_err(|error| error.to_string())?;
            let extra_amount = Money::new(1).map_err(|error| error.to_string())?;
            Ok((
                deposited,
                account.withdraw(full_balance),
                account.withdraw(extra_amount),
                account.balance(),
            ))
        })();

        assert_eq!(result, Ok((Some(()), true, false, 0)));
    }

    /// 口座IDが正数、初期残高が非負という生成条件を検証する。
    #[test]
    fn rejects_invalid_account_state() {
        assert_eq!(Account::new(0, 10), Err(super::AccountError::InvalidId));
        assert_eq!(Account::new(-1, 10), Err(super::AccountError::InvalidId));
        assert_eq!(
            Account::new(1, -1),
            Err(super::AccountError::NegativeBalance)
        );
    }
}
