//! ユースケース、アプリケーションDTO、外部アダプターが実装する契約。
//!
//! この層は具体的なデータベースやWebフレームワークではなく、リポジトリ契約に依存する。

mod dashboard;
mod transfer;

pub use dashboard::{
    Dashboard, DashboardAccount, DashboardActivity, DashboardService, GetDashboardError,
    GetDashboardUseCase,
};
pub use transfer::{
    SendMoneyError, SendMoneyService, SendMoneyUseCase, TransferRepository, TransferRepositoryError,
};
