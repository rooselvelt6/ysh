use leptos::prelude::*;
use crate::api;

#[component]
pub fn WalletPage() -> impl IntoView {
    let (balance, set_balance) = signal(Option::<f64>::None);
    let (transactions, set_txs) = signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = signal(true);

    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(val) = api::get::<serde_json::Value>("/wallet/balance").await {
            if let Some(b) = val.get("balance").and_then(|v| v.as_f64()) {
                set_balance.set(Some(b));
            }
        }
        if let Ok(val) = api::get::<serde_json::Value>("/wallet/transactions").await {
            if let Some(arr) = val.get("transactions").and_then(|v| v.as_array()) {
                set_txs.set(arr.clone());
            }
        }
        set_loading.set(false);
    });

    view! {
        <div class="main-header">
            <h2>"Wallet"</h2>
        </div>
        <div class="wallet-balance-card">
            <div class="wallet-label">"YSH Balance"</div>
            <div class="wallet-amount">
                {move || balance.get().map(|b| format!("{b:.2}")).unwrap_or_else(|| "...".into())}
            </div>
            <div style="display:flex;gap:8px;justify-content:center;margin-top:16px;">
                <button class="btn btn-outline btn-sm">"Deposit"</button>
                <button class="btn btn-outline btn-sm">"Withdraw"</button>
                <button class="btn btn-outline btn-sm">"Transfer"</button>
            </div>
        </div>
        <div class="main-tabs">
            <button class="main-tab active">"Transactions"</button>
            <button class="main-tab">"Limits"</button>
        </div>
        {move || {
            if loading.get() {
                view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else {
                let txs = transactions.get();
                if txs.is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-state-title">"No transactions yet"</div>
                        </div>
                    }.into_any()
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
