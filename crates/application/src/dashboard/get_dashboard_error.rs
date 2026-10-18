use thiserror::Error;

/// ダッシュボード取得ユースケースが返すアプリケーションエラー。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GetDashboardError {
    #[error("ダッシュボード情報を取得できませんでした")]
    /// ダッシュボード情報の読み込みに失敗した。
    Unavailable,
}
