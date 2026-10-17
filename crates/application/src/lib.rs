//! ユースケースと、外部アダプターが実装するポート。
//!
//! この層は具体的なデータベースやWebフレームワークではなく、traitに依存する。

mod dashboard_port;
mod get_dashboard;
mod send_money;
mod transfer_port;

pub use dashboard_port::{
    Dashboard, DashboardAccount, DashboardActivity, DashboardQueryFailure, LoadDashboardPort,
};
pub use get_dashboard::{DashboardService, GetDashboardError, GetDashboardUseCase};
pub use send_money::{SendMoneyError, SendMoneyService, SendMoneyUseCase};
pub use transfer_port::{LoadTransferAccountsPort, RecordTransferPort, TransferPortError};
