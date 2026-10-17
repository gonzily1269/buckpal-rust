//! アプリケーション出力DTOを画面描画用のモデルへ変換する。

use buckpal_application::{Dashboard, DashboardAccount, DashboardActivity};

use crate::messages::{Messages, format_message};

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
    pub id: i64,
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
    /// メッセージ定義を適用した送金説明。
    pub description: String,
    /// 履歴の記録日時。
    pub created_at: String,
    /// 通貨表記を適用した送金額。
    pub amount: String,
}

impl DashboardView {
    /// アプリケーションDTO一式をメッセージ定義に基づき画面用に変換する。
    pub fn new(dashboard: Dashboard, messages: &Messages) -> Self {
        Self {
            accounts: dashboard
                .accounts
                .into_iter()
                .map(|account| account_view(account, messages))
                .collect(),
            activities: dashboard
                .activities
                .into_iter()
                .map(|activity| activity_view(activity, messages))
                .collect(),
        }
    }
}

/// アプリケーションの口座DTOを画面表示向けに整形する。
fn account_view(account: DashboardAccount, messages: &Messages) -> AccountView {
    AccountView {
        id: account.id,
        id_label: format_message(&messages.account.id, &[&format!("{:02}", account.id)]),
        name: account.name,
        balance: format_amount(account.balance),
    }
}

/// アプリケーションの送金DTOを画面表示向けに整形する。
fn activity_view(activity: DashboardActivity, messages: &Messages) -> ActivityView {
    ActivityView {
        description: format_message(
            &messages.activity.transfer,
            &[
                &activity.source_account_id.to_string(),
                &activity.target_account_id.to_string(),
            ],
        ),
        created_at: activity.created_at,
        amount: format_message(
            &messages.activity.amount,
            &[&format_amount(activity.amount)],
        ),
    }
}

/// 整数金額に3桁ごとのカンマを付けた文字列を返す。
fn format_amount(amount: i64) -> String {
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

#[cfg(test)]
mod tests {
    use super::format_amount;

    /// 3桁区切りが金額の桁数に応じて正しく入ることを確認する。
    #[test]
    fn formats_amounts_for_display() {
        assert_eq!(format_amount(0), "0");
        assert_eq!(format_amount(500), "500");
        assert_eq!(format_amount(5_000), "5,000");
        assert_eq!(format_amount(1_000_000), "1,000,000");
    }
}
