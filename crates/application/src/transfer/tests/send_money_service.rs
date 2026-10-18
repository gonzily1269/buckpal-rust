use std::{
    collections::BTreeMap,
    num::NonZeroU64,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;

use buckpal_domain::{Account, Transfer, TransferError};

use super::super::error::{SendMoneyError, TransferRepositoryError};
use super::super::repository::TransferRepository;
use super::{SendMoneyService, map_repository_error};

#[derive(Default)]
/// 送金ユースケーステスト用に口座状態と送金記録を保持するfake。
struct TransferFake {
    accounts: Mutex<BTreeMap<NonZeroU64, Account>>,
    transfers: Mutex<Vec<(NonZeroU64, NonZeroU64, u64)>>,
}

fn account_id(id: u64) -> NonZeroU64 {
    NonZeroU64::new(id).unwrap_or(NonZeroU64::MIN)
}

fn amount(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap_or(NonZeroU64::MIN)
}

/// 永続化アダプターのエラーをユースケースの結果へ隠蔽することを確認する。
#[test]
fn translates_repository_errors_to_use_case_errors() {
    assert!(matches!(
        map_repository_error(&TransferRepositoryError::LoadFailed),
        SendMoneyError::PersistenceFailure
    ));
    assert!(matches!(
        map_repository_error(&TransferRepositoryError::RecordFailed),
        SendMoneyError::PersistenceFailure
    ));
}

impl TransferFake {
    /// テスト用の初期残高を持つ口座fakeを生成する。
    fn with_accounts() -> Self {
        Self {
            accounts: Mutex::new(BTreeMap::from([
                (NonZeroU64::MIN, Account::new(NonZeroU64::MIN, 1_000)),
                (account_id(2), Account::new(account_id(2), 500)),
            ])),
            transfers: Mutex::new(Vec::new()),
        }
    }

    /// Mutex poisoningを抽象化された永続化エラーへ変換する。
    fn load_error<T>(_: std::sync::PoisonError<T>) -> TransferRepositoryError {
        TransferRepositoryError::RecordFailed
    }
}

#[async_trait]
impl TransferRepository for TransferFake {
    async fn load_dashboard(&self) -> Result<crate::Dashboard, TransferRepositoryError> {
        Err(TransferRepositoryError::DashboardLoadFailed)
    }

    /// fakeで口座をロックし、ドメイン検証済みの送金を記録する。
    async fn transfer(
        &self,
        source_account_id: NonZeroU64,
        target_account_id: NonZeroU64,
        amount: NonZeroU64,
    ) -> Result<(), TransferRepositoryError> {
        let mut accounts = self.accounts.lock().map_err(Self::load_error)?;
        let Some(source) = accounts.get(&source_account_id).cloned() else {
            return Err(TransferRepositoryError::AccountNotFound);
        };
        let Some(target) = accounts.get(&target_account_id).cloned() else {
            return Err(TransferRepositoryError::AccountNotFound);
        };
        let transfer = Transfer::new(source, target, amount).map_err(|error| match error {
            TransferError::SameAccount => TransferRepositoryError::AccountNotFound,
            TransferError::InsufficientFunds => TransferRepositoryError::InsufficientFunds,
            TransferError::BalanceOverflow => TransferRepositoryError::BalanceOverflow,
        })?;
        let source = Account::new(source_account_id, transfer.resulting_source_balance());
        let target = Account::new(target_account_id, transfer.resulting_target_balance());
        accounts.insert(source_account_id, source);
        accounts.insert(target_account_id, target);
        drop(accounts);
        self.transfers
            .lock()
            .map_err(|_| TransferRepositoryError::RecordFailed)?
            .push((source_account_id, target_account_id, amount.get()));
        Ok(())
    }
}

/// 不正入力ではリポジトリを呼ばず、有効入力のみ記録することを確認する。
#[tokio::test]
async fn validates_input_before_calling_repository() {
    let result = async {
        let port = Arc::new(TransferFake::with_accounts());
        let repository: Arc<dyn TransferRepository> = port.clone();
        let service = SendMoneyService::new(repository, 1_000);

        let same_account = matches!(
            service
                .send_money(NonZeroU64::MIN, NonZeroU64::MIN, amount(100))
                .await,
            Err(SendMoneyError::SameAccount)
        );
        let over_threshold = matches!(
            service
                .send_money(NonZeroU64::MIN, account_id(2), amount(1_001))
                .await,
            Err(SendMoneyError::ThresholdExceeded)
        );
        let no_invalid_transfers = port
            .transfers
            .lock()
            .map_err(|error| error.to_string())?
            .is_empty();

        let successful_transfer = service
            .send_money(NonZeroU64::MIN, account_id(2), amount(500))
            .await
            .is_ok();
        let recorded = port
            .transfers
            .lock()
            .map_err(|error| error.to_string())?
            .as_slice()
            == [(NonZeroU64::MIN, account_id(2), 500)];
        Ok::<_, String>((
            same_account,
            over_threshold,
            no_invalid_transfers,
            successful_transfer,
            recorded,
        ))
    }
    .await;

    assert_eq!(result, Ok((true, true, true, true, true)));
}

/// 残高不足と不存在口座をドメイン・ユースケースで拒否することを確認する。
#[tokio::test]
async fn applies_account_rules_before_recording() {
    let result = async {
        let port = Arc::new(TransferFake::with_accounts());
        let repository: Arc<dyn TransferRepository> = port.clone();
        let service = SendMoneyService::new(repository, 2_000);

        let insufficient_funds = matches!(
            service
                .send_money(NonZeroU64::MIN, account_id(2), amount(1_001))
                .await,
            Err(SendMoneyError::InsufficientFunds)
        );
        let missing_account = matches!(
            service
                .send_money(NonZeroU64::MIN, account_id(3), amount(100))
                .await,
            Err(SendMoneyError::AccountNotFound)
        );
        let no_transfers = port
            .transfers
            .lock()
            .map_err(|error| error.to_string())?
            .is_empty();
        Ok::<_, String>((insufficient_funds, missing_account, no_transfers))
    }
    .await;

    assert_eq!(result, Ok((true, true, true)));
}
