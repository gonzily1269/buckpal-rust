use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

use buckpal_domain::{Money, MoneyError, Transfer, TransferError};

use super::{LoadTransferAccountsPort, RecordTransferPort, TransferPortError};

/// 送金上限の確認と送金ポートへの委譲を行うユースケース。
#[derive(Clone)]
pub struct SendMoneyService {
    /// 送金対象の口座を読み込む出力ポート。
    accounts_port: Arc<dyn LoadTransferAccountsPort>,
    /// ドメインルール適用後の送金を記録する出力ポート。
    transfer_port: Arc<dyn RecordTransferPort>,
    /// 1回の送金で許可される最大金額。
    maximum_transfer: i64,
}

/// 送金ユースケースが返す入力検証・業務・永続化エラー。
#[derive(Debug, Error)]
pub enum SendMoneyError {
    #[error("amount must be greater than zero")]
    /// 金額が正の整数ではない。
    InvalidAmount,
    #[error("source and target accounts must be different")]
    /// 送金元と送金先に同じ口座が指定された。
    SameAccount,
    #[error("transfer exceeds the configured limit")]
    /// 送金額が設定された上限を超えている。
    ThresholdExceeded,
    #[error("account does not exist")]
    /// 送金元または送金先の口座が存在しない。
    AccountNotFound,
    #[error("source account has insufficient funds")]
    /// 送金元口座の残高が送金額に満たない。
    InsufficientFunds,
    #[error("target account balance exceeds the supported range")]
    /// 入金によって送金先残高が整数の上限を超える。
    BalanceOverflow,
    #[error("account balances changed before the transfer was recorded")]
    /// 口座残高が読込後に変化し、送金を確定できなかった。
    ConcurrentModification,
    #[error("transfer could not be completed because storage failed")]
    /// 口座データの読込または送金記録に失敗した。
    PersistenceFailure,
}

/// 入力アダプターが送金ユースケースを呼び出すための入力ポート。
#[async_trait]
pub trait SendMoneyUseCase: Send + Sync {
    /// 2口座間の送金を試み、成功または業務・ポートエラーを返す。
    async fn send_money(
        &self,
        source_account_id: i64,
        target_account_id: i64,
        amount: i64,
    ) -> Result<(), SendMoneyError>;
}

impl SendMoneyService {
    /// 口座読込・送金記録ポートと1回あたりの送金上限からユースケースを構築する。
    pub fn new(
        accounts_port: Arc<dyn LoadTransferAccountsPort>,
        transfer_port: Arc<dyn RecordTransferPort>,
        maximum_transfer: i64,
    ) -> Self {
        Self {
            accounts_port,
            transfer_port,
            maximum_transfer,
        }
    }

    /// 口座間の送金をドメインモデルで判断し、出力ポートへ記録を依頼する。
    ///
    /// 入力と上限を検証し、口座モデルに残高ルールを適用してから送金を記録する。
    ///
    /// # Errors
    ///
    /// 入力検証、送金ルール、口座読込、または送金記録に失敗した場合にエラーを返す。
    pub async fn send_money(
        &self,
        source_account_id: i64,
        target_account_id: i64,
        amount: i64,
    ) -> Result<(), SendMoneyError> {
        if source_account_id <= 0 || target_account_id <= 0 {
            return Err(SendMoneyError::AccountNotFound);
        }
        if source_account_id == target_account_id {
            return Err(SendMoneyError::SameAccount);
        }
        let amount =
            Money::new(amount).map_err(|MoneyError::NotPositive| SendMoneyError::InvalidAmount)?;
        if amount.amount() > self.maximum_transfer {
            return Err(SendMoneyError::ThresholdExceeded);
        }

        let (source, target) = self
            .accounts_port
            .load_accounts(source_account_id, target_account_id)
            .await
            .map_err(|error| map_transfer_port_error(&error))?
            .ok_or(SendMoneyError::AccountNotFound)?;
        let transfer = Transfer::new(source, target, amount).map_err(|error| match error {
            TransferError::SameAccount => SendMoneyError::SameAccount,
            TransferError::InsufficientFunds => SendMoneyError::InsufficientFunds,
            TransferError::BalanceOverflow => SendMoneyError::BalanceOverflow,
        })?;

        self.transfer_port
            .record_transfer(transfer)
            .await
            .map_err(|error| map_transfer_port_error(&error))?;
        Ok(())
    }
}

