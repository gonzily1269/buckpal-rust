use super::format_amount;

/// 3桁区切りが金額の桁数に応じて正しく入ることを確認する。
#[test]
fn formats_amounts_for_display() {
    assert_eq!(format_amount(0), "0");
    assert_eq!(format_amount(500), "500");
    assert_eq!(format_amount(5_000), "5,000");
    assert_eq!(format_amount(1_000_000), "1,000,000");
}
