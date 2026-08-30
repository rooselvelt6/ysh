use leptos::prelude::*;
use leptos::tachys::view::any_view::AnyView;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::ui::toast::ToastCtx;

use super::{fmt_int, f, num, s, truncate};

#[component]
pub fn ModerationTab() -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (queue, set_queue) = signal(Vec::<serde_json::Value>::new());
    let (reports, set_reports) = signal(Vec::<serde_json::Value>::new());
    let (flags, set_flags) = signal(Vec::<serde_json::Value>::new());
    let (appeals, set_appeals) = signal(Vec::<serde_json::Value>::new());
    let (bans, set_bans) = signal(Vec::<serde_json::Value>::new());
    let toast = ToastCtx::use_();

    let reload = {
        let set_loading = set_loading.clone();
        move || {
            set_loading.set(true);
            spawn_local(async move {
                if let Ok(v) = api::get::<serde_json::Value>("/admin/moderation/queue?status=pending").await {
                    if let Some(arr) = v.get("queue").and_then(|x| x.as_array()) {
                        set_queue.set(arr.clone());
                    }
                }
                if let Ok(v) = api::get::<serde_json::Value>("/admin/moderation/reports?status=pending").await {
                    if let Some(arr) = v.get("reports").and_then(|x| x.as_array()) {
                        set_reports.set(arr.clone());
                    }
                }
                if let Ok(v) = api::get::<serde_json::Value>("/admin/moderation/flags?status=pending").await {
                    if let Some(arr) = v.get("flags").and_then(|x| x.as_array()) {
                        set_flags.set(arr.clone());
                    }
                }
                if let Ok(v) = api::get::<serde_json::Value>("/admin/moderation/appeals?status=open").await {
                    if let Some(arr) = v.get("appeals").and_then(|x| x.as_array()) {
                        set_appeals.set(arr.clone());
                    }
                }
                if let Ok(v) = api::get::<serde_json::Value>("/admin/shadow-bans").await {
                    if let Some(arr) = v.get("shadow_bans").and_then(|x| x.as_array()) {
                        set_bans.set(arr.clone());
                    }
                }
                set_loading.set(false);
            });
        }
    };
    reload();

    let post = move |path: String, body: serde_json::Value| {
        let set_loading = set_loading.clone();
        let reload = reload.clone();
        spawn_local(async move {
            match api::post::<serde_json::Value>(&path, &body).await {
                Ok(v) => {
                    let msg = s(&v, "message");
                    toast.success(if msg.is_empty() { "OK" } else { &msg });
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

    let post_checked = move |path: String, body: serde_json::Value| {
        let reload = reload.clone();
        let set_loading = set_loading.clone();
        spawn_local(async move {
            match api::post::<serde_json::Value>(&path, &body).await {
                Ok(_) => {
                    toast.success("OK");
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
            let q = queue.get();
            let rp = reports.get();
            let fl = flags.get();
            let ap = appeals.get();
            let bn = bans.get();

            let summary: Vec<AnyView> = vec![
                tile("Cola pendiente", fmt_int(q.len() as i64)),
                tile("Reportes", fmt_int(rp.len() as i64)),
                tile("Flags de contenido", fmt_int(fl.len() as i64)),
                tile("Apelaciones abiertas", fmt_int(ap.len() as i64)),
                tile("Shadow bans", fmt_int(bn.len() as i64)),
            ];

            view! {
                <div style="margin-top:16px;">
                    <div class="stat-grid">{summary.into_iter().collect_view()}</div>

                    <button class="btn btn-primary btn-sm" style="margin:14px 0;" on:click=move |_| reload()>
                        "\u{1F504} Recargar"
                    </button>

                    <div class="panel">
                        <div class="panel-title">"Cola de moderación"</div>
                        {if q.is_empty() {
                            view! { <div class="item-meta">"Sin ítems pendientes."</div> }.into_any()
                        } else {
                            view! {
                                <table class="data-table">
                                    <thead>
                                        <tr><th>"ID"</th><th>"Tipo"</th><th>"Ref"</th><th>"Severidad"</th><th>"Notas"</th><th>"Creación"</th><th></th></tr>
                                    </thead>
                                    <tbody>
                                        {q.iter().map(|i| {
                                            let i = i.clone();
                                            let id = num(&i, "id");
                                            let post = post.clone();
                                            view! {
                                                <tr>
                                                    <td>#{fmt_int(id)}</td>
                                                    <td><span class="badge">{s(&i, "item_type")}</span></td>
                                                    <td>{fmt_int(num(&i, "reference_id"))}</td>
                                                    <td>{format!("{:.0}%", f(&i, "severity") * 100.0)}</td>
                                                    <td style="color:var(--text-dim);">{truncate(&s(&i, "notes"), 40)}</td>
                                                    <td style="color:var(--text-dim);font-size:0.8rem;">{s(&i, "created_at")}</td>
                                                    <td>
                                                        <button
                                                            class="btn btn-sm"
                                                            on:click=move |_| post(format!("/admin/moderation/queue/{id}"), serde_json::json!({"status": "reviewed"}))
                                                        >
                                                            "\u{2713} Revisar"
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_any()
                        }}
                    </div>

                    <div class="panel">
                        <div class="panel-title">"Reportes"</div>
                        {if rp.is_empty() {
                            view! { <div class="item-meta">"Sin reportes pendientes."</div> }.into_any()
                        } else {
                            view! {
                                <table class="data-table">
                                    <thead>
                                        <tr><th>"ID"</th><th>"Denunciante"</th><th>"Objetivo"</th><th>"Categoría"</th><th>"Descripción"</th><th>"Estado"</th><th>"Acción"</th></tr>
                                    </thead>
                                    <tbody>
                                        {rp.iter().map(|r| {
                                            let r = r.clone();
                                            let id = num(&r, "id");
                                            let post = post.clone();
                                            let post_checked = post_checked.clone();
                                            view! {
                                                <tr>
                                                    <td>#{fmt_int(id)}</td>
                                                    <td>#{fmt_int(num(&r, "reporter_id"))}</td>
                                                    <td>{s(&r, "target_type")} #{fmt_int(num(&r, "target_id"))}</td>
                                                    <td>{s(&r, "category")}</td>
                                                    <td style="color:var(--text-dim);">{truncate(&s(&r, "description"), 36)}</td>
                                                    <td><span class="badge">{s(&r, "status")}</span></td>
                                                    <td>
                                                        <div style="display:flex;gap:6px;">
                                                            <button
                                                                class="btn btn-sm"
                                                                on:click=move |_| post(format!("/admin/moderation/report/{id}"), serde_json::json!({"status": "reviewed"}))
                                                            >
                                                                "\u{2713} Revisar"
                                                            </button>
                                                            <button
                                                                class="btn btn-sm btn-outline"
                                                                on:click={let post_checked = post_checked.clone(); move |_| post_checked(format!("/admin/moderation/report/{id}"), serde_json::json!({"status": "actioned"}))}
                                                            >
                                                                "Accionar"
                                                            </button>
                                                        </div>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_any()
                        }}
                    </div>

                    <div class="panel">
                        <div class="panel-title">"Flags de contenido"</div>
                        {if fl.is_empty() {
                            view! { <div class="item-meta">"Sin flags pendientes."</div> }.into_any()
                        } else {
                            view! {
                                <table class="data-table">
                                    <thead>
                                        <tr><th>"ID"</th><th>"Tipo"</th><th>"Objetivo"</th><th>"Flag"</th><th>"Fuente"</th><th>"Severidad"</th><th>"Acción"</th></tr>
                                    </thead>
                                    <tbody>
                                        {fl.iter().map(|x| {
                                            let x = x.clone();
                                            let id = num(&x, "id");
                                            let post = post.clone();
                                            view! {
                                                <tr>
                                                    <td>#{fmt_int(id)}</td>
                                                    <td>{s(&x, "target_type")} #{fmt_int(num(&x, "target_id"))}</td>
                                                    <td><span class="badge">{s(&x, "flag_type")}</span></td>
                                                    <td>{s(&x, "source")}</td>
                                                    <td>{format!("{:.0}%", f(&x, "severity") * 100.0)}</td>
                                                    <td>{truncate(&s(&x, "description"), 30)}</td>
                                                    <td>
                                                        <button
                                                            class="btn btn-sm"
                                                            on:click=move |_| post(format!("/admin/moderation/flag/{id}"), serde_json::json!({"status": "reviewed"}))
                                                        >
                                                            "\u{2713} Revisar"
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_any()
                        }}
                    </div>

                    <div class="panel">
                        <div class="panel-title">"Apelaciones"</div>
                        {if ap.is_empty() {
                            view! { <div class="item-meta">"Sin apelaciones abiertas."</div> }.into_any()
                        } else {
                            view! {
                                <table class="data-table">
                                    <thead>
                                        <tr><th>"ID"</th><th>"Usuario"</th><th>"Objetivo"</th><th>"Razón"</th><th>"Creada"</th><th>"Acción"</th></tr>
                                    </thead>
                                    <tbody>
                                        {ap.iter().map(|a| {
                                            let a = a.clone();
                                            let id = num(&a, "id");
                                            let post = post.clone();
                                            view! {
                                                <tr>
                                                    <td>#{fmt_int(id)}</td>
                                                    <td>#{fmt_int(num(&a, "user_id"))}</td>
                                                    <td>{s(&a, "target_type")} #{fmt_int(num(&a, "target_id"))}</td>
                                                    <td style="color:var(--text-dim);">{truncate(&s(&a, "reason"), 40)}</td>
                                                    <td style="color:var(--text-dim);font-size:0.8rem;">{s(&a, "created_at")}</td>
                                                    <td>
                                                        <div style="display:flex;gap:6px;">
                                                            <button
                                                                class="btn btn-sm"
                                                                on:click=move |_| post(format!("/admin/moderation/appeal/{id}"), serde_json::json!({"approved": true, "notes": "approved by admin"}))
                                                            >
                                                                "\u{2713} Aprobar"
                                                            </button>
                                                            <button
                                                                class="btn btn-sm btn-outline"
                                                                on:click=move |_| post(format!("/admin/moderation/appeal/{id}"), serde_json::json!({"approved": false, "notes": "rejected by admin"}))
                                                            >
                                                                "\u{2717} Rechazar"
                                                            </button>
                                                        </div>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_any()
                        }}
                    </div>

                    <div class="panel">
                        <div class="panel-title">"Shadow bans activos"</div>
                        {if bn.is_empty() {
                            view! { <div class="item-meta">"Sin shadow bans."</div> }.into_any()
                        } else {
                            view! {
                                <table class="data-table">
                                    <thead>
                                        <tr><th>"User ID"</th><th>"Razón"</th><th>"Hasta"</th><th></th></tr>
                                    </thead>
                                    <tbody>
                                        {bn.iter().map(|b| {
                                            let b = b.clone();
                                            let uid = num(&b, "user_id");
                                            let post = post.clone();
                                            view! {
                                                <tr>
                                                    <td>#{fmt_int(uid)}</td>
                                                    <td>{s(&b, "reason")}</td>
                                                    <td style="color:var(--text-dim);">{s(&b, "banned_until")}</td>
                                                    <td>
                                                        <button
                                                            class="btn btn-sm"
                                                            on:click=move |_| post(format!("/admin/user/{uid}/unshadow-ban"), serde_json::json!({}))
                                                        >
                                                            "Quitar"
                                                        </button>
                                                    </td>
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

fn tile(label: &'static str, value: String) -> AnyView {
    view! {
        <div class="stat-tile">
            <div class="stat-value">{value}</div>
            <div class="stat-label">{label}</div>
        </div>
    }
    .into_any()
}