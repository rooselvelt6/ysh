use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::ui::toast::ToastCtx;

use super::{fmt_int, num, s, truncate};

#[component]
pub fn MomentsTab() -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (moments, set_moments) = signal(Vec::<serde_json::Value>::new());
    let toast = ToastCtx::use_();

    let reload = {
        let set_loading = set_loading.clone();
        move || {
            set_loading.set(true);
            spawn_local(async move {
                if let Ok(v) = api::get::<serde_json::Value>("/admin/moments?limit=150").await {
                    if let Some(arr) = v.get("moments").and_then(|x| x.as_array()) {
                        set_moments.set(arr.clone());
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
            let list = moments.get();
            view! {
                <div style="margin-top:16px;">
                    <div class="stat-grid">
                        {vec![
                            ("Momentos cargados", fmt_int(list.len() as i64)),
                            ("Con imagen", fmt_int(list.iter().filter(|m| s(m, "media_type") == "image").count() as i64)),
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

                    <div class="panel">
                        <div class="panel-title">"Todos los momentos"</div>
                        <table class="data-table">
                            <thead>
                                <tr><th>"ID"</th><th>"Usuario"</th><th>"Contenido"</th><th>"Tipo"</th><th>"Likes"</th><th>"Comentarios"</th><th>"Fecha"</th><th>"Acción"</th></tr>
                            </thead>
                            <tbody>
                                {list.iter().map(|m| {
                                    let m = m.clone();
                                    let mid = num(&m, "id");
                                    let reload = reload.clone();
                                    view! {
                                        <tr>
                                            <td>#{fmt_int(mid)}</td>
                                            <td><strong>{s(&m, "username")}</strong></td>
                                            <td style="max-width:280px;">{truncate(&s(&m, "content"), 60)}</td>
                                            <td><span class="badge">{s(&m, "media_type")}</span></td>
                                            <td>{fmt_int(num(&m, "likes"))}</td>
                                            <td>{fmt_int(num(&m, "comments"))}</td>
                                            <td style="color:var(--text-dim);font-size:0.8rem;">{s(&m, "created_at")}</td>
                                            <td>
                                                <button
                                                    class="btn btn-sm"
                                                    style="background:var(--danger,#dc3545);color:#fff;"
                                                    on:click={let reload = reload.clone(); move |_| {
                                                        let reload = reload.clone();
                                                        let reload2 = reload.clone();
                                                        spawn_local(async move {
                                                            match api::post::<serde_json::Value>(&format!("/admin/moment/{mid}/delete"), &serde_json::json!({})).await {
                                                                Ok(v) => {
                                                                    let msg = s(&v, "message");
                                                                    toast.success(if msg.is_empty() { "Eliminado".to_string() } else { msg });
                                                                    reload();
                                                                }
                                                                Err(e) => { toast.error(format!("Failed: {e}")); reload2(); }
                                                            }
                                                        });
                                                    }}
                                                >
                                                    "\u{1F5D1} Eliminar"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                </div>
            }.into_any()
        }}
    }
}