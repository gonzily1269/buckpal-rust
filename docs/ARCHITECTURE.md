# アーキテクチャ

このプロジェクトは、送金の業務ルールをWebフレームワークやデータベースから分離するため、クリーンアーキテクチャ／ヘキサゴナルアーキテクチャの考え方を取り入れています。中心にドメインとユースケースを置き、外部技術との接続はアダプターが担います。

## レイヤーと責務

| レイヤー | 主な場所 | 責務 |
| --- | --- | --- |
| ドメイン | `crates/domain/` | `Money`、`Account`などの業務概念と基本ルール |
| アプリケーション | `crates/application/` | 送金・ダッシュボード取得ユースケース、入出力契約、アプリケーションDTO |
| アダプター | `crates/adapters/` | SQLite問い合わせ、トランザクション、アプリケーション契約の実装 |
| Webアダプター | `crates/web/` | AxumのHTTP入出力、Leptos SSR画面、ViewModelへの変換 |
| 起動・構成 | `crates/bootstrap/` (`src/main.rs`) | 設定読込、具体実装の組み立て、サーバー起動 |

各レイヤーは別々のCargoクレートです。プロジェクト内クレート間のコンパイル時依存は次の向きに限定します。外部ライブラリへの依存は各クレートの責務に閉じ、domain/applicationからWeb・DBフレームワークを参照しません。

```text
bootstrap ──▶ web ─────────────▶ application ──▶ domain
    ├───────▶ adapters ────────▶ application
    │              └───────────────────────────▶ domain
    └───────▶ application
```

図の矢印はコンパイル時の依存方向です。実行時にはWebアダプターがapplicationのユースケースを呼び、ユースケースがリポジトリ契約を呼びます。SQLiteアダプターがそのリポジトリ契約を実装します。契約と境界DTOは利用側であるapplicationが所有し、`crates/bootstrap/src/main.rs`が具体実装を結び付けます。許可するプロジェクト内依存とdomain/applicationの外部依存は`crates/architecture/tests/architecture.rs`で検査します。

## 送金の処理フロー

1. `TransferForm`が通常のHTML POSTを`/`へ送信し、Axumの`submit_transfer`がHTTPフォームを`SendMoneyUseCase`入力へ変換します。
2. `SendMoneyService`は口座ID、同一口座、金額、送金上限を検証し、正の金額をドメイン型`Money`へ変換します。
3. `TransferRepository`で口座を読み込み、`Account::withdraw`／`Account::deposit`を使って残高不足とオーバーフローをドメイン層で判断します。
4. 成立した送金はドメイン型`Transfer`として`TransferRepository`へ渡します。SQLiteアダプターは`BEGIN IMMEDIATE`で書き込みロックを先に取得し、ロック中に残高を読み込んで送金と活動記録をまとめてコミットします。ロック取得前に作られた送金スナップショットが古い場合だけ、安全のため競合として拒否します。
5. Web層はユースケース結果をHTTPのリダイレクトとステータスコードへ変換します。ホーム画面の取得は`GetDashboardUseCase`契約を介し、`DashboardService`が`TransferRepository`へ委譲します。Web層は受け取ったアプリケーションDTOをWeb所有のViewModelへ変換し、LeptosはそのViewModelだけを描画します。

ホーム画面の読み取りはユースケース契約とリポジトリを通ります。Web層は`GetDashboardUseCase`契約のみを呼び、`DashboardService`は`TransferRepository`の読み取り操作を利用します。リポジトリの失敗は`GetDashboardError`へ変換され、Web層へ技術固有の失敗型を漏らしません。Web層のHTTPハンドラーは`Dashboard`／`DashboardAccount`／`DashboardActivity` DTOを受け取り、ViewModelへ変換します。SQLx行型やSQLite接続型は知りません。また、Leptos UIはアプリケーションDTOへ直接依存しません。

## 依存関係の組み立て

