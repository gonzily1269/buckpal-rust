//! ダッシュボード取得の契約とユースケース。

use std::sync::Arc;

use async_trait::async_trait;

use crate::{Dashboard, TransferRepository};

use super::error::GetDashboardError;

#[cfg(test)]
#[path = "tests/use_case.rs"]
mod tests;

/// 入力アダプターがダッシュボード取得を呼び出すための契約。
#[async_trait]
pub trait GetDashboardUseCase: Send + Sync {
    /// 画面表示に必要な口座残高と最近の送金履歴を取得する。
    async fn get_dashboard(&self) -> Result<Dashboard, GetDashboardError>;
}

/// ダッシュボード取得ユースケースの実装。
pub struct DashboardService {
    /// ダッシュボード情報の読み込みを委譲するリポジトリ。
    repository: Arc<dyn TransferRepository>,
}

impl DashboardService {
    /// 指定されたリポジトリを利用するダッシュボード取得サービスを生成する。
    pub fn new(repository: Arc<dyn TransferRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl GetDashboardUseCase for DashboardService {
    /// リポジトリからダッシュボード情報を読み込み、呼び出し元へ返す。
    async fn get_dashboard(&self) -> Result<Dashboard, GetDashboardError> {
        self.repository
            .load_dashboard()
            .await
            .map_err(|_| GetDashboardError::Unavailable)
    }
}
