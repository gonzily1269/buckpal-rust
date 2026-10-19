use std::{num::NonZeroU64, sync::Arc};

use async_trait::async_trait;

use super::send_money::SendMoneyUseCase;
use super::send_money_error::SendMoneyError;
use crate::{TransferRepository, TransferRepositoryError};

#[cfg(test)]
#[path = "tests/send_money_service.rs"]
mod tests;

/// 送金上限の確認とリポジトリへの委譲を行うユースケース実装。
#[derive(Clone)]
pub struct SendMoneyService {
    /// 送金に必要な口座の読み込みと送金記録を担うリポジトリ。
    repository: Arc<dyn TransferRepository>,
    /// 1回の送金で許可される最大金額。
    maximum_transfer: u64,
}

#[async_trait]
impl SendMoneyUseCase for SendMoneyService {
    /// ユースケースtraitの呼び出しをサービス実装へ委譲する。
    async fn send_money(
        &self,
        source_account_id: NonZeroU64,
        target_account_id: NonZeroU64,
        amount: NonZeroU64,
    ) -> Result<(), SendMoneyError> {
        SendMoneyService::send_money(self, source_account_id, target_account_id, amount).await
    }
}

impl SendMoneyService {
    /// 送金リポジトリと1回あたりの送金上限からユースケースを構築する。
    pub fn new(repository: Arc<dyn TransferRepository>, maximum_transfer: u64) -> Self {
        Self {
            repository,
            maximum_transfer,
        }
    }

    /// 口座間の送金をドメインモデルで判断し、リポジトリへ記録を依頼する。
    ///
    /// 入力と上限を検証し、口座モデルに残高ルールを適用してから送金を記録する。
    ///
    /// # Errors
    ///
    /// 入力検証、送金ルール、または永続化に失敗した場合にエラーを返す。
    pub async fn send_money(
        &self,
        source_account_id: NonZeroU64,
        target_account_id: NonZeroU64,
        amount: NonZeroU64,
    ) -> Result<(), SendMoneyError> {
        if source_account_id == target_account_id {
            return Err(SendMoneyError::SameAccount);
        }

        if amount.get() > self.maximum_transfer {
            return Err(SendMoneyError::ThresholdExceeded);
        }

        self.repository
            .transfer(source_account_id, target_account_id, amount)
            .await
            .map_err(|error| map_repository_error(&error))?;
        Ok(())
    }
}

/// リポジトリの技術エラーをユースケースの結果へ変換する。
fn map_repository_error(error: &TransferRepositoryError) -> SendMoneyError {
    match error {
        TransferRepositoryError::AccountNotFound => SendMoneyError::AccountNotFound,
        TransferRepositoryError::InsufficientFunds => SendMoneyError::InsufficientFunds,
        TransferRepositoryError::BalanceOverflow => SendMoneyError::BalanceOverflow,
        TransferRepositoryError::DashboardLoadFailed
        | TransferRepositoryError::LoadFailed
        | TransferRepositoryError::RecordFailed => SendMoneyError::PersistenceFailure,
    }
}
