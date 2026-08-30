use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::ui::toast::ToastCtx;

use super::{fmt_amt, fmt_int, num, s, truncate};

#[component]
pub fn PayoutsTab() -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (payouts, set_payouts) = signal(Vec::<serde_json::Value>::new());
    let toast = ToastCtx::use_();

    let reload = {
        let set_loading = set_loading.clone();
        move || {
            set_loading.set(true);
            spawn_local(async move {
                if let Ok(v) = api::get::<serde_json::Value>("/admin/payouts").await {
                    if let Some(arr) = v.get("payouts").and_then(|x| x.as_array()) {
                        set_payouts.set(arr.clone());
                    }
                }
                set_loading.set(false);
            });
        }
    };
    reload();

    let process = move |payout_id: i64, approved: bool| {
        let set_loading = set_loading.clone();
        let reload = reload.clone();
        spawn_local(async move {
            let tx_hash = if approved {
                web_sys::window()
                    .and_then(|w| w.prompt_with_message("Tx hash (opcional):").ok().flatten())
                    .filter(|h| !h.trim().is_empty())
                    .unwrap_or_else(|| format!("manual-{payout_id}"))
            } else {
                String::new()
            };
            let body = serde_json::json!({"payout_id": payout_id, "approved": approved, "tx_hash": tx_hash});
            match api::post::<serde_json::Value>("/admin/payouts/process", &body).await {
                Ok(v) => {
                    let msg = s(&v, "status");
                    toast.success(if approved { format!("Pago aprobado ({msg})") } else { "Pago rechazado".into() });
                    set_loading.set(false);
                    reload();
                }
                Err(e) => {
                    toast.error(format!("Failed: {e}"));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        {move || {
            if loading.get() {
                return view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any();
            }
            let list = payouts.get();
            let pending = list.iter().filter(|p| s(p, "status") == "pending").count();

            view! {
                <div style="margin-top:16px;">
                    <div class="stat-grid">
                        {vec![
                            ("Pagos totales", fmt_int(list.len() as i64)),
                            ("Pendientes", fmt_int(pending as i64)),
                            ("Completados", fmt_int(list.iter().filter(|p| s(p, "status") == "completed").count() as i64)),
                            ("Rechazados", fmt_int(list.iter().filter(|p| s(p, "status") == "rejected").count() as i64)),
                            ("Monto pendiente (YSH)", fmt_amt(list.iter().filter(|p| s(p, "status") == "pending").map(|p| num(p, "amount")).sum())),
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
                        "\u{1F504} Recargar"
                    </button>

                    {if list.is_empty() {
                        view! {
                            <div class="empty-state" style="margin-top:24px;">
                                <div class="empty-state-icon">"\u{1F4E9}"</div>
                                <div class="empty-state-title">"Sin pagos"</div>
                                <div class="empty-state-text">"No hay solicitudes de pago todavía."</div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="panel">
                                <div class="panel-title">"Solicitudes de pago"</div>
                                <table class="data-table">
                                    <thead>
                                        <tr><th>"ID"</th><th>"Usuario"</th><th>"Monto (YSH)"</th><th>"Red"</th><th>"Wallet"</th><th>"Estado"</th><th>"Pedido"</th><th>"Acción"</th></tr>
                                    </thead>
                                    <tbody>
                                        {list.iter().map(|p| {
                                            let p = p.clone();
                                            let id = num(&p, "id");
                                            let status = s(&p, "status");
                                            let process = process.clone();
                                            view! {
                                                <tr>
                                                    <td>#{fmt_int(id)}</td>
                                                    <td><strong>{s(&p, "username")}</strong></td>
                                                    <td>{fmt_amt(num(&p, "amount"))}</td>
                                                    <td>{s(&p, "network")}</td>
                                                    <td style="font-size:0.75rem;max-width:160px;overflow:hidden;text-overflow:ellipsis;">{truncate(&s(&p, "wallet_address"), 20)}</td>
                                                    <td><span class="badge">{status.clone()}</span></td>
                                                    <td style="color:var(--text-dim);font-size:0.8rem;">{s(&p, "requested_at")}</td>
                                                    <td>{
                                                        if status == "pending" {
                                                            let process2 = process.clone();
                                                            view! {
                                                                <div style="display:flex;gap:6px;">
                                                                    <button
                                                                        class="btn btn-sm"
                                                                        on:click=move |_| process(id, true)
                                                                    >
                                                                        "\u{2713} Aprobar"
                                                                    </button>
                                                                    <button
                                                                        class="btn btn-sm btn-outline"
                                                                        on:click=move |_| process2(id, false)
                                                                    >
                                                                        "\u{2717} Rechazar"
                                                                    </button>
                                                                </div>
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
                            </div>
                        }.into_any()
                    }}
                </div>
            }.into_any()
        }}
    }
}