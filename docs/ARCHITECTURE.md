# アーキテクチャ

## クレートと依存方向

```mermaid
%%{init: {"theme": "base", "themeVariables": {"background": "#f8fafc", "mainBkg": "#f8fafc", "primaryColor": "#f8fafc", "primaryTextColor": "#111827", "primaryBorderColor": "#334155", "lineColor": "#94a3b8", "textColor": "#111827", "edgeLabelBackground": "#f8fafc", "fontFamily": "sans-serif"}}}%%
flowchart TB
    app["app<br/>composition root"]
    subgraph adapters["外側のアダプター"]
        direction LR
        web["web<br/>HTTP / UI"]
        persistence["persistence<br/>SqliteRepository"]
    end
    subgraph service["service — 内側のアプリケーションルール"]
        direction LR
        inputPort["入力ポート<br/>SendMoneyUseCase<br/>GetDashboardUseCase"]
        usecase["ユースケース実装<br/>SendMoneyService<br/>GetDashboardService"]
        outputPort["出力ポート<br/>TransferRepository"]
    end
    domain["domain<br/>Account / Transfer"]

    app -->|"依存"| adapters
    app -->|"生成・注入"| usecase
    web -->|"依存"| inputPort
    usecase -->|"実装"| inputPort
    usecase -->|"依存"| outputPort
    persistence -->|"実装"| outputPort
    usecase -->|"依存"| domain
    persistence -->|"依存"| domain

    classDef external fill:#475569,stroke:#1e293b,stroke-width:2px,color:#ffffff
    classDef composition fill:#6d28d9,stroke:#4c1d95,stroke-width:2px,color:#ffffff
    classDef inputAdapter fill:#2563eb,stroke:#1e40af,stroke-width:2px,color:#ffffff
    classDef outputAdapter fill:#0f766e,stroke:#134e4a,stroke-width:2px,color:#ffffff
    classDef applicationService fill:#c2410c,stroke:#7c2d12,stroke-width:2px,color:#ffffff
    classDef port fill:#a16207,stroke:#713f12,stroke-width:2px,color:#ffffff
    classDef domainLayer fill:#15803d,stroke:#14532d,stroke-width:2px,color:#ffffff
    class app composition
    class web inputAdapter
    class persistence outputAdapter
    class usecase applicationService
    class inputPort,outputPort port
    class domain domainLayer
    style adapters fill:#e2e8f0,stroke:#64748b,stroke-width:2px,color:#111827
    style service fill:#7c2d12,stroke:#431407,stroke-width:2px,color:#ffffff
    linkStyle 0,2,4,6,7 stroke:#2563eb,stroke-width:3px
    linkStyle 1 stroke:#7e22ce,stroke-width:3px
    linkStyle 3,5 stroke:#16a34a,stroke-width:3px
```

この図の**実線**はソースコード上の関係を示します。青は依存、紫は`app`による生成・注入、緑はポートの実装です。外側の`persistence`が内側のserviceで定義された`TransferRepository`を実装し、`web`はserviceの入力ポートに依存します。

### 実行時の呼び出し

```mermaid
%%{init: {"theme": "base", "themeVariables": {"background": "#f8fafc", "mainBkg": "#f8fafc", "primaryColor": "#f8fafc", "primaryTextColor": "#111827", "primaryBorderColor": "#334155", "lineColor": "#94a3b8", "textColor": "#111827", "edgeLabelBackground": "#f8fafc", "fontFamily": "sans-serif"}}}%%
flowchart TB
    browser["ブラウザー"]
    webHandler["web<br/>HTTPハンドラー"]
    inputPort["入力ポート<br/>SendMoneyUseCase"]
    usecase["service<br/>SendMoneyService"]
    outputPort["出力ポート<br/>TransferRepository"]
    sqliteRepo["persistence<br/>SqliteRepository"]
    database[("SQLite")]

    browser -.->|"HTTP要求"| webHandler
    webHandler -.->|"ユースケース呼び出し"| inputPort
    inputPort -.->|"trait object"| usecase
    usecase -.->|"リポジトリ呼び出し"| outputPort
    outputPort -.->|"trait object"| sqliteRepo
    sqliteRepo -.->|"SQL"| database

    classDef external fill:#475569,stroke:#1e293b,stroke-width:2px,color:#ffffff
    classDef inputAdapter fill:#2563eb,stroke:#1e40af,stroke-width:2px,color:#ffffff
    classDef outputAdapter fill:#0f766e,stroke:#134e4a,stroke-width:2px,color:#ffffff
    classDef applicationService fill:#c2410c,stroke:#7c2d12,stroke-width:2px,color:#ffffff
    classDef port fill:#a16207,stroke:#713f12,stroke-width:2px,color:#ffffff
    class browser,database external
    class webHandler inputAdapter
    class sqliteRepo outputAdapter
    class usecase applicationService
    class inputPort,outputPort port
    linkStyle 0 stroke:#64748b,stroke-width:3px
    linkStyle 1 stroke:#2563eb,stroke-width:3px
    linkStyle 2 stroke:#7e22ce,stroke-width:3px
    linkStyle 3 stroke:#0f766e,stroke-width:3px
    linkStyle 4 stroke:#16a34a,stroke-width:3px
    linkStyle 5 stroke:#64748b,stroke-width:3px
```

この図の**点線**は実行時の呼び出しです。グレーは外部との通信、青はユースケース呼び出し、紫はtrait object経由のディスパッチ、青緑はリポジトリ呼び出し、緑は永続化処理を示します。serviceがポートを呼び、注入済みのSQLite実装が応答します。`app`は`Arc<dyn TransferRepository>`とユースケースtrait objectを組み立ててWebへ注入します。`domain`は他の自プロジェクトクレートに依存せず、serviceにもHTTP・SQLx・SQLiteの型を持ち込みません。

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
