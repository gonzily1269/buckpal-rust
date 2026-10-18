use axum::{extract::Query, http::Uri};

use super::{HomeQuery, TransferStatus};

/// クエリコードは既知のkebab-case値だけがデシリアライズされる。
#[test]
fn transfer_status_accepts_known_codes_and_rejects_typos() {
    let parsed = Query::<HomeQuery>::try_from_uri(&Uri::from_static("/?status=insufficient-funds"));
    assert!(matches!(
        parsed,
        Ok(Query(HomeQuery {
            status: Some(TransferStatus::InsufficientFunds)
        }))
    ));
    assert!(
        Query::<HomeQuery>::try_from_uri(&Uri::from_static("/?status=insufficient_fund")).is_err()
    );
}
