use leptos::prelude::*;
use leptos::tachys::view::any_view::AnyView;
use wasm_bindgen_futures::spawn_local;

use crate::api;

use super::{f, fmt_bytes, fmt_int, fmt_pct, num};

#[component]
pub fn OverviewTab() -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (realtime, set_realtime) = signal(serde_json::Value::Null);
    let (health, set_health) = signal(serde_json::Value::Null);
    let (stats, set_stats) = signal(serde_json::Value::Null);
    let (revenue, set_revenue) = signal(serde_json::Value::Null);
    let (call_stats, set_call_stats) = signal(serde_json::Value::Null);

    spawn_local(async move {
        set_loading.set(true);
        if let Ok(v) = api::get::<serde_json::Value>("/admin/analytics/realtime").await {
            set_realtime.set(v);
        }
        if let Ok(v) = api::get::<serde_json::Value>("/admin/analytics/health").await {
            set_health.set(v);
        }
        if let Ok(v) = api::get::<serde_json::Value>("/admin/stats").await {
            set_stats.set(v);
        }
        if let Ok(v) = api::get::<serde_json::Value>("/admin/analytics/revenue?days=30").await {
            set_revenue.set(v);
        }
        if let Ok(v) = api::get::<serde_json::Value>("/admin/calls").await {
            if let Some(st) = v.get("stats") {
                set_call_stats.set(st.clone());
            }
        }
        set_loading.set(false);
    });

    view! {
        {move || {
            if loading.get() {
                return view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any();
            }
            let rt = realtime.get();
            let h = health.get();
            let st = stats.get();
            let rv = revenue.get();
            let cs = call_stats.get();

            let db_info = rt.get("db").cloned().unwrap_or_default();
            let today = rt.get("today").cloned().unwrap_or_default();

            let platform: Vec<AnyView> = vec![
                tile("Usuarios", fmt_int(num(&st, "users"))),
                tile("Agencias", fmt_int(num(&st, "agencies"))),
                tile("Hosts", fmt_int(num(&st, "hosts"))),
                tile("Momentos", fmt_int(num(&st, "moments"))),
                tile("Regalos", fmt_int(num(&st, "gifts"))),
                tile("Notificaciones", fmt_int(num(&st, "notifications"))),
                tile("Volumen total (YSH)", fmt_int(num(&rv, "gross_volume"))),
                tile("ARPU (YSH)", fmt_int(num(&rv, "arpu"))),
            ];

            let rt_rows: Vec<AnyView> = vec![
                tile("Usuarios online", fmt_int(num(&rt, "online_users"))),
                tile("Salas activas", fmt_int(num(&rt, "active_rooms"))),
                tile("Llamadas activas", fmt_int(num(&db_info, "active_calls"))),
                tile("Reportes pendientes", fmt_int(num(&db_info, "pending_reports"))),
                tile("Cache entries", fmt_int(num(&rt, "cache_entries"))),
                tile("Transacciones hoy", fmt_int(num(&today, "transactions"))),
                tile("Regalos hoy", fmt_int(num(&today, "gifts"))),
                tile("Total llamadas", fmt_int(num(&cs, "total_calls"))),
            ];

            let uptime = f(&h, "uptime_secs");
            let cpu = f(&h, "cpu_usage_pct");
            let mem = h
                .get("memory")
                .and_then(|m| m.get("used_pct"))
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0);
            let db_size = h.get("db_size_bytes").and_then(|x| x.as_u64()).unwrap_or(0);
            let threads = num(&h, "threads");

            view! {
                <div style="margin-top:16px;">
                    <div class="panel">
                        <div class="panel-title">"Plataforma"</div>
                        <div class="stat-grid">{platform.into_iter().collect_view()}</div>
                    </div>
                    <div class="panel">
                        <div class="panel-title">"Actividad en vivo"</div>
                        <div class="stat-grid">{rt_rows.into_iter().collect_view()}</div>
                    </div>
                    <div class="panel">
                        <div class="panel-title">"Salud del sistema"</div>
                        <div class="stat-grid">
                            {vec![
                                ("Uptime (s)", format!("{uptime:.0}")),
                                ("CPU", fmt_pct(cpu)),
                                ("Memoria", fmt_pct(mem)),
                                ("Tamaño DB", fmt_bytes(db_size)),
                                ("Hilos", fmt_int(threads)),
                            ].into_iter().map(|(label, value)| {
                                view! {
                                    <div class="stat-tile">
                                        <div class="stat-value">{value}</div>
                                        <div class="stat-label">{label}</div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            }.into_any()
        }}
    }
}

fn tile(label: &'static str, value: String) -> AnyView {
    view! {
        <div class="stat-tile">
            <div class="stat-value">{value}</div>
            <div class="stat-label">{label}</div>
        </div>
    }
    .into_any()
}