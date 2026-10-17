//! ダッシュボード参照のための出力ポートとアプリケーションDTO。

use async_trait::async_trait;
use thiserror::Error;

/// ダッシュボードに表示する口座の残高情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardAccount {
    /// 口座の一意な識別子。
    pub id: i64,
    /// 画面に表示する口座名。
    pub name: String,
    /// 現在の残高を整数単位で表した値。
    pub balance: i64,
}

/// ダッシュボードに表示する送金履歴。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardActivity {
    /// 送金履歴の一意な識別子。
    pub id: i64,
    /// 送金元口座の識別子。
    pub source_account_id: i64,
    /// 送金先口座の識別子。
    pub target_account_id: i64,
    /// 送金額を整数単位で表した値。
    pub amount: i64,
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

/// ダッシュボード参照で発生するアプリケーション境界のエラー。
#[derive(Debug, Error)]
#[error("dashboard query failed")]
pub struct DashboardQueryFailure;

/// ダッシュボード情報を取得するための出力ポート。
#[async_trait]
pub trait LoadDashboardPort: Send + Sync {
    /// ダッシュボードに表示する口座と送金履歴を読み込む。
    async fn load_dashboard(&self) -> Result<Dashboard, DashboardQueryFailure>;
}
