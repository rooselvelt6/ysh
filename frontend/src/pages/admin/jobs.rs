use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::ui::toast::ToastCtx;

use super::{fmt_int, num, s, truncate};

const JOB_NAMES: [&str; 6] = [
    "payouts",
    "staking",
    "moderation",
    "cleanup",
    "notifications",
    "analytics",
];

#[component]
pub fn JobsTab() -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (stats, set_stats) = signal(serde_json::Value::Null);
    let toast = ToastCtx::use_();

    let reload = {
        let set_loading = set_loading.clone();
        move || {
            set_loading.set(true);
            spawn_local(async move {
                if let Ok(v) = api::get::<serde_json::Value>("/admin/jobs/stats").await {
                    set_stats.set(v);
                }
                set_loading.set(false);
            });
        }
    };
    reload();

    let run_job = move |name: String| {
        let set_loading = set_loading.clone();
        let reload = reload.clone();
        spawn_local(async move {
            if let Ok(_v) = api::post::<serde_json::Value>(&format!("/admin/jobs/run/{name}"), &serde_json::json!({})).await {
                toast.success(format!("Job {name} lanzado"));
                set_loading.set(false);
                reload();
            } else {
                toast.error(format!("No se pudo lanzar {name}"));
                set_loading.set(false);
            }
        });
    };

    view! {
        {move || {
            if loading.get() {
                return view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any();
            }
            let st = stats.get();
            let enabled = st.get("enabled").and_then(|x| x.as_bool()).unwrap_or(false);
            let interval = num(&st, "interval_secs");
            let jobs = st.get("jobs").cloned().unwrap_or_default();
            let names = {
                let mut n: Vec<String> = jobs.as_object().map(|o| o.keys().cloned().collect()).unwrap_or_default();
                for j in JOB_NAMES {
                    if !n.iter().any(|x| x == j) {
                        n.push(j.to_string());
                    }
                }
                n
            };

            view! {
                <div style="margin-top:16px;">
                    <div class="stat-grid">
                        {vec![
                            ("Worker activo", if enabled { "\u{2705} si" } else { "\u{274C} no" }.to_string()),
                            ("Intervalo (s)", fmt_int(interval)),
                            ("Registros de ejecución", fmt_int(jobs.as_object().map(|o| o.len()).unwrap_or(0) as i64)),
                        ].into_iter().map(|(label, value)| {
                            view! {
                                <div class="stat-tile">
                                    <div class="stat-value">{value}</div>
                                    <div class="stat-label">{label}</div>
                                </div>
                            }
                        }).collect_view()}
                    </div>

                    <div class="panel" style="margin-top:16px;">
                        <div class="panel-title">"Ejecutar job manualmente"</div>
                        <div style="display:flex;gap:8px;flex-wrap:wrap;">
                            {names.iter().map(|name| {
                                let name = name.clone();
                                let run_job = run_job.clone();
                                view! {
                                    <button
                                        class="btn btn-outline btn-sm"
                                        on:click=move |_| run_job(name.clone())
                                    >
                                        "\u{25B6} " {name.clone()}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    {if jobs.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                        view! {
                            <div class="empty-state" style="margin-top:24px;">
                                <div class="empty-state-icon">"\u{1F4C8}"</div>
                                <div class="empty-state-title">"Sin ejecuciones registradas"</div>
                                <div class="empty-state-text">"Lanza un job para ver su resultado aquí."</div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="panel">
                                <div class="panel-title">"Últimos resultados"</div>
                                {names.iter().map(|name| {
                                    let name = name.clone();
                                    let entry = jobs.get(&name).cloned().unwrap_or_default();
                                    let runs = num(&entry, "runs");
                                    let run_at = s(&entry, "run_at");
                                    let result = entry.get("result").cloned().unwrap_or_default();
                                    let result_s = serde_json::to_string_pretty(&result).unwrap_or_default();
                                    view! {
                                        <div class="panel" style="box-shadow:none;border:1px solid var(--border);margin-bottom:10px;">
                                            <div style="display:flex;justify-content:space-between;align-items:center;">
                                                <strong>{name.clone()}</strong>
                                                <span style="color:var(--text-dim);font-size:0.8rem;">
                                                    {if runs > 0 { format!("{runs} ejecuciones") } else { "no ejecutado".into() }}
                                                    {if !run_at.is_empty() { format!(" · {run_at}") } else { String::new() }}
                                                </span>
                                            </div>
                                            {if !result_s.is_empty() {
                                                view! {
                                                    <pre style="background:var(--bg);border-radius:10px;padding:10px;font-size:0.75rem;margin-top:8px;overflow-x:auto;white-space:pre-wrap;color:var(--text-dim);">{truncate(&result_s, 600)}</pre>
                                                }.into_any()
                                            } else {
                                                view! {}.into_any()
                                            }}
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }}
                </div>
            }.into_any()
        }}
    }
}