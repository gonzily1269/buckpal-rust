# 開発規約

この文書では、このリポジトリで採用しているファイル配置と実装上の規約を定めます。クレート間の依存方向や各レイヤーの責務は[アーキテクチャの説明](./ARCHITECTURE.md)を参照してください。

## ユースケースのファイル構成

- application層のユースケースは、入力アダプターが依存するtraitとそのサービス実装を別ファイルにします。
- ファイル名は操作名を基準とし、traitを`<operation>.rs`、サービス実装を`<operation>_service.rs`に置きます。たとえば`get_dashboard.rs`に`GetDashboardUseCase`、`get_dashboard_service.rs`に`GetDashboardService`を置きます。
- 機能ごとのモジュールは`crates/application/src/<feature>/`に配置し、`mod.rs`で外部へ公開するAPIを管理します。現在の機能ディレクトリは`transfer/`と`dashboard/`です。
- enumは用途や所属レイヤーにかかわらず、1 enumにつき1ファイルへ分けます。ファイル名はenum名の`snake_case`とし、enumに関連する実装も同じファイルに置きます。例: `send_money_error.rs`、`transfer_status.rs`。

## テストの配置

- ソースと分離するモジュールテストは、対象モジュールのディレクトリ内にある`tests/`へ置きます。例: `src/dashboard/tests/use_case.rs`。
- テストファイルは対象実装モジュールから`#[cfg(test)]`と`#[path = "tests/..."]`で読み込みます。
- テストモジュール宣言は、対象ソース内の`use`宣言の後、最初の本番コード項目の前に置きます。crate rootでは、既存の`mod`宣言や`pub use`宣言の後、本番コード項目があればその前に置きます。

## Rustコード

- 公開APIの説明は日本語のdoc commentで記述します。
- 利用者向け表示、エラー、ログなど、実行時に人が読むメッセージは日本語で記述します。環境変数名、設定キー、URL・クエリの値など、外部仕様として固定された識別子は変更しません。
- サービス実装では型定義の後に、公開traitの実装、公開メソッド、非公開関数の順で配置します。利用側から見える概要を先に示し、内部の詳細へ読み進める構成にします。
- Clippyはworkspace設定で`all`と`pedantic`をdenyにしています。また、`unwrap`、`expect`、明示的なpanicや未実装処理、panicにつながるインデックス・文字列スライスもdeny対象です。設定はルートの`Cargo.toml`を参照してください。
