//! `Axum`のHTTPハンドラーとWeb層の状態を提供するWebアダプター。

mod transfer_status;
mod ui;
mod view_model;

pub use transfer_status::TransferStatus;

use anyhow::anyhow;
use axum::{
    body::Body,
    extract::{Form, Query, State},
    http::{Request, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use std::num::NonZeroU64;
use tracing::error;

use buckpal_application::{
    GetDashboardError, GetDashboardUseCase, SendMoneyError, SendMoneyUseCase,
};

use view_model::DashboardView;

#[cfg(test)]
#[path = "tests/web.rs"]
mod tests;

/// Web層からユースケースと口座データを利用するための共有状態。
#[derive(Clone)]
pub struct AppState {
    /// 送金を検証・実行するアプリケーションユースケース。
    send_money: std::sync::Arc<dyn SendMoneyUseCase>,
    /// 画面に表示する口座と取引を取得するリポジトリ。
    dashboard: std::sync::Arc<dyn GetDashboardUseCase>,
}

impl AppState {
    /// アプリケーションユースケースと文言をHTTP状態へ束ねる。
    pub fn new(
        send_money: std::sync::Arc<dyn SendMoneyUseCase>,
        dashboard: std::sync::Arc<dyn GetDashboardUseCase>,
    ) -> Self {
        Self {
            send_money,
            dashboard,
        }
    }
}

/// SSR画面の描画に必要な口座、取引履歴、送金結果メッセージ。
#[derive(Clone)]
struct PageData {
    /// 残高を含む表示対象口座。
    dashboard: DashboardView,
    /// リダイレクト後に画面へ表示する送金結果コード。
    status: Option<TransferStatus>,
}

/// 送金フォームからAxumがデシリアライズする入力項目。
#[derive(Debug, Deserialize)]
pub struct TransferForm {
    /// 送金元口座のID。
    source_account_id: NonZeroU64,
    /// 送金先口座のID。
    target_account_id: NonZeroU64,
    /// 送金する整数単位の金額。
    amount: NonZeroU64,
}

/// ホーム画面のクエリ文字列から読み取る送金結果。
#[derive(Debug, Default, Deserialize)]
pub struct HomeQuery {
    /// 送金結果を識別するコード。未指定なら結果メッセージを表示しない。
    status: Option<TransferStatus>,
}

/// 口座一覧と送金履歴を取得し、Leptosでホーム画面をSSR描画する。
///
/// # Errors
///
/// ダッシュボードの取得に失敗した場合、利用者向けHTTPエラーを返す。
pub async fn home(
    State(state): State<AppState>,
    Query(query): Query<HomeQuery>,
    request: Request<Body>,
) -> Result<Response, WebError> {
    let dashboard = state.dashboard.get_dashboard().await.map_err(|error| {
        let message = match error {
            GetDashboardError::Unavailable => {
                "処理中にエラーが発生しました。サーバーログを確認してください。".to_owned()
            }
        };
        WebError(anyhow!(error), message)
    })?;
    let dashboard = DashboardView::new(dashboard);
    let page = PageData {
        dashboard,
        status: query.status,
    };

    let handler = leptos_axum::render_app_to_stream_with_context(
        move || {
            leptos::prelude::provide_context(page.clone());
        },
        ui::App,
    );
    Ok(handler(request).await.into_response())
}

/// 送金フォームを処理し、成功または業務エラーをクエリ文字列で画面へ返す。
///
/// # Errors
///
/// 送金ユースケースが失敗した場合、利用者向けHTTPエラーを返す。
pub async fn submit_transfer(
    State(state): State<AppState>,
    Form(form): Form<TransferForm>,
) -> Result<Redirect, WebError> {
    let status = match state
        .send_money
        .send_money(form.source_account_id, form.target_account_id, form.amount)
        .await
    {
        Ok(()) => TransferStatus::Success,
        Err(SendMoneyError::InvalidAmount) => TransferStatus::InvalidAmount,
        Err(SendMoneyError::SameAccount) => TransferStatus::SameAccount,
        Err(SendMoneyError::ThresholdExceeded) => TransferStatus::Threshold,
        Err(SendMoneyError::AccountNotFound) => TransferStatus::AccountNotFound,
        Err(SendMoneyError::InsufficientFunds) => TransferStatus::InsufficientFunds,
        Err(SendMoneyError::BalanceOverflow) => TransferStatus::BalanceOverflow,
        Err(SendMoneyError::PersistenceFailure) => {
            return Err(WebError(
                anyhow!("application use case failed"),
                "処理中にエラーが発生しました。サーバーログを確認してください。".to_owned(),
            ));
        }
    };
    Ok(Redirect::to(&format!("/?status={}", status.as_str())))
}

/// ログへ原因を記録し、利用者向けHTTP 500応答へ変換するWebエラー。
#[derive(Debug)]
pub struct WebError(anyhow::Error, String);

impl IntoResponse for WebError {
    /// 内部エラー詳細はログに記録し、レスポンスには一般的な案内だけを含める。
    fn into_response(self) -> Response {
        error!(error = ?self.0, "request failed");
        (StatusCode::INTERNAL_SERVER_ERROR, self.1).into_response()
    }
}
