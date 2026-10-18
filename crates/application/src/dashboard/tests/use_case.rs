use std::sync::Arc;

use async_trait::async_trait;

use super::super::super::{
    Dashboard, GetDashboardError, GetDashboardService, GetDashboardUseCase, TransferRepository,
    TransferRepositoryError,
};

struct FailingDashboardRepository;

#[async_trait]
impl TransferRepository for FailingDashboardRepository {
    async fn load_dashboard(&self) -> Result<Dashboard, TransferRepositoryError> {
        Err(TransferRepositoryError::DashboardLoadFailed)
    }

    async fn transfer(
        &self,
        _source_account_id: std::num::NonZeroU64,
        _target_account_id: std::num::NonZeroU64,
        _amount: std::num::NonZeroU64,
    ) -> Result<(), TransferRepositoryError> {
        Err(TransferRepositoryError::DashboardLoadFailed)
    }
}

/// リポジトリのエラーがユースケース固有のエラーへ変換されることを確認する。
#[tokio::test]
async fn translates_repository_failure() {
    let service = GetDashboardService::new(Arc::new(FailingDashboardRepository));
    assert_eq!(
        service.get_dashboard().await,
        Err(GetDashboardError::Unavailable)
    );
}
