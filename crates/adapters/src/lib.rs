//! アプリケーション契約を実装する外部システム用アダプター。

pub mod sqlite;

#[cfg(test)]
#[path = "tests/sqlite.rs"]
mod tests;
