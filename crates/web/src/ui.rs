//! DaisyUIを使った画面と、その画面を構成するLeptosコンポーネント。

use leptos::prelude::*;

use super::{
    PageData, TransferStatus,
    view_model::{AccountView, ActivityView},
};

#[component]
/// 口座残高、送金フォーム、最近の送金履歴を表示するLeptos画面。
pub(super) fn App() -> impl IntoView {
    let Some(page) = use_context::<PageData>() else {
        tracing::error!("page data was not provided during SSR");
        return None;
    };
    Some(view! {
        <html lang="ja" data-theme="light">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <title>"Buckpal | 口座間送金"</title>
                <link href="https://cdn.jsdelivr.net/npm/daisyui@5.7.50" rel="stylesheet" type="text/css"/>
                <script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4.3.3"></script>
            </head>
            <body>
                <main class="container mx-auto max-w-6xl px-4 sm:px-6">
                    <header class="navbar border-b border-base-300 px-0">
                        <a class="text-2xl font-extrabold tracking-tight text-primary" href="/">
                            "buckpal"<span class="text-secondary">".rust"</span>
                        </a>
                        <div class="badge badge-success badge-outline gap-2">
                            <span class="status status-success"></span>
                            "SQLite 接続中"
                        </div>
                    </header>
                    <section class="py-12 sm:py-14">
                        <p class="mb-3 text-xs font-bold tracking-[0.16em] text-base-content/60">"PERSONAL FINANCE / DEMO"</p>
                        <h1 class="text-3xl font-bold tracking-tight text-primary sm:text-4xl">"お金の流れを、シンプルに。"</h1>
                        <p class="mt-3 text-base text-base-content/70">"口座残高を確認して、安全に送金できます。"</p>
                    </section>
                    <StatusMessage status=page.status/>
                    <section class="space-y-4">
                        <div class="flex flex-wrap items-end justify-between gap-2">
                            <div>
                                <p class="mb-2 text-xs font-bold tracking-[0.16em] text-base-content/60">"YOUR ACCOUNTS"</p>
                                <h2 class="text-2xl font-semibold">"口座"</h2>
                            </div>
                            <span class="badge badge-ghost">{format!("{} ACCOUNTS", page.dashboard.accounts.len())}</span>
                        </div>
                        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                            {page.dashboard.accounts.iter().cloned().map(|account| view! { <AccountCard account=account/> }).collect_view()}
                        </div>
                    </section>
                    <section class="mt-8 grid grid-cols-1 gap-5 lg:grid-cols-2">
                        <TransferForm accounts=page.dashboard.accounts.clone()/>
                        <div class="card bg-base-100 shadow-sm">
                            <div class="card-body">
                                <div class="mb-2">
                                    <p class="mb-2 text-xs font-bold tracking-[0.16em] text-base-content/60">"RECENT TRANSFERS"</p>
                                    <h2 class="card-title">"最近の送金"</h2>
                                </div>
                                <ActivityList activities=page.dashboard.activities.clone()/>
                            </div>
                        </div>
                    </section>
                    <footer class="py-8 text-center text-xs tracking-widest text-base-content/50">
                        "BUCKPAL RUST"
                        <span class="px-2">"·"</span>
                        "クリーンアーキテクチャの送金デモ"
                    </footer>
                </main>
            </body>
        </html>
    })
}

#[component]
/// 口座名と利用可能残高を表示するDaisyUIカード。
fn AccountCard(account: AccountView) -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body gap-4">
                <div class="flex items-center justify-between">
                    <span class="avatar placeholder"><span class="rounded-xl bg-primary/10 text-primary">"↗"</span></span>
                    <span class="badge badge-ghost">{account.id_label}</span>
                </div>
                <h3 class="text-lg font-semibold">{account.name}</h3>
                <div class="stat p-0">
                    <div class="stat-title">"利用可能残高"</div>
                    <div class="stat-value text-primary">{account.balance}</div>
                    <div class="stat-desc">"units"</div>
                </div>
            </div>
        </div>
    }
}

