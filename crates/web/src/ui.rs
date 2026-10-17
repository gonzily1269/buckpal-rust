//! DaisyUIを使った画面と、その画面を構成するLeptosコンポーネント。

use leptos::prelude::*;

use super::{
    PageData,
    messages::{Messages, format_message},
    view_model::{AccountView, ActivityView},
};

#[component]
/// 口座残高、送金フォーム、最近の送金履歴を表示するLeptos画面。
pub(super) fn App() -> impl IntoView {
    let Some(page) = use_context::<PageData>() else {
        tracing::error!("page data was not provided during SSR");
        return None;
    };
    let Some(messages) = use_context::<std::sync::Arc<Messages>>() else {
        tracing::error!("messages were not provided during SSR");
        return None;
    };

    Some(view! {
        <html lang="ja" data-theme="light">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <title>{messages.page.title.clone()}</title>
                <link href="https://cdn.jsdelivr.net/npm/daisyui@5.7.50" rel="stylesheet" type="text/css"/>
                <script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4.3.3"></script>
            </head>
            <body>
                <main class="container mx-auto max-w-6xl px-4 sm:px-6">
                    <header class="navbar border-b border-base-300 px-0">
                        <a class="text-2xl font-extrabold tracking-tight text-primary" href="/">
                            {messages.page.brand.clone()}<span class="text-secondary">{messages.page.brand_suffix.clone()}</span>
                        </a>
                        <div class="badge badge-success badge-outline gap-2">
                            <span class="status status-success"></span>
                            {messages.page.connection_status.clone()}
                        </div>
                    </header>
                    <section class="py-12 sm:py-14">
                        <p class="mb-3 text-xs font-bold tracking-[0.16em] text-base-content/60">{messages.page.hero_eyebrow.clone()}</p>
                        <h1 class="text-3xl font-bold tracking-tight text-primary sm:text-4xl">{messages.page.hero_title.clone()}</h1>
                        <p class="mt-3 text-base text-base-content/70">{messages.page.hero_subtitle.clone()}</p>
                    </section>
                    <StatusMessage status=page.status messages=messages.clone()/>
                    <section class="space-y-4">
                        <div class="flex flex-wrap items-end justify-between gap-2">
                            <div>
                                <p class="mb-2 text-xs font-bold tracking-[0.16em] text-base-content/60">{messages.page.accounts_eyebrow.clone()}</p>
                                <h2 class="text-2xl font-semibold">{messages.page.accounts_title.clone()}</h2>
                            </div>
                            <span class="badge badge-ghost">{format_message(&messages.page.account_count, &[&page.dashboard.accounts.len().to_string()])}</span>
                        </div>
                        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                            {page.dashboard.accounts.iter().cloned().map(|account| view! { <AccountCard account=account messages=messages.clone()/> }).collect_view()}
                        </div>
                    </section>
                    <section class="mt-8 grid grid-cols-1 gap-5 lg:grid-cols-2">
                        <TransferForm accounts=page.dashboard.accounts.clone() messages=messages.clone()/>
                        <div class="card bg-base-100 shadow-sm">
                            <div class="card-body">
                                <div class="mb-2">
                                    <p class="mb-2 text-xs font-bold tracking-[0.16em] text-base-content/60">{messages.page.recent_transfers_eyebrow.clone()}</p>
                                    <h2 class="card-title">{messages.page.recent_transfers_title.clone()}</h2>
                                </div>
                                <ActivityList activities=page.dashboard.activities.clone() messages=messages.clone()/>
                            </div>
                        </div>
                    </section>
                    <footer class="py-8 text-center text-xs tracking-widest text-base-content/50">
                        {messages.page.footer_brand.clone()}
                        <span class="px-2">{messages.page.footer_separator.clone()}</span>
                        {messages.page.footer_caption.clone()}
                    </footer>
                </main>
            </body>
        </html>
    })
}

#[component]
/// 口座名と利用可能残高を表示するDaisyUIカード。
fn AccountCard(account: AccountView, messages: std::sync::Arc<Messages>) -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body gap-4">
                <div class="flex items-center justify-between">
                    <span class="avatar placeholder"><span class="rounded-xl bg-primary/10 text-primary">"↗"</span></span>
                    <span class="badge badge-ghost">{account.id_label}</span>
                </div>
                <h3 class="text-lg font-semibold">{account.name}</h3>
                <div class="stat p-0">
                    <div class="stat-title">{messages.account.balance_label.clone()}</div>
                    <div class="stat-value text-primary">{account.balance}</div>
                    <div class="stat-desc">{messages.account.units.clone()}</div>
                </div>
            </div>
        </div>
    }
}

#[component]
/// DaisyUIのフォーム部品を使った送金フォーム。
fn TransferForm(accounts: Vec<AccountView>, messages: std::sync::Arc<Messages>) -> impl IntoView {
    let source_options = accounts
        .iter()
        .map(|account| {
            let id = account.id.to_string();
            let label = format_message(
                &messages.transfer.account_option,
                &[&account.name, &account.balance],
            );
            view! { <option value=id>{label}</option> }
        })
        .collect_view();
    let target_options = accounts
        .iter()
        .map(|account| {
            let id = account.id.to_string();
            let name = account.name.clone();
            view! { <option value=id>{name}</option> }
        })
        .collect_view();
    let amount_placeholder = messages.transfer.amount_placeholder.clone();

    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body">
                <div class="mb-2">
                    <p class="mb-2 text-xs font-bold tracking-[0.16em] text-base-content/60">{messages.page.transfer_eyebrow.clone()}</p>
                    <h2 class="card-title">{messages.transfer.title.clone()}</h2>
                    <p class="mt-2 text-sm text-base-content/70">{messages.transfer.description.clone()}</p>
                </div>
                <form method="post" action="/" class="flex flex-col gap-3">
                    <label class="form-control w-full">
                        <span class="label-text mb-1">{messages.transfer.source_label.clone()}</span>
                        <select class="select select-bordered w-full" name="source_account_id" required>
                            {source_options}
                        </select>
                    </label>
                    <label class="form-control w-full">
                        <span class="label-text mb-1">{messages.transfer.target_label.clone()}</span>
                        <select class="select select-bordered w-full" name="target_account_id" required>
                            {target_options}
                        </select>
                    </label>
                    <label class="form-control w-full">
                        <span class="label-text mb-1">{messages.transfer.amount_label.clone()}</span>
                        <input class="input input-bordered w-full" name="amount" type="number" min="1" step="1" placeholder=amount_placeholder required/>
                    </label>
                    <button class="btn btn-primary mt-2" type="submit">
                        {messages.transfer.submit.clone()}
                        <span aria-hidden="true">{messages.transfer.submit_arrow.clone()}</span>
                    </button>
                </form>
                <p class="mt-2 text-xs text-base-content/60">{messages.transfer.hint.clone()}</p>
            </div>
        </div>
    }
}

#[component]
/// 最近の送金履歴を一覧表示する。
fn ActivityList(
    activities: Vec<ActivityView>,
    messages: std::sync::Arc<Messages>,
) -> impl IntoView {
    if activities.is_empty() {
        view! { <p class="py-8 text-center text-sm text-base-content/60">{messages.activity.empty.clone()}</p> }.into_any()
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
fn StatusMessage(status: Option<String>, messages: std::sync::Arc<Messages>) -> impl IntoView {
    match status.as_deref().and_then(|code| messages.status(code)) {
        Some(message) => view! {
            <div class=format!("alert {} my-6", message.class) role="status">
                {message.text.clone()}
            </div>
        }
        .into_any(),
        None => view! { <div></div> }.into_any(),
    }
}
