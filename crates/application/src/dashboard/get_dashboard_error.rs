use thiserror::Error;

/// ダッシュボード取得ユースケースが返すアプリケーションエラー。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GetDashboardError {
    #[error("dashboard data could not be retrieved")]
    /// ダッシュボード情報の読み込みに失敗した。
    Unavailable,
}
