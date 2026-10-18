//! アプリケーション出力DTOを画面描画用のモデルへ変換する。

use buckpal_application::{Dashboard, DashboardAccount, DashboardActivity};

#[cfg(test)]
#[path = "tests/view_model.rs"]
mod tests;

/// アプリケーションのダッシュボードDTOを画面用の一覧へ変換したモデル。
#[derive(Clone)]
pub(crate) struct DashboardView {
    /// 画面表示用に整形した口座カードの一覧。
    pub accounts: Vec<AccountView>,
    /// 画面表示用に整形した送金履歴の一覧。
    pub activities: Vec<ActivityView>,
}

/// 画面に表示する口座情報。
#[derive(Clone)]
pub(crate) struct AccountView {
    /// 送金フォームで使う口座ID。
    pub id: std::num::NonZeroU64,
    /// 画面に表示する書式付き口座ID。
    pub id_label: String,
    /// 口座名。
    pub name: String,
    /// 桁区切りを付けた残高。
    pub balance: String,
}

/// 画面に表示する送金履歴情報。
#[derive(Clone)]
pub(crate) struct ActivityView {
    /// 送金元と送金先を含む送金説明。
    pub description: String,
    /// 履歴の記録日時。
    pub created_at: String,
    /// 通貨表記を付けた送金額。
    pub amount: String,
}

impl DashboardView {
    /// アプリケーションDTO一式を画面用に変換する。
    pub fn new(dashboard: Dashboard) -> Self {
        Self {
            accounts: dashboard.accounts.into_iter().map(account_view).collect(),
            activities: dashboard
                .activities
                .into_iter()
                .map(activity_view)
                .collect(),
        }
    }
}

/// アプリケーションの口座DTOを画面表示向けに整形する。
fn account_view(account: DashboardAccount) -> AccountView {
    AccountView {
        id: account.id,
        id_label: format!("ACCOUNT {:02}", account.id),
        name: account.name,
        balance: format_amount(account.balance),
    }
}

/// アプリケーションの送金DTOを画面表示向けに整形する。
fn activity_view(activity: DashboardActivity) -> ActivityView {
    ActivityView {
        description: format!(
            "口座 {} → 口座 {}",
            activity.source_account_id, activity.target_account_id
        ),
        created_at: activity.created_at,
        amount: format!("−{}", format_amount(activity.amount)),
    }
}

/// 整数金額に3桁ごとのカンマを付けた文字列を返す。
fn format_amount(amount: u64) -> String {
    let digits = amount.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            formatted.push(',');
        }
        formatted.push(digit);
    }
    formatted
}
