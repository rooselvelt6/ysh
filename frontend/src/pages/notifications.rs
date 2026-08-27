use leptos::prelude::*;
use crate::api;

#[component]
pub fn NotificationsPage() -> impl IntoView {
    let (notifs, set_notifs) = signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = signal(true);

    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(val) = api::get::<serde_json::Value>("/notifications").await {
            if let Some(arr) = val.get("notifications").and_then(|v| v.as_array()) {
                set_notifs.set(arr.clone());
            }
        }
        set_loading.set(false);
    });

    let mark_all_read = move |_: leptos::ev::MouseEvent| {
        wasm_bindgen_futures::spawn_local(async move {
            let _ = api::post::<serde_json::Value>("/notifications/read-all", &serde_json::json!({})).await;
            if let Ok(val) = api::get::<serde_json::Value>("/notifications").await {
                if let Some(arr) = val.get("notifications").and_then(|v| v.as_array()) {
                    set_notifs.set(arr.clone());
                }
            }
        });
    };

    view! {
        <div class="main-header">
            <div style="display:flex;justify-content:space-between;align-items:center;">
                <h2>"Notifications"</h2>
                <button class="btn-ghost btn-sm" on:click=mark_all_read>"Mark all read"</button>
            </div>
        </div>
        {move || {
            if loading.get() {
                view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else {
                let list = notifs.get();
                if list.is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-state-icon">"\u{1F514}"</div>
                            <div class="empty-state-title">"No notifications"</div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        {list.into_iter().map(|n| {
                            let is_read = n.get("read").and_then(|v| v.as_bool()).unwrap_or(false);
                            let title = n.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let body = n.get("body").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let ntype = n.get("ntype").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let icon = match ntype.as_str() {
                                "gift" => "\u{1F381}",
                                "message" => "\u{1F4AC}",
                                "like" => "\u{2764}",
                                _ => "\u{1F514}",
                            };
                            view! {
                                <div class={format!("list-item{}", if is_read { "" } else { " unread" })}>
                                    <span style="font-size:1.5rem;">{icon}</span>
                                    <div class="list-item-body">
                                        <div class="list-item-title">{title}</div>
                                        <div class="list-item-text">{body}</div>
                                    </div>
                                    {if !is_read { view! { <div class="unread-dot"></div> }.into_any() } else { view! {}.into_any() }}
                                </div>
                            }
                        }).collect_view()}
                    }.into_any()
                }
            }
        }}
    }
}
