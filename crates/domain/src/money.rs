//! 正の整数単位で金額を表すドメイン型と、その生成時エラー。

use thiserror::Error;

/// 正の値だけを保持する金額型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Money(i64);

/// 金額の生成に失敗した理由。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MoneyError {
    #[error("amount must be greater than zero")]
    /// 金額がゼロ以下であり、送金額として扱えない。
    NotPositive,
}

impl Money {
    /// 正の整数から金額を生成する。
    ///
    /// ゼロ以下が指定された場合は [`MoneyError::NotPositive`] を返す。
    ///
    /// # Errors
    ///
    /// 金額がゼロ以下の場合は[`MoneyError::NotPositive`]を返す。
    pub fn new(amount: i64) -> Result<Self, MoneyError> {
        if amount <= 0 {
            return Err(MoneyError::NotPositive);
        }
        Ok(Self(amount))
    }

    /// 金額を整数の通貨単位で返す。
    #[must_use]
    pub fn amount(self) -> i64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Money, MoneyError};

    /// ゼロ以下の金額を拒否し、正の金額を保持することを確認する。
    #[test]
    fn money_must_be_positive() {
        assert_eq!(Money::new(0), Err(MoneyError::NotPositive));
        assert_eq!(Money::new(-1), Err(MoneyError::NotPositive));
        assert_eq!(Money::new(1).map(Money::amount), Ok(1));
    }
}
