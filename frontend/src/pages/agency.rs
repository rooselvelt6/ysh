use leptos::prelude::*;
use crate::api;

#[component]
pub fn AgencyPage() -> impl IntoView {
    let (agencies, set_agencies) = signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = signal(true);

    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(val) = api::get::<serde_json::Value>("/agencies").await {
            if let Some(arr) = val.get("agencies").and_then(|v| v.as_array()) {
                set_agencies.set(arr.clone());
            }
        }
        set_loading.set(false);
    });

    view! {
        <div class="main-header">
            <div style="display:flex;justify-content:space-between;align-items:center;">
                <h2>"Agencies"</h2>
                <button class="btn btn-primary btn-sm" style="width:auto;padding:8px 16px;">"+ New"</button>
            </div>
        </div>
        {move || {
            if loading.get() {
                view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else {
                let list = agencies.get();
                if list.is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-state-icon">"\u{1F3E2}"</div>
                            <div class="empty-state-title">"No agencies yet"</div>
                            <div class="empty-state-text">"Be the first to create one!"</div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        {list.into_iter().map(|a| {
                            let name = a.get("name").and_then(|v| v.as_str()).unwrap_or("Agency").to_string();
                            let desc = a.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let members = a.get("member_count").and_then(|v| v.as_i64()).unwrap_or(0);
                            view! {
                                <div class="item-card">
                                    <div class="avatar avatar-lg" style="background:#7856ff;">
                                        {name.chars().next().unwrap_or('A').to_uppercase().to_string()}
                                    </div>
                                    <div class="item-info" style="flex:1;">
                                        <div class="item-name">{name}</div>
                                        <div class="item-meta">{format!("{members} members")}</div>
                                        {if !desc.is_empty() { view! { <div class="item-meta" style="margin-top:4px;">{desc}</div> }.into_any() } else { view! {}.into_any() }}
                                    </div>
                                    <button class="btn btn-outline btn-sm">"Join"</button>
                                </div>
                            }
                        }).collect_view()}
                    }.into_any()
                }
            }
        }}
    }
}
