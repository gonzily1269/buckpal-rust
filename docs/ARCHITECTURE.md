# アーキテクチャ

## クレートと依存方向

```mermaid
%%{init: {"theme": "base", "themeVariables": {"background": "#f8fafc", "mainBkg": "#f8fafc", "primaryColor": "#f8fafc", "primaryTextColor": "#111827", "primaryBorderColor": "#334155", "lineColor": "#ffffff", "textColor": "#111827", "edgeLabelBackground": "#f8fafc", "fontFamily": "sans-serif"}}}%%
flowchart TB
    browser["ブラウザー"]
    app["app<br/>composition root"]
    web["web<br/>HTTP / UI adapter"]
    subgraph service["service — 内側のアプリケーションルール"]
        direction TB
        inport["入力ポート<br/>SendMoneyUseCase<br/>GetDashboardUseCase"]
        usecase["ユースケース実装<br/>SendMoneyService<br/>GetDashboardService"]
        outport["出力ポート<br/>TransferRepository"]
    end
    persistence["persistence<br/>SqliteRepository"]
    db[("SQLite")]
    domain["domain<br/>Account / Transfer"]

    app -->|"依存"| web
    app -->|"依存"| usecase
    app -->|"依存"| persistence
    web -->|"依存する契約"| inport
    usecase -->|"実装"| inport
    usecase -->|"依存する契約"| outport
    persistence -->|"実装"| outport
    usecase -->|"業務ルールを利用"| domain
    persistence -->|"ドメイン型を利用"| domain

    browser -. "HTTP要求" .-> web
    web -. "ユースケース呼び出し" .-> inport
    inport -. "動的ディスパッチ" .-> usecase
    usecase -. "リポジトリ呼び出し" .-> outport
    outport -. "動的ディスパッチ" .-> persistence
    app -. "生成・注入" .-> persistence
    app -. "生成・注入" .-> usecase
    app -. "AppStateへ注入" .-> web
    persistence -. "SQL" .-> db

    classDef external fill:#475569,stroke:#1e293b,stroke-width:2px,color:#ffffff
    classDef composition fill:#6d28d9,stroke:#4c1d95,stroke-width:2px,color:#ffffff
    classDef inputAdapter fill:#2563eb,stroke:#1e40af,stroke-width:2px,color:#ffffff
    classDef outputAdapter fill:#0f766e,stroke:#134e4a,stroke-width:2px,color:#ffffff
    classDef applicationService fill:#c2410c,stroke:#7c2d12,stroke-width:2px,color:#ffffff
    classDef port fill:#a16207,stroke:#713f12,stroke-width:2px,color:#ffffff
    classDef domainLayer fill:#15803d,stroke:#14532d,stroke-width:2px,color:#ffffff
    class browser,db external
    class app composition
    class web inputAdapter
    class persistence outputAdapter
    class usecase applicationService
    class inport,outport port
    class domain domainLayer
    style service fill:#7c2d12,stroke:#431407,stroke-width:2px,color:#ffffff
    linkStyle default stroke:#ffffff,stroke-width:3px
```

実線はソースコード上の依存またはtrait実装関係、点線は起動時の注入と実行時の呼び出しを示します。依存性逆転の要点は、外側の`persistence`が内側の`service`所有の`TransferRepository`を実装し、内側のserviceがSQLiteではなくこのポートに依存することです。`web`もserviceの入力ポートだけを呼び出し、具象ユースケースを知りません。

`app`は具体型を選び、`Arc<dyn TransferRepository>`とユースケースtrait objectを組み立てて注入します。実行時はユースケースからポートを呼びますが、ソースコード上の依存先は内側の抽象です。`domain`は他の自プロジェクトクレートに依存せず、serviceにもHTTP・SQLx・SQLiteの型を持ち込みません。

## ポートの具体例

送金とダッシュボード取得で、serviceはデータ保存を必要としますが、SQLiteの具体型には依存しません。[`TransferRepository`](../crates/service/src/ports/repository.rs) が必要な操作を定義し、`persistence`の`SqliteRepository`が実装します。永続化の詳細がservice所有の抽象に合わせます。

同様に、serviceは [`SendMoneyUseCase`](../crates/service/src/transfer/send_money.rs) と `GetDashboardUseCase` を入力ポートとして定義します。`web`はこれらのtraitを呼び出し、具象ユースケースを参照しません。serviceはSQLite・SQLx・HTTPを知りません。

実行時にはserviceが`TransferRepository`を呼びますが、実装の選択と注入は [`app/src/main.rs`](../app/src/main.rs) で行います。ここで`SqliteRepository`を`Arc<dyn TransferRepository>`としてユースケースへ渡し、`SendMoneyService`／`GetDashboardService`をWebへ渡します。ユースケーステストでは、この実装をfakeに差し替えられます。

## 境界と責務

- `service`は同一口座や送金上限などのユースケース上の検証を行います。送金先・送金元の残高ルールは`domain`のモデルで判断し、SQLite実装は書き込みロック中に残高を読み、検証・記録を同一トランザクションで行います。
- `persistence`はSQLxの失敗をログに記録して`TransferRepositoryError`へ変換します。serviceはそれをユースケースのエラーへ変換し、webは業務結果や抽象化された失敗をHTTP応答へ変換します。SQLite固有のエラー型は外側に留まります。
- `web`は`SendMoneyUseCase`／`GetDashboardUseCase`とserviceのDTOを使い、画面表示前にWeb所有のViewModelへ変換します。SQLx行型やSQLite接続型を扱いません。
- 設定の読込、DB接続・マイグレーション、依存の組み立ては`app`が担います。画面文言は`web`、DBスキーマは`migrations/`が所有します。

## テスト

ユースケースはfakeリポジトリでSQLiteなしにテストし、永続化はインメモリSQLiteと実マイグレーションで検証します。全テストは`cargo test --workspace`で実行します。