#[component]
/// DaisyUIのフォーム部品を使った送金フォーム。
fn TransferForm(accounts: Vec<AccountView>) -> impl IntoView {
    let target_options = || {
        accounts
            .iter()
            .map(|account| {
                let id = account.id.to_string();
                let name = account.name.clone();
                view! { <option value=id>{name}</option> }
            })
            .collect_view()
    };
    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body">
                <div class="mb-2">
                    <p class="mb-2 text-xs font-bold tracking-[0.16em] text-base-content/60">"MOVE MONEY"</p>
                    <h2 class="card-title">"送金する"</h2>
                    <p class="mt-2 text-sm text-base-content/70">"送金は2口座に記録され、残高へすぐ反映されます。"</p>
                </div>
                <form method="post" action="/" class="flex flex-col gap-3">
                    <label class="form-control w-full">
                        <span class="label-text mb-1">"送金元"</span>
                        <select class="select select-bordered w-full" name="source_account_id" required>
                            {target_options()}
                        </select>
                    </label>
                    <label class="form-control w-full">
                        <span class="label-text mb-1">"送金先"</span>
                        <select class="select select-bordered w-full" name="target_account_id" required>
                            {target_options()}
                        </select>
                    </label>
                    <label class="form-control w-full">
                        <span class="label-text mb-1">"金額（整数単位）"</span>
                        <input class="input input-bordered w-full" name="amount" type="number" min="1" step="1" placeholder="例: 500" required/>
                    </label>
                    <button class="btn btn-primary mt-2" type="submit">
                        "送金を確定する"
                        <span aria-hidden="true">"→"</span>
                    </button>
                </form>
                <p class="mt-2 text-xs text-base-content/60">"残高不足・送金上限超過の場合は送金されません。"</p>
            </div>
        </div>
    }
}

#[component]
/// 最近の送金履歴を一覧表示する。
fn ActivityList(activities: Vec<ActivityView>) -> impl IntoView {
    if activities.is_empty() {
        view! { <p class="py-8 text-center text-sm text-base-content/60">"送金履歴はまだありません。"</p> }.into_any()
    } else {
        view! {
            <ul class="list">
                {activities.into_iter().map(|activity| {
                    view! {
                        <li class="list-row items-center gap-3 px-0">
                            <div class="avatar placeholder">
                                <div class="w-10 rounded-full bg-base-200 text-base-content/70">"↗"</div>
                            </div>
                            <div class="min-w-0 flex-1">
                                <div class="font-medium">
                                    {activity.description}
                                </div>
                                <div class="truncate text-xs text-base-content/60">{activity.created_at}</div>
                            </div>
                            <div class="whitespace-nowrap font-semibold text-primary">
                                {activity.amount}
                            </div>
                        </li>
                    }
                }).collect_view()}
            </ul>
        }.into_any()
    }
}

#[component]
/// 送金結果コードを利用者向けDaisyUIアラートへ変換する。
fn StatusMessage(status: Option<TransferStatus>) -> impl IntoView {
    match status {
        Some(status) => {
            let (class, text) = match status {
                TransferStatus::Success => ("alert-success", "送金が完了しました。"),
                TransferStatus::InvalidAmount => {
                    ("alert-error", "金額は1以上の整数で入力してください。")
                }
                TransferStatus::SameAccount => (
                    "alert-error",
                    "送金元と送金先には別の口座を選んでください。",
                ),
                TransferStatus::Threshold => {
                    ("alert-error", "送金額が設定された上限を超えています。")
                }
                TransferStatus::AccountNotFound => {
                    ("alert-error", "指定された口座が見つかりません。")
                }
                TransferStatus::InsufficientFunds => ("alert-error", "残高が不足しています。"),
                TransferStatus::BalanceOverflow => {
                    ("alert-error", "送金後の残高が上限を超えます。")
                }
                TransferStatus::ConcurrentUpdate => (
                    "alert-warning",
                    "口座残高が更新されました。残高を確認して、もう一度お試しください。",
                ),
            };
            view! {
                <div class=format!("alert {} my-6", class) role="status">
                    {text}
                </div>
            }
            .into_any()
        }
        None => view! { <div></div> }.into_any(),
    }
}