/// 出力ポートの技術境界エラーを、ユースケースの結果へ変換する。
fn map_transfer_port_error(error: &TransferPortError) -> SendMoneyError {
    match error {
        TransferPortError::ConcurrentModification => SendMoneyError::ConcurrentModification,
        TransferPortError::LoadFailed | TransferPortError::RecordFailed => {
            SendMoneyError::PersistenceFailure
        }
    }
}

#[async_trait]
impl SendMoneyUseCase for SendMoneyService {
    /// ユースケースtraitの呼び出しをサービス実装へ委譲する。
    async fn send_money(
        &self,
        source_account_id: i64,
        target_account_id: i64,
        amount: i64,
    ) -> Result<(), SendMoneyError> {
        SendMoneyService::send_money(self, source_account_id, target_account_id, amount).await
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        sync::{Arc, Mutex},
    };

    use async_trait::async_trait;

    use buckpal_domain::{Account, AccountError, Transfer};

    use super::super::{LoadTransferAccountsPort, RecordTransferPort, TransferPortError};
    use super::{SendMoneyError, SendMoneyService, map_transfer_port_error};

    #[derive(Default)]
    /// 送金ユースケーステスト用に口座状態と送金記録を保持するfake。
    struct TransferFake {
        accounts: Mutex<BTreeMap<i64, Account>>,
        transfers: Mutex<Vec<(i64, i64, i64)>>,
    }

    /// 永続化アダプターのエラーをユースケースの結果へ隠蔽することを確認する。
    #[test]
    fn translates_output_port_errors_to_use_case_errors() {
        assert!(matches!(
            map_transfer_port_error(&TransferPortError::ConcurrentModification),
            SendMoneyError::ConcurrentModification
        ));
        assert!(matches!(
            map_transfer_port_error(&TransferPortError::LoadFailed),
            SendMoneyError::PersistenceFailure
        ));
        assert!(matches!(
            map_transfer_port_error(&TransferPortError::RecordFailed),
            SendMoneyError::PersistenceFailure
        ));
    }

    impl TransferFake {
        /// テスト用の初期残高を持つ口座fakeを生成する。
        fn with_accounts() -> Result<Self, AccountError> {
            Ok(Self {
                accounts: Mutex::new(BTreeMap::from([
                    (1, Account::new(1, 1_000)?),
                    (2, Account::new(2, 500)?),
                ])),
                transfers: Mutex::new(Vec::new()),
            })
        }

        /// Mutex poisoningを抽象化された口座読込エラーへ変換する。
        fn load_error<T>(_: std::sync::PoisonError<T>) -> TransferPortError {
            TransferPortError::LoadFailed
        }
    }

    #[async_trait]
    impl LoadTransferAccountsPort for TransferFake {
        /// fakeから指定口座を読み込み、口座がない場合はNoneを返す。
        async fn load_accounts(
            &self,
            source_account_id: i64,
            target_account_id: i64,
        ) -> Result<Option<(Account, Account)>, TransferPortError> {
            let accounts = self.accounts.lock().map_err(Self::load_error)?;
            let Some(source) = accounts.get(&source_account_id).cloned() else {
                return Ok(None);
            };
            let Some(target) = accounts.get(&target_account_id).cloned() else {
                return Ok(None);
            };
            Ok(Some((source, target)))
        }
    }

