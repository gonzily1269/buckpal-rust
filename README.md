# Buckpal Rust

Spring版Buckpalの送金ユースケースを、Rustのクリーン/ヘキサゴナルアーキテクチャで実装したWebアプリです。画面はLeptos SSR、HTTPサーバーはAxum、永続化はSQLx + SQLiteです。画面はDaisyUIとTailwind CSSのCDNを利用し、送金フォームは通常のHTML POSTとして動作するため、フロントエンド用の別ビルドは不要です。

## 必要環境

- Rust stable

## 起動

```powershell
cargo run
```

`http://127.0.0.1:3000` を開くと、デモ画面を利用できます。初回起動時にカレントディレクトリへ `buckpal.db` を作成し、口座（生活口座 5,000 units、貯蓄口座 2,500 units）を登録します。以後の起動ではSQLiteのデータが維持されます。

## 設定

実行時設定はプロジェクトルートの [`config.toml`](./config.toml) で一元管理します。別の設定ファイルを使う場合は `BUCKPAL_CONFIG` でパスを指定できます。全項目が必須で、欠落や形式不正があれば起動時にエラーになります。環境変数を設定した項目は、TOMLの値を上書きします。

| 環境変数 | 上書き対象 | 説明 |
| --- | --- | --- |
| `BUCKPAL_CONFIG` | 設定ファイル | 読み込むTOMLファイルのパス |
| `DATABASE_URL` | `database.url` | SQLite接続URL |
| `BUCKPAL_TRANSFER_THRESHOLD` | `transfer.threshold` | 1回の送金上限（整数単位） |
| `BIND_ADDR` | `server.bind_addr` | HTTP待受アドレス |
| `RUST_LOG` | `logging.filter` | ログフィルター |
| `BUCKPAL_DB_MAX_CONNECTIONS` | `database.max_connections` | SQLite接続プールの最大接続数（1以上） |
| `BUCKPAL_DB_BUSY_TIMEOUT_SECONDS` | `database.busy_timeout_seconds` | SQLiteロック待機時間（秒、0以上） |

接続プール数とSQLiteのbusy timeoutは`config.toml`で設定でき、対応する環境変数で上書きできます。環境変数はTOMLの値に優先します。設定値は起動時に読み込まれ、TOML解析や型・範囲検証に失敗した場合はエラーで終了します。

画面表示とHTTPエラー応答の文言は、それぞれの画面コンポーネントとHTTPハンドラーに定義しています。文言を変更した場合はアプリケーションの再ビルドが必要です。送金結果のクエリ値は定義済みの結果コードに限られ、未知の値はHTTP 400になります。

```toml
[database]
url = "sqlite://buckpal.db?mode=rwc"
max_connections = 5
busy_timeout_seconds = 5

[server]
bind_addr = "127.0.0.1:3000"

[transfer]
threshold = 10000

[logging]
filter = "info"
```

例:

```powershell
$env:DATABASE_URL = "sqlite://buckpal.db?mode=rwc"
$env:BUCKPAL_TRANSFER_THRESHOLD = "20000"
$env:BUCKPAL_DB_MAX_CONNECTIONS = "10"
$env:BUCKPAL_DB_BUSY_TIMEOUT_SECONDS = "15"
cargo run
```

サーバーはCtrl-C（UnixではSIGTERMも）を受け取ると、新しい接続の受付を停止し、進行中のリクエストを完了してから終了します。

## 構成

- [アーキテクチャの説明](./docs/ARCHITECTURE.md)
- `crates/domain`: 金額・口座など、外部技術に依存しないドメインモデル
- `crates/application`: 送金・ダッシュボード取得ユースケースとアプリケーション契約
- `crates/adapters`: SQLiteによるリポジトリ実装と残高・履歴の取得
- `crates/web`: AxumのHTTPハンドラー、Leptos SSR画面、画面用ViewModel
- `crates/bootstrap/src/main.rs`: 設定、具体実装、ユースケースを組み立てるcomposition root
- `crates/bootstrap/src/config.rs`: TOML設定と環境変数オーバーライドの読み込み・検証
- `crates/architecture`: クレート間の依存方向を検査するArchitecture test
- `config.toml`: 実行時設定の既定値
- `migrations`: 初期スキーマとデモ口座

画面の`AccountCard`、`TransferForm`、`ActivityList`、`StatusMessage`は独立したLeptosコンポーネントです。UIライブラリはWeb層だけで使用し、ドメイン層・ユースケース層には依存しません。DaisyUIとTailwind CSSのアセットはCDNから読み込むため、画面表示にはインターネット接続が必要です。

送金可否はドメインモデルで判断し、SQLiteアダプターは`BEGIN IMMEDIATE`で書き込みロックを取得してから、口座読込・送金検証・送金と2つの活動記録を1回のRepository操作で完了します。残高不足や上限超過などの業務エラーは画面上に表示され、永続化エラーはHTTP 500とサーバーログで明示されます。

## 検証

```powershell
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```
