//! ダッシュボード取得ユースケースの契約。

use async_trait::async_trait;

use crate::Dashboard;

use super::get_dashboard_error::GetDashboardError;

/// 入力アダプターがダッシュボード取得を呼び出すための契約。
#[async_trait]
pub trait GetDashboardUseCase: Send + Sync {
    /// 画面表示に必要な口座残高と最近の送金履歴を取得する。
    async fn get_dashboard(&self) -> Result<Dashboard, GetDashboardError>;
}