    #[async_trait]
    impl RecordTransferPort for TransferFake {
        /// 残高競合を検査し、成立済み送金をfakeの口座と記録へ反映する。
        async fn record_transfer(&self, transfer: Transfer) -> Result<(), TransferPortError> {
            let mut accounts = self
                .accounts
                .lock()
                .map_err(|_| TransferPortError::RecordFailed)?;
            let source_account_id = transfer.source_account_id();
            let target_account_id = transfer.target_account_id();
            let Some(source) = accounts.get(&source_account_id) else {
                return Err(TransferPortError::ConcurrentModification);
            };
            if source.balance() != transfer.expected_source_balance() {
                return Err(TransferPortError::ConcurrentModification);
            }
            let Some(target) = accounts.get(&target_account_id) else {
                return Err(TransferPortError::ConcurrentModification);
            };
            if target.balance() != transfer.expected_target_balance() {
                return Err(TransferPortError::ConcurrentModification);
            }
            let source = Account::new(source_account_id, transfer.resulting_source_balance())
                .map_err(|_| TransferPortError::RecordFailed)?;
            let target = Account::new(target_account_id, transfer.resulting_target_balance())
                .map_err(|_| TransferPortError::RecordFailed)?;
            accounts.insert(source_account_id, source);
            accounts.insert(target_account_id, target);
            drop(accounts);
            self.transfers
                .lock()
                .map_err(|_| TransferPortError::RecordFailed)?
                .push((
                    source_account_id,
                    target_account_id,
                    transfer.amount().amount(),
                ));
            Ok(())
        }
    }

    /// 不正入力では出力ポートを呼ばず、有効入力のみ記録することを確認する。
    #[tokio::test]
    async fn validates_input_before_calling_output_port() {
        let result = async {
            let port = Arc::new(TransferFake::with_accounts().map_err(|error| error.to_string())?);
            let accounts_port: Arc<dyn LoadTransferAccountsPort> = port.clone();
            let transfer_port: Arc<dyn RecordTransferPort> = port.clone();
            let service = SendMoneyService::new(accounts_port, transfer_port, 1_000);

            let same_account = matches!(
                service.send_money(1, 1, 100).await,
                Err(SendMoneyError::SameAccount)
            );
            let invalid_amount = matches!(
                service.send_money(1, 2, 0).await,
                Err(SendMoneyError::InvalidAmount)
            );
            let over_threshold = matches!(
                service.send_money(1, 2, 1_001).await,
                Err(SendMoneyError::ThresholdExceeded)
            );
            let no_invalid_transfers = port
                .transfers
                .lock()
                .map_err(|error| error.to_string())?
                .is_empty();

            let successful_transfer = service.send_money(1, 2, 500).await.is_ok();
            let recorded = port
                .transfers
                .lock()
                .map_err(|error| error.to_string())?
                .as_slice()
                == [(1, 2, 500)];
            Ok::<_, String>((
                same_account,
                invalid_amount,
                over_threshold,
                no_invalid_transfers,
                successful_transfer,
                recorded,
            ))
        }
        .await;

        assert_eq!(result, Ok((true, true, true, true, true, true)));
    }

    /// 残高不足と不存在口座をドメイン・ユースケースで拒否することを確認する。
    #[tokio::test]
    async fn applies_account_rules_before_recording() {
        let result = async {
            let port = Arc::new(TransferFake::with_accounts().map_err(|error| error.to_string())?);
            let accounts_port: Arc<dyn LoadTransferAccountsPort> = port.clone();
            let transfer_port: Arc<dyn RecordTransferPort> = port.clone();
            let service = SendMoneyService::new(accounts_port, transfer_port, 2_000);

            let insufficient_funds = matches!(
                service.send_money(1, 2, 1_001).await,
                Err(SendMoneyError::InsufficientFunds)
            );
            let missing_account = matches!(
                service.send_money(1, 3, 100).await,
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
}
