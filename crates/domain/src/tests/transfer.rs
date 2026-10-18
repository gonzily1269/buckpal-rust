use std::num::NonZeroU64;

use crate::account::Account;
use crate::transfer::{Transfer, TransferError};

/// 送金前後の残高と送金額を成立済み送金へ保持することを確認する。
#[test]
fn creates_transfer_with_expected_and_resulting_balances() {
    type TransferDetails = (NonZeroU64, NonZeroU64, u64, u64, u64, u64, u64);
    let result = (|| -> Result<TransferDetails, String> {
        let source = Account::new(NonZeroU64::MIN, 1_000);
        let target = Account::new(NonZeroU64::new(2).ok_or("invalid id")?, 500);
        let amount = NonZeroU64::new(300).ok_or("invalid amount")?;
        let transfer = Transfer::new(source, target, amount).map_err(|error| error.to_string())?;
        Ok((
            transfer.source_account_id(),
            transfer.target_account_id(),
            transfer.expected_source_balance(),
            transfer.expected_target_balance(),
            transfer.resulting_source_balance(),
            transfer.resulting_target_balance(),
            transfer.amount().get(),
        ))
    })();

    assert_eq!(
        result,
        Ok((
            NonZeroU64::MIN,
            NonZeroU64::new(2)
                .ok_or("invalid id")
                .unwrap_or(NonZeroU64::MIN),
            1_000,
            500,
            700,
            800,
            300
        ))
    );
}

/// 同一口座、残高不足、残高超過を送金成立前に拒否することを確認する。
#[test]
fn rejects_transfers_that_violate_account_rules() {
    let result = (|| -> Result<(bool, bool, bool), String> {
        let source = Account::new(NonZeroU64::MIN, 100);
        let target = Account::new(NonZeroU64::new(2).ok_or("invalid id")?, u64::MAX);
        let one = NonZeroU64::new(1).ok_or("invalid amount")?;
        let too_much = NonZeroU64::new(101).ok_or("invalid amount")?;
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
