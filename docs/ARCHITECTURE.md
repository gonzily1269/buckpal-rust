# アーキテクチャ

## クレートと依存方向

```mermaid
%%{init: {"theme": "base", "themeVariables": {"background": "#f8fafc", "mainBkg": "#f8fafc", "primaryColor": "#f8fafc", "primaryTextColor": "#111827", "primaryBorderColor": "#334155", "lineColor": "#94a3b8", "textColor": "#111827", "edgeLabelBackground": "#f8fafc", "fontFamily": "sans-serif"}}}%%
flowchart TB
    app["app<br/>composition root"]
    subgraph adapters["インターフェースアダプター層"]
        direction LR
        web["web<br/>HTTP / UI"]
        persistence["persistence<br/>SqliteRepository"]
    end
    subgraph service["アプリケーション層"]
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

    classDef composition fill:#4338ca,stroke:#312e81,stroke-width:2px,color:#ffffff
    classDef inputAdapter fill:#0284c7,stroke:#075985,stroke-width:2px,color:#ffffff
    classDef outputAdapter fill:#0f766e,stroke:#115e59,stroke-width:2px,color:#ffffff
    classDef applicationService fill:#2563eb,stroke:#1d4ed8,stroke-width:2px,color:#ffffff
    classDef port fill:#0891b2,stroke:#0e7490,stroke-width:2px,color:#ffffff
    classDef domainLayer fill:#172554,stroke:#0f172a,stroke-width:2px,color:#ffffff
    class app composition
    class web inputAdapter
    class persistence outputAdapter
    class usecase applicationService
    class inputPort,outputPort port
    class domain domainLayer
    style adapters fill:#e2e8f0,stroke:#64748b,stroke-width:2px,color:#111827
    style service fill:#e2e8f0,stroke:#64748b,stroke-width:2px,color:#1e293b
    linkStyle 0,2,4,6,7 stroke:#2563eb,stroke-width:3px
    linkStyle 1 stroke:#6366f1,stroke-width:3px
    linkStyle 3,5 stroke:#14b8a6,stroke-width:3px
```

この図の実線はソースコード上の関係です。青はクレート依存、インディゴは`app`による生成・注入、シアンはポートの実装を示します。外側の`persistence`が内側のserviceで定義された`TransferRepository`を実装し、`web`はserviceの入力ポートに依存します。

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
    classDef inputAdapter fill:#0284c7,stroke:#075985,stroke-width:2px,color:#ffffff
    classDef outputAdapter fill:#0f766e,stroke:#115e59,stroke-width:2px,color:#ffffff
    classDef applicationService fill:#2563eb,stroke:#1d4ed8,stroke-width:2px,color:#ffffff
    classDef port fill:#0891b2,stroke:#0e7490,stroke-width:2px,color:#ffffff
    class browser,database external
    class webHandler inputAdapter
    class sqliteRepo outputAdapter
    class usecase applicationService
    class inputPort,outputPort port
    linkStyle 0,5 stroke:#64748b,stroke-width:3px
    linkStyle 1 stroke:#2563eb,stroke-width:3px
    linkStyle 2 stroke:#6366f1,stroke-width:3px
    linkStyle 3 stroke:#14b8a6,stroke-width:3px
    linkStyle 4 stroke:#0f766e,stroke-width:3px
```

この図の点線は実行時の呼び出しです。スレートは外部との通信、青はユースケース呼び出し、インディゴはtrait object経由のディスパッチ、シアンはリポジトリ呼び出し、濃いシアンは永続化処理を示します。serviceがポートを呼び、注入済みのSQLite実装が応答します。`app`は`Arc<dyn TransferRepository>`とユースケースtrait objectを組み立ててWebへ注入します。

## 依存性逆転

serviceはデータ保存を必要としますが、SQLiteの具体型には依存しません。[`TransferRepository`](../crates/service/src/ports/repository.rs)が必要な操作を定義し、外側の`persistence`にある`SqliteRepository`がそのtraitを実装します。永続化の詳細が内側のservice所有の抽象に合わせます。

同様に、serviceは[`SendMoneyUseCase`](../crates/service/src/transfer/send_money.rs)と`GetDashboardUseCase`を入力ポートとして定義します。`web`はこれらのtraitを呼び、具象ユースケースを参照しません。依存の向きと実行時の呼び出し方向が異なり、serviceはSQLite・SQLx・HTTPの型を知りません。

## 境界と責務

- `service`は同一口座や送金上限を検証します。残高ルールは`domain`のモデルで判断します。
- SQLite実装は書き込みロック中に残高を読み、送金と活動記録を同一トランザクションでコミットします。
- `persistence`はSQLxの失敗をログに記録して`TransferRepositoryError`へ変換し、serviceはユースケースのエラーへ変換します。SQLite固有のエラー型は外側に留まります。
- `web`はserviceのDTOをWeb所有のViewModelへ変換して描画し、SQLxやSQLiteの型を扱いません。
- 設定読込、DB接続・マイグレーション、依存の組み立ては`app`、画面文言は`web`、DBスキーマは`migrations/`が所有します。

## テスト

ユースケースはfakeリポジトリでSQLiteなしにテストし、永続化はインメモリSQLiteと実マイグレーションで検証します。全テストは`cargo test --workspace`で実行します。
