//! ダッシュボード取得の入力ポートとユースケース。

use std::sync::Arc;

use async_trait::async_trait;

use thiserror::Error;

use super::{Dashboard, DashboardQueryFailure, LoadDashboardPort};

/// ダッシュボード取得ユースケースが返すアプリケーションエラー。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GetDashboardError {
    #[error("dashboard data could not be retrieved")]
    /// ダッシュボード出力ポートからデータを取得できなかった。
    Unavailable,
}

/// 入力アダプターがダッシュボード取得を呼び出すための入力ポート。
#[async_trait]
pub trait GetDashboardUseCase: Send + Sync {
    /// 画面表示に必要な口座残高と最近の送金履歴を取得する。
    async fn get_dashboard(&self) -> Result<Dashboard, GetDashboardError>;
}

/// ダッシュボード取得入力ポートのユースケース実装。
pub struct DashboardService {
    /// ダッシュボードの読み込みを委譲する出力ポート。
    dashboard_port: Arc<dyn LoadDashboardPort>,
}

impl DashboardService {
    /// 指定された出力ポートを利用するダッシュボード取得サービスを生成する。
    pub fn new(dashboard_port: Arc<dyn LoadDashboardPort>) -> Self {
        Self { dashboard_port }
    }
}

#[async_trait]
impl GetDashboardUseCase for DashboardService {
    /// 出力ポートからダッシュボード情報を読み込み、入力ポートの結果として返す。
    async fn get_dashboard(&self) -> Result<Dashboard, GetDashboardError> {
        self.dashboard_port
            .load_dashboard()
            .await
            .map_err(|DashboardQueryFailure| GetDashboardError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::{DashboardService, GetDashboardError, GetDashboardUseCase};
    use crate::{Dashboard, DashboardQueryFailure, LoadDashboardPort};

    struct FailingDashboardPort;

    #[async_trait]
    impl LoadDashboardPort for FailingDashboardPort {
        async fn load_dashboard(&self) -> Result<Dashboard, DashboardQueryFailure> {
            Err(DashboardQueryFailure)
        }
    }

    /// 出力ポートの境界エラーがユースケース固有のエラーへ変換されることを確認する。
    #[tokio::test]
    async fn translates_output_port_failure() {
        let service = DashboardService::new(Arc::new(FailingDashboardPort));
        assert_eq!(
            service.get_dashboard().await,
            Err(GetDashboardError::Unavailable)
        );
    }
}
