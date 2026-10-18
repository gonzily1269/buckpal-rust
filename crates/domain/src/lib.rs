//! 外部技術に依存しない、口座・金額のドメインモデル。

mod account;
mod transfer;

pub use account::Account;
pub use transfer::{Transfer, TransferError};

#[cfg(test)]
#[path = "tests/account.rs"]
mod account_tests;

#[cfg(test)]
#[path = "tests/transfer.rs"]
mod transfer_tests;
