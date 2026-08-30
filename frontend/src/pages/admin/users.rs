use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::ui::toast::ToastCtx;

use super::{fmt_int, num, s, truncate};

#[component]
pub fn UsersTab() -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (users, set_users) = signal(Vec::<serde_json::Value>::new());
    let (bans, set_bans) = signal(Vec::<serde_json::Value>::new());
    let toast = ToastCtx::use_();

    let reload = {
        let set_users = set_users.clone();
        let set_bans = set_bans.clone();
        let set_loading = set_loading.clone();
        move || {
            set_loading.set(true);
            spawn_local(async move {
                if let Ok(v) = api::get::<serde_json::Value>("/admin/users?limit=300").await {
                    if let Some(arr) = v.get("users").and_then(|x| x.as_array()) {
                        set_users.set(arr.clone());
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

    let act = move |path: String, body: Option<serde_json::Value>| {
        let set_loading = set_loading.clone();
        let reload = reload.clone();
        spawn_local(async move {
            let result = match body {
                Some(b) => api::post::<serde_json::Value>(&path, &b).await,
                None => api::post::<serde_json::Value>(&path, &serde_json::json!({})).await,
            };
            match result {
                Ok(v) => {
                    let msg = s(&v, "message");
                    toast.success(if msg.is_empty() { "OK".into() } else { msg });
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

    let (badge, set_badge) = signal(String::new());

    view! {
        {move || {
            if loading.get() {
                return view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any();
            }
            let list = users.get();
            let total = list.len();
            let admins = list.iter().filter(|u| s(u, "role") == "admin").count();
            let banned = list.iter().filter(|u| u.get("banned").and_then(|b| b.as_bool()).unwrap_or(false)).count();

            view! {
                <div style="margin-top:16px;">
                    <div class="stat-grid">
                        {vec![
                            ("Usuarios cargados", fmt_int(total as i64)),
                            ("Admins", fmt_int(admins as i64)),
                            ("Baneados", fmt_int(banned as i64)),
                            ("Shadow-bans activos", fmt_int(bans.get().len() as i64)),
                        ].into_iter().map(|(label, value)| {
                            view! {
                                <div class="stat-tile">
                                    <div class="stat-value">{value}</div>
                                    <div class="stat-label">{label}</div>
                                </div>
                            }
                        }).collect_view()}
                    </div>

                    <div style="display:flex;gap:8px;align-items:center;justify-content:flex-end;margin:14px 0;">
                        <input
                            type="text"
                            placeholder="Badge: verified | agency | host | staff"
                            value=move || badge.get()
                            on:input=move |ev| set_badge.set(event_target_value(&ev))
                            style="padding:8px 12px;border:1px solid var(--border);border-radius:10px;background:var(--surface);color:var(--text);min-width:220px;"
                        />
                        <button class="btn btn-primary btn-sm" on:click=move |_| reload()>
                            "\u{1F504} Recargar"
                        </button>
                    </div>

                    <div class="panel">
                        <div class="panel-title">"Usuarios"</div>
                        <table class="data-table">
                            <thead>
                                <tr><th>"ID"</th><th>"Usuario"</th><th>"Email"</th><th>"Rol"</th><th>"KYC"</th><th>"Creado"</th><th>"Estado"</th><th>"Acciones"</th></tr>
                            </thead>
                            <tbody>
                                {list.iter().map(|u| {
                                    let u = u.clone();
                                    let uid = num(&u, "id");
                                    let uname = s(&u, "username");
                                    let email = s(&u, "email");
                                    let role = s(&u, "role");
                                    let is_banned = u.get("banned").and_then(|b| b.as_bool()).unwrap_or(false);
                                    let badge_val = badge.get();
                                    let act = act.clone();
                                    view! {
                                        <tr>
                                            <td>#{fmt_int(uid)}</td>
                                            <td><strong>{truncate(&uname, 20)}</strong></td>
                                            <td style="max-width:180px;overflow:hidden;text-overflow:ellipsis;">{truncate(&email, 24)}</td>
                                            <td><span class="badge">{role.clone()}</span></td>
                                            <td>L{fmt_int(num(&u, "kyc_level"))}</td>
                                            <td style="color:var(--text-dim);font-size:0.8rem;">{s(&u, "created_at")}</td>
                                            <td>{
                                                if is_banned { "\u{1F6AB} baneado".to_string() } else { "activo".to_string() }
                                            }</td>
                                            <td>
                                                <div style="display:flex;gap:6px;align-items:center;flex-wrap:wrap;">
                                                    <button
                                                        class="btn btn-sm"
                                                        on:click={let act = act.clone(); move |_| {
                                                            let act = act.clone();
                                                            let path = if is_banned { format!("/admin/user/{uid}/unban") } else { format!("/admin/user/{uid}/ban") };
                                                            act(path, None);
                                                        }}
                                                    >
                                                        {if is_banned { "Desbanear" } else { "Banear" }}
                                                    </button>
                                                    <select
                                                        style="padding:6px 8px;border-radius:8px;border:1px solid var(--border);background:var(--surface);color:var(--text);"
                                                        on:change={let act = act.clone(); move |ev| {
                                                            let role = event_target_value(&ev);
                                                            let act = act.clone();
                                                            act(format!("/admin/user/{uid}/role"), Some(serde_json::json!({"role": role})));
                                                        }}
                                                    >
                                                        <option value="user" selected=role=="user">"user"</option>
                                                        <option value="host" selected=role=="host">"host"</option>
                                                        <option value="moderator" selected=role=="moderator">"moderator"</option>
                                                        <option value="admin" selected=role=="admin">"admin"</option>
                                                    </select>
                                                    <button
                                                        class="btn btn-sm btn-outline"
                                                        title="Shadow ban 24h"
                                                        on:click={let act = act.clone(); move |_| {
                                                            let act = act.clone();
                                                            act(format!("/admin/user/{uid}/shadow-ban"), Some(serde_json::json!({"reason": "admin action", "duration_secs": 86400})));
                                                        }}
                                                    >
                                                        "Sombra"
                                                    </button>
                                                    <button
                                                        class="btn btn-sm btn-outline"
                                                        title="Quitar shadow ban"
                                                        on:click={let act = act.clone(); move |_| {
                                                            let act = act.clone();
                                                            act(format!("/admin/user/{uid}/unshadow-ban"), None);
                                                        }}
                                                    >
                                                        "Sombra/"
                                                    </button>
                                                    {
                                                        if badge_val.is_empty() {
                                                            view! {}.into_any()
                                                        } else {
                                                            let b = badge_val.clone();
                                                            let act = act.clone();
                                                            view! {
                                                                <button
                                                                    class="btn btn-sm"
                                                                    on:click=move |_| {
                                                                        let act = act.clone();
                                                                        act(format!("/admin/user/{uid}/badge"), Some(serde_json::json!({"badge_type": b.clone()})));
                                                                    }
                                                                >
                                                                    "Badge"
                                                                </button>
                                                            }.into_any()
                                                        }
                                                    }
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>

                    <div class="panel">
                        <div class="panel-title">"Shadow bans"</div>
                        {if bans.get().is_empty() {
                            view! { <div class="item-meta">"No hay shadow bans."</div> }.into_any()
                        } else {
                            view! {
                                <table class="data-table">
                                    <thead>
                                        <tr><th>"User ID"</th><th>"Razón"</th><th>"Hasta"</th><th>"Acción"</th></tr>
                                    </thead>
                                    <tbody>
                                        {bans.get().iter().map(|b| {
                                            let b = b.clone();
                                            let uid = num(&b, "user_id");
                                            let act = act.clone();
                                            view! {
                                                <tr>
                                                    <td>#{fmt_int(uid)}</td>
                                                    <td>{s(&b, "reason")}</td>
                                                    <td>{s(&b, "banned_until")}</td>
                                                    <td>
                                                        <button
                                                            class="btn btn-sm"
                                                            on:click=move |_| {
                                                                let act = act.clone();
                                                                act(format!("/admin/user/{uid}/unshadow-ban"), None);
                                                            }
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