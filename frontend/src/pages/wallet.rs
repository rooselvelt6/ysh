use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::components::ui::toast::ToastCtx;

#[derive(Clone, Copy, PartialEq)]
enum Action { Deposit, Withdraw, Transfer }

#[component]
pub fn WalletPage() -> impl IntoView {
    let (balance, set_balance) = signal(0.0f64);
    let (transactions, set_txs) = signal(Vec::<serde_json::Value>::new());
    let (limits, set_limits) = signal(Option::<serde_json::Value>::None);
    let (loading, set_loading) = signal(true);
    let (tab, set_tab) = signal(0u8); // 0 transactions, 1 limits
    let (modal, set_modal) = signal(Option::<Action>::None);
    let (amount_str, set_amount_str) = signal(String::new());
    let (to_user_str, set_to_user_str) = signal(String::new());
    let (daily_str, set_daily_str) = signal(String::new());
    let (monthly_str, set_monthly_str) = signal(String::new());
    let (busy, set_busy) = signal(false);
    let toast = ToastCtx::use_();

    let load_balance = {
        let set_balance = set_balance.clone();
        move || {
            spawn_local(async move {
                if let Ok(val) = api::get::<serde_json::Value>("/wallet/balance").await {
                    if let Some(b) = val.get("balance").and_then(|v| v.as_f64()) {
                        set_balance.set(b);
                    }
                }
            });
        }
    };
    let load_txs = {
        let set_txs = set_txs.clone();
        move || {
            spawn_local(async move {
                if let Ok(val) = api::get::<serde_json::Value>("/wallet/transactions").await {
                    if let Some(arr) = val.get("transactions").and_then(|v| v.as_array()) {
                        set_txs.set(arr.clone());
                    }
                }
            });
        }
    };
    let load_limits = {
        let set_limits = set_limits.clone();
        move || {
            spawn_local(async move {
                if let Ok(val) = api::get::<serde_json::Value>("/wallet/limits").await {
                    set_limits.set(Some(val));
                }
            });
        }
    };

    // initial load
    {
        let load_balance = load_balance.clone();
        let load_txs = load_txs.clone();
        let load_limits = load_limits.clone();
        spawn_local(async move {
            load_balance();
            load_txs();
            load_limits();
            set_loading.set(false);
        });
    }

    let after_change = {
        let load_balance = load_balance.clone();
        let load_txs = load_txs.clone();
        move || {
            load_balance();
            load_txs();
        }
    };

    let submit_action = move |_: leptos::ev::MouseEvent| {
        let action = match modal.get() {
            Some(a) => a,
            None => return,
        };
        let amount: i64 = match amount_str.get().trim().parse() {
            Ok(n) => n,
            Err(_) => {
                toast.error("Enter a valid amount");
                return;
            }
        };
        if amount <= 0 {
            toast.error("Amount must be positive");
            return;
        }
        set_busy.set(true);
        let after_change = after_change.clone();
        spawn_local(async move {
            let result = match action {
                Action::Deposit => {
                    let req = serde_json::json!({"amount": amount, "description": "Deposit"});
                    api::post::<serde_json::Value>("/wallet/deposit", &req).await
                }
                Action::Withdraw => {
                    let req = serde_json::json!({"amount": amount, "description": "Withdraw"});
                    api::post::<serde_json::Value>("/wallet/withdraw", &req).await
                }
                Action::Transfer => {
                    let to_user: i64 = match to_user_str.get().trim().parse() {
                        Ok(n) => n,
                        Err(_) => {
                            toast.error("Enter a valid recipient user ID");
                            set_busy.set(false);
                            return;
                        }
                    };
                    let req = serde_json::json!({"to_user_id": to_user, "amount": amount, "description": "Transfer"});
                    api::post::<serde_json::Value>("/wallet/transfer", &req).await
                }
            };
            match result {
                Ok(_) => {
                    toast.success("Wallet updated!");
                    set_modal.set(None);
                    set_amount_str.set(String::new());
                    set_to_user_str.set(String::new());
                    after_change();
                }
                Err(e) => toast.error(format!("Failed: {e}")),
            }
            set_busy.set(false);
        });
    };

    let save_limits = move |_: leptos::ev::MouseEvent| {
        let daily: i64 = daily_str.get().trim().parse().unwrap_or(100000);
        let monthly: i64 = monthly_str.get().trim().parse().unwrap_or(1000000);
        spawn_local(async move {
            let req = serde_json::json!({"daily_limit": daily, "monthly_limit": monthly});
            match api::post::<serde_json::Value>("/wallet/limits", &req).await {
                Ok(_) => {
                    toast.success("Limits updated");
                    if let Ok(val) = api::get::<serde_json::Value>("/wallet/limits").await {
                        set_limits.set(Some(val));
                    }
                }
                Err(e) => toast.error(format!("Failed: {e}")),
            }
        });
    };

    view! {
        <div class="main-header">
            <h2>"Wallet"</h2>
        </div>
        <div class="wallet-balance-card">
            <div class="wallet-label">"YSH Balance"</div>
            <div class="wallet-amount">
                {move || format!("{:.2}", balance.get())}
            </div>
            <div style="display:flex;gap:8px;justify-content:center;margin-top:16px;">
                <button class="btn btn-outline btn-sm" on:click=move |_| set_modal.set(Some(Action::Deposit))>"Deposit"</button>
                <button class="btn btn-outline btn-sm" on:click=move |_| set_modal.set(Some(Action::Withdraw))>"Withdraw"</button>
                <button class="btn btn-outline btn-sm" on:click=move |_| set_modal.set(Some(Action::Transfer))>"Transfer"</button>
            </div>
        </div>

        // Action modal
        {move || {
            if let Some(action) = modal.get() {
                let title = match action { Action::Deposit => "Deposit", Action::Withdraw => "Withdraw", Action::Transfer => "Transfer" };
                view! {
                    <div class="modal-overlay" on:click=move |_| set_modal.set(None)>
                        <div class="modal" on:click=move |ev| ev.stop_propagation()>
                            <div class="modal-header">
                                <h2 class="modal-title">{title}</h2>
                                <button class="modal-close" on:click=move |_| set_modal.set(None)>"\u{00d7}"</button>
                            </div>
                            <div style="padding:0 16px 16px;">
                                {move || {
                                    if modal.get() == Some(Action::Transfer) {
                                        view! {
                                            <div class="form-group" style="margin-bottom:16px;">
                                                <label class="form-label">"Recipient user ID"</label>
                                                <input class="form-input" type="text" placeholder="User ID"
                                                    prop:value=move || to_user_str.get()
                                                    on:input=move |ev| set_to_user_str.set(event_target_value(&ev)) />
                                            </div>
                                        }.into_any()
                                    } else { view! {}.into_any() }
                                }}
                                <div class="form-group" style="margin-bottom:16px;">
                                    <label class="form-label">"Amount (YSH)"</label>
                                    <input class="form-input" type="text" placeholder="e.g. 100"
                                        prop:value=move || amount_str.get()
                                        on:input=move |ev| set_amount_str.set(event_target_value(&ev)) />
                                </div>
                                <button class="btn btn-primary" on:click=submit_action
                                    prop:disabled=move || busy.get()>
                                    {move || if busy.get() { "Processing...".to_string() } else { title.to_string() }}
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { view! {}.into_any() }
        }}

        <div class="main-tabs">
            <button class={move || format!("main-tab{}", if tab.get() == 0 { " active" } else { "" })}
                on:click=move |_| set_tab.set(0)>"Transactions"</button>
            <button class={move || format!("main-tab{}", if tab.get() == 1 { " active" } else { "" })}
                on:click=move |_| set_tab.set(1)>"Limits"</button>
        </div>

        {move || {
            if loading.get() {
                view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else if tab.get() == 1 {
                let limits_owned = limits.get();
                let d = limits_owned.as_ref().and_then(|l| l["daily_limit"].as_i64()).unwrap_or(100000);
                let m = limits_owned.as_ref().and_then(|l| l["monthly_limit"].as_i64()).unwrap_or(1000000);
                view! {
                    <div style="padding:16px;">
                        <div class="form-group" style="margin-bottom:16px;">
                            <label class="form-label">"Daily limit"</label>
                            <input class="form-input" type="text" placeholder="Daily" prop:value=move || daily_str.get()
                                on:input=move |ev| set_daily_str.set(event_target_value(&ev)) />
                        </div>
                        <div class="form-group" style="margin-bottom:16px;">
                            <label class="form-label">"Monthly limit"</label>
                            <input class="form-input" type="text" placeholder="Monthly" prop:value=move || monthly_str.get()
                                on:input=move |ev| set_monthly_str.set(event_target_value(&ev)) />
                        </div>
                        <div class="item-meta" style="margin-bottom:12px;">{format!("Current: {d} daily / {m} monthly")}</div>
                        <button class="btn btn-primary" on:click=save_limits>"Save limits"</button>
                    </div>
                }.into_any()
            } else {
                let txs = transactions.get();
                if txs.is_empty() {
                    view! { <div class="empty-state"><div class="empty-state-title">"No transactions yet"</div></div> }.into_any()
                } else {
                    view! {
                        {txs.into_iter().map(|tx| {
                            let tx_type = tx.get("tx_type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                            let amount = tx.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let created = tx.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let (icon, cls) = match tx_type.as_str() {
                                "deposit" => ("\u{1F4B3}", "tx-positive"),
                                "withdrawal" => ("\u{1F4B5}", "tx-negative"),
                                "transfer_in" => ("\u{1F4E5}", "tx-positive"),
                                "transfer_out" => ("\u{1F4E4}", "tx-negative"),
                                "gift" => ("\u{1F381}", "tx-negative"),
                                _ => ("\u{1F4B1}", ""),
                            };
                            view! {
                                <div class="tx-item">
                                    <div class="tx-icon">{icon}</div>
                                    <div class="tx-info">
                                        <div class="tx-type">{tx_type}</div>
                                        <div class="tx-date">{created}</div>
                                    </div>
                                    <span class={format!("tx-amount {}", cls)}>{format!("{amount:.2}")}</span>
                                </div>
                            }
                        }).collect_view()}
                    }.into_any()
                }
            }
        }}
    }
}
