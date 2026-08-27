use leptos::prelude::*;
use crate::api;

#[component]
pub fn ChatPage() -> impl IntoView {
    let (sessions, set_sessions) = signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = signal(true);

    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(val) = api::get::<serde_json::Value>("/chat/sessions").await {
            if let Some(arr) = val.get("sessions").and_then(|v| v.as_array()) {
                set_sessions.set(arr.clone());
            }
        }
        set_loading.set(false);
    });

    view! {
        <div class="main-header">
            <h2>"Messages"</h2>
        </div>
        {move || {
            if loading.get() {
                view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else {
                let list = sessions.get();
                if list.is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-state-icon">"\u{1F4AC}"</div>
                            <div class="empty-state-title">"No conversations"</div>
                            <div class="empty-state-text">"Start a chat from a host's profile"</div>
                        </div>
                    }.into_any()
                } else {
                    let colors = ["#7856ff", "#1d9bf0", "#f91880", "#00ba7c"];
                    view! {
                        {list.into_iter().enumerate().map(|(i, s)| {
                            let participants = s.get("participants").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                            let name = participants.first()
                                .and_then(|p| p.get("username").and_then(|v| v.as_str()))
                                .unwrap_or("Unknown")
                                .to_string();
                            let updated = s.get("updated_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let short_time = if updated.len() >= 10 { updated[5..10].to_string() } else { updated.clone() };
                            let color = colors[i % colors.len()];
                            view! {
                                <div class="list-item">
                                    <div class="avatar avatar-md" style={format!("background:{}", color)}>
                                        {name.chars().next().unwrap_or('U').to_uppercase().to_string()}
                                    </div>
                                    <div class="list-item-body">
                                        <div class="list-item-title">{name}</div>
                                        <div class="list-item-text">{format!("Session {}", short_time)}</div>
                                    </div>
                                    <div class="list-item-time">{short_time}</div>
                                </div>
                            }
                        }).collect_view()}
                    }.into_any()
                }
            }
        }}
    }
}
