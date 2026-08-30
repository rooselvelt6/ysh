use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;

use super::{fmt_int, num, s, truncate};

#[component]
pub fn AgenciesTab() -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (agencies, set_agencies) = signal(Vec::<serde_json::Value>::new());

    let reload = {
        let set_loading = set_loading.clone();
        move || {
            set_loading.set(true);
            spawn_local(async move {
                if let Ok(v) = api::get::<serde_json::Value>("/agencies").await {
                    if let Some(arr) = v.get("agencies").and_then(|x| x.as_array()) {
                        set_agencies.set(arr.clone());
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
            let list = agencies.get();
            if list.is_empty() {
                return view! {
                    <div class="empty-state" style="margin-top:24px;">
                        <div class="empty-state-icon">"\u{1F3E2}"</div>
                        <div class="empty-state-title">"Sin agencias"</div>
                        <div class="empty-state-text">"Aún no hay agencias registradas."</div>
                    </div>
                }.into_any();
            }
            view! {
                <div style="margin-top:16px;">
                    <div class="stat-grid">
                        {vec![
                            ("Agencias", fmt_int(list.len() as i64)),
                            ("Miembros totales", fmt_int(list.iter().map(|a| num(a, "member_count")).sum())),
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

                    {list.iter().map(|a| {
                        let a = a.clone();
                        let name = s(&a, "name");
                        let members = a.get("members").and_then(|x| x.as_array()).cloned().unwrap_or_default();
                        view! {
                            <div class="panel">
                                <div class="panel-title" style="display:flex;justify-content:space-between;align-items:center;">
                                    <span>{name}</span>
                                    <span style="color:var(--text-dim);font-weight:400;font-size:0.8rem;">
                                        "ID " {fmt_int(num(&a, "id"))} " · Dueño #" {fmt_int(num(&a, "owner_id"))}
                                    </span>
                                </div>
                                <div class="item-meta" style="margin-bottom:10px;">
                                    {truncate(&s(&a, "description"), 120)}
                                </div>
                                <div style="color:var(--text-dim);font-size:0.85rem;margin-bottom:8px;">
                                    <strong style="color:var(--text);">{fmt_int(num(&a, "member_count"))}</strong> " miembros — creada " {s(&a, "created_at")}
                                </div>
                                {if members.is_empty() {
                                    view! { <div class="item-meta">"Sin miembros aún."</div> }.into_any()
                                } else {
                                    view! {
                                        <table class="data-table">
                                            <thead>
                                                <tr><th>"User ID"</th><th>"Rol"</th><th>"Ingreso"</th></tr>
                                            </thead>
                                            <tbody>
                                                {members.iter().map(|mm| {
                                                    let agency_id = num(&a, "id");
                                                    let user_id = num(mm, "user_id");
                                                    let remove_click = move |_: leptos::ev::MouseEvent| {
                                                        let ag = agency_id;
                                                        let uid = user_id;
                                                        spawn_local(async move {
                                                            let _ = api::del::<serde_json::Value>(&format!(
                                                                "/admin/agency/{ag}/members/{uid}"
                                                            )).await;
                                                            reload();
                                                        });
                                                    };
                                                    view! {
                                                        <tr>
                                                            <td>#{fmt_int(num(mm, "user_id"))}</td>
                                                            <td>
                                                                <span class="badge">{s(mm, "role")}</span>
                                                                <button
                                                                    class="btn btn-danger btn-xs"
                                                                    style="margin-left:6px;"
                                                                    title="Eliminar miembro"
                                                                    on:click=remove_click
                                                                >"x"</button>
                                                            </td>
                                                            <td style="color:var(--text-dim);font-size:0.8rem;">{s(mm, "joined_at")}</td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    }.into_any()
                                }}
                            </div>
                        }
                    }).collect_view()}
                </div>
            }.into_any()
        }}
    }
}