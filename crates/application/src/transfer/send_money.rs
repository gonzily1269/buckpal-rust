//! 送金ユースケースの契約。

use std::num::NonZeroU64;

use async_trait::async_trait;

use super::error::SendMoneyError;

/// 入力アダプターが送金ユースケースを呼び出すための契約。
#[async_trait]
pub trait SendMoneyUseCase: Send + Sync {
    /// 2口座間の送金を試み、成功または業務・永続化エラーを返す。
    async fn send_money(
        &self,
        source_account_id: NonZeroU64,
        target_account_id: NonZeroU64,
        amount: NonZeroU64,
    ) -> Result<(), SendMoneyError>;
}
