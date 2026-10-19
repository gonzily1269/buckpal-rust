//! ダッシュボード表示用のアプリケーションDTO。

use std::num::NonZeroU64;

/// ダッシュボードに表示する口座の残高情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardAccount {
    /// 口座の一意な識別子。
    pub id: NonZeroU64,
    /// 画面に表示する口座名。
    pub name: String,
    /// 現在の残高を整数単位で表した値。
    pub balance: u64,
}

/// ダッシュボードに表示する送金履歴。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardActivity {
    /// 送金履歴の一意な識別子。
    pub id: NonZeroU64,
    /// 送金元口座の識別子。
    pub source_account_id: NonZeroU64,
    /// 送金先口座の識別子。
    pub target_account_id: NonZeroU64,
    /// 送金額を整数単位で表した値。
    pub amount: u64,
    /// 送金が記録された日時の文字列表現。
    pub created_at: String,
}

/// ダッシュボードに表示する口座と最近の取引。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dashboard {
    /// 残高を表示する口座の一覧。
    pub accounts: Vec<DashboardAccount>,
    /// 新しい順に並べた最近の送金履歴。
    pub activities: Vec<DashboardActivity>,
}
