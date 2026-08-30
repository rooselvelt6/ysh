use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;

use super::{fmt_int, num, s, truncate};

#[component]
pub fn CallsTab() -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (calls, set_calls) = signal(Vec::<serde_json::Value>::new());
    let (stats, set_stats) = signal(serde_json::Value::Null);

    let reload = {
        let set_loading = set_loading.clone();
        move || {
            set_loading.set(true);
            spawn_local(async move {
                if let Ok(v) = api::get::<serde_json::Value>("/admin/calls?limit=150").await {
                    if let Some(arr) = v.get("calls").and_then(|x| x.as_array()) {
                        set_calls.set(arr.clone());
                    }
                    if let Some(st) = v.get("stats") {
                        set_stats.set(st.clone());
                    }
                }
                set_loading.set(false);
            });
        }
    };
    reload();

    view! {
        {move || {
            if loading.get() {
                return view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any();
            }
            let list = calls.get();
            let st = stats.get();
            let active = list.iter().filter(|c| s(c, "status") == "active").count();
            let dur_min = num(&st, "total_duration_secs") / 60;

            view! {
                <div style="margin-top:16px;">
                    <div class="stat-grid">
                        {vec![
                            ("Llamadas totales", fmt_int(num(&st, "total_calls"))),
                            ("Activas ahora", fmt_int(active as i64)),
                            ("Duración total (min)", fmt_int(dur_min)),
                            ("Historial cargado", fmt_int(list.len() as i64)),
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
                                <div class="empty-state-icon">"\u{260E}\u{FE0F}"</div>
                                <div class="empty-state-title">"Sin llamadas"</div>
                                <div class="empty-state-text">"Las llamadas WebRTC completadas aparecerán aquí."</div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="panel">
                                <div class="panel-title">"Registro de llamadas"</div>
                                <table class="data-table">
                                    <thead>
                                        <tr><th>"Call ID"</th><th>"Host"</th><th>"Tipo"</th><th>"Estado"</th><th>"Participantes"</th><th>"Duración (s)"</th><th>"Facturado"</th><th>"Inicio"</th></tr>
                                    </thead>
                                    <tbody>
                                        {list.iter().map(|c| {
                                            let c = c.clone();
                                            let participants = c.get("participants").and_then(|x| x.as_array()).map(|a| a.len()).unwrap_or(0);
                                            let status = s(&c, "status");
                                            let status2 = status.clone();
                                            view! {
                                                <tr>
                                                    <td style="font-size:0.75rem;">{truncate(&s(&c, "call_id"), 16)}</td>
                                                    <td>{truncate(&s(&c, "host_username"), 16)} (#{fmt_int(num(&c, "host_id"))})</td>
                                                    <td><span class="badge">{s(&c, "call_type")}</span></td>
                                                    <td><span class={format!("badge{}", if status == "active" { " badge-active" } else { "" })}>{status2}</span></td>
                                                    <td>{fmt_int(participants as i64)}</td>
                                                    <td>{fmt_int(num(&c, "duration_secs"))}</td>
                                                    <td>{fmt_int(num(&c, "billed"))}</td>
                                                    <td style="color:var(--text-dim);font-size:0.8rem;">{s(&c, "started_at")}</td>
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