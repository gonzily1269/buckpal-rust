//! 外部技術に依存しない、口座・金額のドメインモデル。

mod account;
mod money;
mod transfer;

pub use account::{Account, AccountError};
pub use money::{Money, MoneyError};
pub use transfer::{Transfer, TransferError};
