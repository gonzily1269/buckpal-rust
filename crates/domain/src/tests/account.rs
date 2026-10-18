use std::num::NonZeroU64;

use crate::account::Account;

/// 残高を超える引き出しを拒否し、残高がゼロになる引き出しを許可する。
#[test]
fn account_withdrawal_cannot_overdraw() {
    let result = (|| -> Result<(bool, bool, bool, u64), String> {
        let mut account = Account::new(NonZeroU64::MIN, 10);
        Ok((
            account.withdraw(NonZeroU64::new(11).ok_or("invalid amount")?),
            account.withdraw(NonZeroU64::new(10).ok_or("invalid amount")?),
            account.withdraw(NonZeroU64::new(1).ok_or("invalid amount")?),
            account.balance(),
        ))
    })();

    assert_eq!(result, Ok((false, true, false, 0)));
}

/// 入金後の残高が、その全額を引き出せる値になっていることを確認する。
#[test]
fn account_deposit_updates_balance() {
    let result = (|| -> Result<(Option<()>, bool, bool, u64), String> {
        let mut account = Account::new(NonZeroU64::MIN, 10);
        let amount = NonZeroU64::new(5).ok_or("invalid amount")?;
        let deposited = account.deposit(amount);
        let full_balance = NonZeroU64::new(15).ok_or("invalid id")?;
        let extra_amount = NonZeroU64::new(1).ok_or("invalid amount")?;
        Ok((
            deposited,
            account.withdraw(full_balance),
            account.withdraw(extra_amount),
            account.balance(),
        ))
    })();

    assert_eq!(result, Ok((Some(()), true, false, 0)));
}
