//! ダッシュボード取得ユースケースの実装。

use std::sync::Arc;

use async_trait::async_trait;

use crate::{Dashboard, TransferRepository};

use super::{get_dashboard::GetDashboardUseCase, get_dashboard_error::GetDashboardError};

#[cfg(test)]
#[path = "tests/use_case.rs"]
mod tests;

/// ダッシュボード取得ユースケースの実装。
pub struct GetDashboardService {
    /// ダッシュボード情報の読み込みを委譲するリポジトリ。
    repository: Arc<dyn TransferRepository>,
}

#[async_trait]
impl GetDashboardUseCase for GetDashboardService {
    /// リポジトリからダッシュボード情報を読み込み、呼び出し元へ返す。
    async fn get_dashboard(&self) -> Result<Dashboard, GetDashboardError> {
        self.repository
            .load_dashboard()
            .await
            .map_err(|_| GetDashboardError::Unavailable)
    }
}

impl GetDashboardService {
    /// 指定されたリポジトリを利用するダッシュボード取得サービスを生成する。
    pub fn new(repository: Arc<dyn TransferRepository>) -> Self {
        Self { repository }
    }
}
