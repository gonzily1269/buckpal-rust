//! UIとHTTP応答で共有するメッセージを読み込む。

use std::sync::Arc;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::TransferStatus;

/// 画面とHTTP応答で共通利用するメッセージ一式。
#[derive(Debug, Clone, Deserialize)]
pub struct Messages {
    pub page: PageMessages,
    pub account: AccountMessages,
    pub transfer: TransferMessages,
    pub activity: ActivityMessages,
    pub errors: ErrorMessages,
    pub status: StatusMessages,
}

/// ページ全体の見出しやフッターに表示する文言。
#[derive(Debug, Clone, Deserialize)]
pub struct PageMessages {
    pub title: String,
    pub brand: String,
    pub brand_suffix: String,
    pub connection_status: String,
    pub hero_eyebrow: String,
    pub hero_title: String,
    pub hero_subtitle: String,
    pub accounts_eyebrow: String,
    pub accounts_title: String,
    pub account_count: String,
    pub transfer_eyebrow: String,
    pub recent_transfers_eyebrow: String,
    pub recent_transfers_title: String,
    pub footer_brand: String,
    pub footer_separator: String,
    pub footer_caption: String,
}

/// 口座カードと口座情報に表示する文言。
#[derive(Debug, Clone, Deserialize)]
pub struct AccountMessages {
    pub id: String,
    pub balance_label: String,
    pub units: String,
}

/// 送金フォームに表示する文言。
#[derive(Debug, Clone, Deserialize)]
pub struct TransferMessages {
    pub title: String,
    pub description: String,
    pub source_label: String,
    pub target_label: String,
    pub amount_label: String,
    pub amount_placeholder: String,
    pub submit: String,
    pub submit_arrow: String,
    pub hint: String,
    pub account_option: String,
}

/// 送金履歴に表示する文言。
#[derive(Debug, Clone, Deserialize)]
pub struct ActivityMessages {
    pub empty: String,
    pub transfer: String,
    pub amount: String,
}

/// HTTP処理で使う共通エラーメッセージ。
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorMessages {
    pub internal_server_error: String,
}

/// 送金結果コードごとの利用者向けメッセージ。
#[derive(Debug, Clone, Deserialize)]
pub struct StatusMessages {
    pub success: StatusMessage,
    #[serde(rename = "invalid-amount")]
    pub invalid_amount: StatusMessage,
    #[serde(rename = "same-account")]
    pub same_account: StatusMessage,
    pub threshold: StatusMessage,
    #[serde(rename = "account-not-found")]
    pub account_not_found: StatusMessage,
    #[serde(rename = "insufficient-funds")]
    pub insufficient_funds: StatusMessage,
    #[serde(rename = "balance-overflow")]
    pub balance_overflow: StatusMessage,
    #[serde(rename = "concurrent-update")]
    pub concurrent_update: StatusMessage,
}

/// 画面に表示する文言とDaisyUIアラートのクラス。
#[derive(Debug, Clone, Deserialize)]
pub struct StatusMessage {
    pub class: AlertLevel,
    pub text: String,
}

/// ステータスメッセージで利用できるDaisyUIアラートの種類。
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum AlertLevel {
    /// 成功を表すアラート。
    #[serde(rename = "alert-success")]
    Success,
    /// 警告を表すアラート。
    #[serde(rename = "alert-warning")]
    Warning,
    /// エラーを表すアラート。
    #[serde(rename = "alert-error")]
    Error,
    /// 情報を表すアラート。
    #[serde(rename = "alert-info")]
    Info,
}

impl AlertLevel {
    /// DaisyUIで利用する固定のCSSクラス名を返す。
    #[must_use]
    pub const fn css_class(self) -> &'static str {
        match self {
            Self::Success => "alert-success",
            Self::Warning => "alert-warning",
            Self::Error => "alert-error",
            Self::Info => "alert-info",
        }
    }
}

impl Messages {
    /// バイナリに埋め込まれたmessages.tomlを解析し、必須項目と書式を検証する。
    ///
    /// # Errors
    ///
    /// 埋め込みTOMLの解析または必須項目の検証に失敗した場合にエラーを返す。
    pub fn load() -> Result<Arc<Self>> {
        let messages: Self = toml::from_str(include_str!("../../../messages.toml"))
            .context("could not parse embedded messages")?;
        messages.validate()?;
        Ok(Arc::new(messages))
    }

    /// 送金結果に対応するメッセージを返す。
    #[must_use]
    pub fn status(&self, status: TransferStatus) -> &StatusMessage {
        match status {
            TransferStatus::Success => &self.status.success,
            TransferStatus::InvalidAmount => &self.status.invalid_amount,
            TransferStatus::SameAccount => &self.status.same_account,
            TransferStatus::Threshold => &self.status.threshold,
            TransferStatus::AccountNotFound => &self.status.account_not_found,
            TransferStatus::InsufficientFunds => &self.status.insufficient_funds,
            TransferStatus::BalanceOverflow => &self.status.balance_overflow,
            TransferStatus::ConcurrentUpdate => &self.status.concurrent_update,
        }
    }

    /// 必須プレースホルダーを検証する。
    fn validate(&self) -> Result<()> {
        ensure!(
            self.page.account_count.matches("{}").count() == 1,
            "messages.page.account_count must contain exactly one {{}} placeholder"
        );
        ensure!(
            self.account.id.matches("{}").count() == 1,
            "messages.account.id must contain exactly one {{}} placeholder"
        );
        ensure!(
            self.transfer.account_option.matches("{}").count() == 2,
            "messages.transfer.account_option must contain exactly two {{}} placeholders"
        );
        ensure!(
            self.activity.transfer.matches("{}").count() == 2,
            "messages.activity.transfer must contain exactly two {{}} placeholders"
        );
        ensure!(
            self.activity.amount.matches("{}").count() == 1,
            "messages.activity.amount must contain exactly one {{}} placeholder"
        );
        Ok(())
    }
}

/// `{}`を左から順に値で置換し、表示用メッセージを作成する。
#[must_use]
pub fn format_message(template: &str, values: &[&str]) -> String {
    let mut parts = template.split("{}");
    let mut formatted = parts.next().unwrap_or_default().to_owned();
    for value in values {
        formatted.push_str(value);
        formatted.push_str(parts.next().unwrap_or_default());
    }
    formatted
}

#[cfg(test)]
mod tests {
    use super::{AlertLevel, Messages, format_message};
    use crate::TransferStatus;

    /// 埋め込みメッセージが読み込まれ、型付き結果とアラートクラスを解決できることを確認する。
    #[test]
    fn loads_and_validates_embedded_messages() {
        let result = Messages::load()
            .map(|messages| messages.status(TransferStatus::Success).class.css_class())
            .map_err(|error| error.to_string());
        assert_eq!(result, Ok(AlertLevel::Success.css_class()));
    }

    /// 複数のプレースホルダーが指定順に置換されることを確認する。
    #[test]
    fn formats_placeholders_in_order() {
        assert_eq!(
            format_message("{} → {}", &["口座1", "口座2"]),
            "口座1 → 口座2"
        );
    }
}