`crates/bootstrap/src/main.rs`が設定を読み込み、SQLite接続プールと`SqliteRepository`を生成します。同じSQLiteアダプターを`TransferRepository`のtrait objectとして`SendMoneyService`／`DashboardService`へ渡し、それぞれを`SendMoneyUseCase`／`GetDashboardUseCase`のtrait objectとしてWebの`AppState`へ渡します。具体型を選択するのはこのcomposition rootだけです。

アプリケーションの契約は技術固有の型や低レベルのエラー原因を公開しません。SQLiteアダプターは技術的な原因をログに記録し、リポジトリの抽象エラーを返します。ユースケースはそれを業務エラーまたはアプリケーションの永続化失敗へ変換し、Webアダプターにはリポジトリ固有のエラー型を露出しません。Webは業務結果を画面へ変換し、抽象的なシステム失敗をHTTP 500へ変換します。ロック取得はSQLiteに待機させ、口座読込から記録までを1回のRepository操作で完了させます。

環境設定は`config.toml`と環境変数で管理します。利用者向け表示文言はWebクレートのコードで管理します。設定の読み込み・解析は外側に置き、ドメインとユースケースにはファイル形式を持ち込みません。

## 境界のルール

- `domain`と`application`から外側の`web`、`adapters`をimportしません。
- Cargo依存グラフは`application → domain`、`adapters → application/domain`、`web → application`、`bootstrap → adapters/application/web`に限定し、`crates/architecture/tests/architecture.rs`で逸脱を検出します。domain/applicationの外部依存も許可リストで制限します。
- WebハンドラーはSQLiteリポジトリやSQLxの型ではなく、`SendMoneyUseCase`／`GetDashboardUseCase`契約とアプリケーションDTOだけを利用します。
- リポジトリ契約とその入出力型は利用側であるアプリケーション層が所有します。外側のアダプターが実装詳細を提供します。
- ユースケース契約もアプリケーション層が定義し、WebアダプターはHTTP入力をアプリケーション呼び出しへ変換します。ユースケースの実装型をWeb層へ露出しません。
- `crates/bootstrap/src/main.rs`をcomposition rootとし、具体アダプターの生成と契約への注入をここに限定します。
- ドメインエンティティの生成時にID・残高などの不変条件を検証し、不正な状態をモデル内に作れないようにします。
- 契約境界では、技術固有エラーや検証前のプリミティブ値ではなく、ドメイン型（`Money`／成立済みの`Transfer`）と抽象化された失敗型を使います。
- 設定ファイルや画面メッセージの読み込みは外側で行い、ドメイン・ユースケースへ設定形式やUIライブラリを持ち込みません。
- WebクレートはアプリケーションDTOを直接HTMLへ埋め込まず、Web所有のViewModelへ変換してから描画します。
- 残高不足・残高オーバーフローなどの送金可否はユースケースがドメインモデルを用いて判断し、DBトランザクション境界と書き込みロックは永続化アダプターが担います。
- HTTPステータス、フォームデコード、リダイレクトはWebアダプターで扱い、アプリケーション層へHTTP型を持ち込みません。
- リポジトリ由来の技術境界エラーはユースケースでアプリケーション結果へ変換し、入力アダプターにリポジトリ固有のエラー型を知らせません。

この分離により、ユースケースはfakeリポジトリでSQLiteを起動せずにテストでき、SQLiteアダプターもHTTPやLeptosを知らずに契約を実装できます。Webアダプターもアプリケーションの契約に対してテスト可能です。

## テスト方針

- ドメインのテストで、正の金額、引き出し、入金などのルールを確認します。
- アプリケーションのユースケーステストでは、リポジトリのspy/fakeを用いて入力検証とリポジトリ呼び出しを確認します。
- SQLiteアダプターのテストでは、インメモリSQLiteと実際のマイグレーションを使い、送金後の残高・履歴と失敗時のロールバックを確認します。
- 設定とメッセージのテストで、TOMLの必須項目やメッセージ書式を検証します。

テストの実行:

```powershell
cargo test --workspace
```
