use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::ui::toast::ToastCtx;

use super::{fmt_amt, fmt_int, num, s, truncate};

#[component]
pub fn WalletsTab() -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (wallets, set_wallets) = signal(Vec::<serde_json::Value>::new());
    let (txs, set_txs) = signal(Vec::<serde_json::Value>::new());
    let (receipts, set_receipts) = signal(Vec::<serde_json::Value>::new());
    let (fraud, set_fraud) = signal(Vec::<serde_json::Value>::new());
    let toast = ToastCtx::use_();

    let reload = {
        let set_loading = set_loading.clone();
        move || {
            set_loading.set(true);
            spawn_local(async move {
                if let Ok(v) = api::get::<serde_json::Value>("/admin/wallets").await {
                    if let Some(arr) = v.get("wallets").and_then(|x| x.as_array()) {
                        set_wallets.set(arr.clone());
                    }
                }
                if let Ok(v) = api::get::<serde_json::Value>("/admin/transactions?limit=150").await {
                    if let Some(arr) = v.get("transactions").and_then(|x| x.as_array()) {
                        set_txs.set(arr.clone());
                    }
                }
                if let Ok(v) = api::get::<serde_json::Value>("/admin/receipts?limit=60").await {
                    if let Some(arr) = v.get("receipts").and_then(|x| x.as_array()) {
                        set_receipts.set(arr.clone());
                    }
                }
                if let Ok(v) = api::get::<serde_json::Value>("/admin/fraud").await {
                    if let Some(arr) = v.get("alerts").and_then(|x| x.as_array()) {
                        set_fraud.set(arr.clone());
                    }
                }
                set_loading.set(false);
            });
        }
    };
    reload();

    let act = move |path: String, body: serde_json::Value, reload: Box<dyn Fn() + 'static>| {
        spawn_local(async move {
            match api::post::<serde_json::Value>(&path, &body).await {
                Ok(v) => {
                    let msg = s(&v, "message");
                    toast.success(if msg.is_empty() { "OK".into() } else { msg });
                    reload();
                }
                Err(e) => toast.error(format!("Failed: {e}")),
            }
        });
    };

    view! {
        {move || {
            if loading.get() {
                return view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any();
            }
            let ws = wallets.get();
            let txs_all = txs.get();
            let rec = receipts.get();
            let fa = fraud.get();

            let mut total_balance: i64 = 0;
            for w in &ws {
                total_balance += num(w, "balance");
            }
            let frozen_count = ws.iter().filter(|w| w.get("frozen").and_then(|x| x.as_bool()).unwrap_or(false)).count();

            view! {
                <div style="margin-top:16px;">
                    <div class="stat-grid">
                        {vec![
                            ("Wallets", fmt_int(ws.len() as i64)),
                            ("Saldo total (YSH)", fmt_amt(total_balance)),
                            ("Congeladas", fmt_int(frozen_count as i64)),
                            ("Transacciones", fmt_int(txs_all.len() as i64)),
                            ("Recibos", fmt_int(rec.len() as i64)),
                            ("Alertas fraude", fmt_int(fa.len() as i64)),
                        ].into_iter().map(|(label, value)| {
                            view! {
                                <div class="stat-tile">
                                    <div class="stat-value">{value}</div>
                                    <div class="stat-label">{label}</div>
                                </div>
                            }
                        }).collect_view()}
                    </div>

                    <button class="btn btn-primary btn-sm" style="margin:14px 0;" on:click=move |_| reload()>
                        "\u{1F504} Recargar todo"
                    </button>

                    <div class="panel">
                        <div class="panel-title">"Wallets"</div>
                        <table class="data-table">
                            <thead>
                                <tr><th>"ID"</th><th>"Usuario"</th><th>"Saldo (YSH)"</th><th>"Frozen"</th><th>"Actualizado"</th><th>"Acciones"</th></tr>
                            </thead>
                            <tbody>
                                {ws.iter().map(|w| {
                                    let w = w.clone();
                                    let uid = num(&w, "user_id");
                                    let frozen = w.get("frozen").and_then(|x| x.as_bool()).unwrap_or(false);
                                    let reload = reload.clone();
                                    let act = act.clone();
                                    view! {
                                        <tr>
                                            <td>#{fmt_int(uid)}</td>
                                            <td><strong>{s(&w, "username")}</strong></td>
                                            <td>{fmt_amt(num(&w, "balance"))}</td>
                                            <td>{
                                                if frozen { "\u{274C} si" } else { "\u{2705} no" }
                                            }</td>
                                            <td style="color:var(--text-dim);font-size:0.8rem;">{s(&w, "updated_at")}</td>
                                            <td>
                                                <div style="display:flex;gap:6px;flex-wrap:wrap;">
                                                    <button
                                                        class="btn btn-sm btn-outline"
                                                        on:click={let act = act.clone(); let reload = reload.clone(); move |_| {
                                                            let act = act.clone();
                                                            let reload = reload.clone();
                                                            let body = serde_json::json!({});
                                                            let p = if frozen { format!("/admin/wallet/{uid}/unfreeze") } else { format!("/admin/wallet/{uid}/freeze") };
                                                            act(p, body, Box::new(move || reload()));
                                                        }}
                                                    >
                                                        {if frozen { "Descongelar" } else { "Congelar" }}
                                                    </button>
                                                    <button
                                                        class="btn btn-sm"
                                                        on:click={let act = act.clone(); let reload = reload.clone(); move |_| {
                                                            let act = act.clone();
                                                            let reload = reload.clone();
                                                            let prompt = web_sys::window().and_then(|w| w.prompt_with_message("Depósito YSH centavos (1000 = 10.00)").ok().flatten());
                                                            if let Some(am) = prompt {
                                                                if let Ok(amount) = am.trim().parse::<i64>() {
                                                                    if amount != 0 {
                                                                        let body = serde_json::json!({"amount": amount, "description": "admin deposit"});
                                                                        act(format!("/admin/wallet/{uid}/adjust"), body, Box::new(move || reload()));
                                                                    }
                                                                }
                                                            }
                                                        }}
                                                    >
                                                        "+ Depósito"
                                                    </button>
                                                    <button
                                                        class="btn btn-sm"
                                                        on:click={let act = act.clone(); let reload = reload.clone(); move |_| {
                                                            let act = act.clone();
                                                            let reload = reload.clone();
                                                            let prompt = web_sys::window().and_then(|w| w.prompt_with_message("Retiro YSH centavos (1000 = 10.00)").ok().flatten());
                                                            if let Some(am) = prompt {
                                                                if let Ok(amount) = am.trim().parse::<i64>() {
                                                                    if amount != 0 {
                                                                        let body = serde_json::json!({"amount": -1 * amount.abs(), "description": "admin withdraw"});
                                                                        act(format!("/admin/wallet/{uid}/adjust"), body, Box::new(move || reload()));
                                                                    }
                                                                }
                                                            }
                                                        }}
                                                    >
                                                        "- Retiro"
                                                    </button>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>

                    <div class="panel">
                        <div class="panel-title">"Transacciones"</div>
                        <table class="data-table">
                            <thead>
                                <tr><th>"ID"</th><th>"Usuario"</th><th>"Tipo"</th><th>"Monto (YSH)"</th><th>"Descripción"</th><th>"Fecha"</th></tr>
                            </thead>
                            <tbody>
                                {txs_all.iter().map(|t| {
                                    let t = t.clone();
                                    let amt = num(&t, "amount");
                                    let cls = if amt < 0 { "tx-amount neg" } else { "tx-amount pos" };
                                    view! {
                                        <tr>
                                            <td>#{fmt_int(num(&t, "id"))}</td>
                                            <td>{truncate(&s(&t, "username"), 18)}</td>
                                            <td><span class="badge">{s(&t, "tx_type")}</span></td>
                                            <td class={cls}>{fmt_amt(amt)}</td>
                                            <td style="color:var(--text-dim);">{truncate(&s(&t, "description"), 28)}</td>
                                            <td style="color:var(--text-dim);font-size:0.8rem;">{s(&t, "created_at")}</td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>

                    <div class="panel">
                        <div class="panel-title">"Recibos verificables"</div>
                        <table class="data-table">
                            <thead>
                                <tr><th>"ID"</th><th>"Usuario"</th><th>"Tipo"</th><th>"Ref"</th><th>"Monto"</th><th>"Hash"</th></tr>
                            </thead>
                            <tbody>
                                {rec.iter().map(|r| {
                                    let r = r.clone();
                                    view! {
                                        <tr>
                                            <td>#{fmt_int(num(&r, "id"))}</td>
                                            <td>{truncate(&s(&r, "username"), 18)}</td>
                                            <td>{s(&r, "receipt_type")}</td>
                                            <td>{fmt_int(num(&r, "reference_id"))}</td>
                                            <td>{fmt_amt(num(&r, "amount"))} {s(&r, "currency")}</td>
                                            <td style="color:var(--text-dim);font-size:0.7rem;">{truncate(&s(&r, "receipt_hash"), 22)}</td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>

                    <div class="panel">
                        <div class="panel-title">"Alertas de fraude"</div>
                        {if fa.is_empty() {
                            view! { <div class="item-meta">"No hay alertas de fraude."</div> }.into_any()
                        } else {
                            view! {
                                <table class="data-table">
                                    <thead>
                                        <tr><th>"ID"</th><th>"Usuario"</th><th>"Tipo"</th><th>"Severidad"</th><th>"Descripción"</th><th>"Estado"</th><th>"Acción"</th></tr>
                                    </thead>
                                    <tbody>
                                        {fa.iter().map(|a| {
                                            let a = a.clone();
                                            let id = num(&a, "id");
                                            let status = s(&a, "status");
                                            let reload = reload.clone();
                                            view! {
                                                <tr>
                                                    <td>#{fmt_int(id)}</td>
                                                    <td>{match a.get("user_id").and_then(|x| x.as_i64()) { Some(u) => format!("#{u}"), None => "—".into() }}</td>
                                                    <td>{s(&a, "alert_type")}</td>
                                                    <td>{s(&a, "severity")}</td>
                                                    <td style="color:var(--text-dim);">{truncate(&s(&a, "description"), 40)}</td>
                                                    <td><span class="badge">{status.clone()}</span></td>
                                                    <td>{
                                                        if status == "open" {
                                                            let reload = reload.clone();
                                                            view! {
                                                                <button
                                                                    class="btn btn-sm"
                                                                    on:click=move |_| {
                                                                        let reload = reload.clone();
                                                                        let reload2 = reload.clone();
                                                                        spawn_local(async move {
                                                                            match api::post::<serde_json::Value>(&format!("/admin/fraud/{id}/resolve"), &serde_json::json!({})).await {
                                                                                Ok(_) => { toast.success("Alerta resuelta"); reload(); }
                                                                                Err(e) => { toast.error(format!("Failed: {e}")); reload2(); }
                                                                            }
                                                                        });
                                                                    }
                                                                >
                                                                    "Resolver"
                                                                </button>
                                                            }.into_any()
                                                        } else {
                                                            view! {}.into_any()
                                                        }
                                                    }</td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_any()
                        }}
                    </div>
                </div>
            }.into_any()
        }}
    }
}