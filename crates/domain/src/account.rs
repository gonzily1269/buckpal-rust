use std::num::NonZeroU64;

/// 口座IDと残高を保持し、入出金ルールを適用するドメインモデル。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// 口座を識別する一意なID。
    id: NonZeroU64,
    /// 現在の残高。整数の通貨単位で保持する。
    balance: u64,
}

impl Account {
    /// 正のIDと非負の初期残高から口座を生成する。
    #[must_use]
    pub fn new(id: NonZeroU64, balance: u64) -> Self {
        Self { id, balance }
    }

    /// この口座のIDを返す。
    #[must_use]
    pub fn id(&self) -> NonZeroU64 {
        self.id
    }

    /// この口座の残高を整数単位で返す。
    #[must_use]
    pub fn balance(&self) -> u64 {
        self.balance
    }

    /// 残高不足でなければ指定額を引き出し、成功したかを返す。
    ///
    /// 残高不足の場合は口座の状態を変更しない。
    pub fn withdraw(&mut self, amount: NonZeroU64) -> bool {
        let amount = amount.get();
        if self.balance < amount {
            return false;
        }
        self.balance -= amount;
        true
    }

    /// 残高に指定額を加算する。
    ///
    /// `u64` の上限を超える場合は `None` を返し、残高を変更しない。
    pub fn deposit(&mut self, amount: NonZeroU64) -> Option<()> {
        // `checked_add` はオーバーフロー時に `None` を返す。
        self.balance = self.balance.checked_add(amount.get())?;
        Some(())
    }
}
