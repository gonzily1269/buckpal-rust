//! 送金とダッシュボード読み取りに必要なリポジトリ契約。

use async_trait::async_trait;
use std::num::NonZeroU64;

use crate::Dashboard;

use super::repository_error::TransferRepositoryError;

/// 送金に必要な口座の読み込みと、送金記録を担うリポジトリ。
#[async_trait]
pub trait TransferRepository: Send + Sync {
    /// 口座残高と最近の送金履歴を読み込む。
    async fn load_dashboard(&self) -> Result<Dashboard, TransferRepositoryError>;

    /// 口座をロックして残高を読み、送金を一括して記録する。
    async fn transfer(
        &self,
        source_account_id: NonZeroU64,
        target_account_id: NonZeroU64,
        amount: NonZeroU64,
    ) -> Result<(), TransferRepositoryError>;
}
