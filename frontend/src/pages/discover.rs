use leptos::prelude::*;
use crate::api;

#[component]
pub fn DiscoverPage() -> impl IntoView {
    let (hosts, set_hosts) = signal(Vec::<serde_json::Value>::new());

    let (loading, set_loading) = signal(true);

    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(val) = api::get::<serde_json::Value>("/hosts").await {
            if let Some(arr) = val.get("hosts").and_then(|v| v.as_array()) {
                set_hosts.set(arr.clone());
            }
        }
        set_loading.set(false);
    });

    view! {
        <div class="main-header">
            <h2>"Discover"</h2>
        </div>
        {move || {
            if loading.get() {
                view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else {
                let h = hosts.get();
                if h.is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-state-icon">"\u{1F50D}"</div>
                            <div class="empty-state-title">"No hosts available"</div>
                            <div class="empty-state-text">"Check back later"</div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        {h.into_iter().enumerate().map(|(i, host)| {
                            let uid = host.get("user_id").and_then(|v| v.as_i64()).unwrap_or(i as i64);
                            let lang = host.get("languages").and_then(|v| v.as_str()).unwrap_or("N/A").to_string();
                            let rate = host.get("hourly_rate").and_then(|v| v.as_i64()).unwrap_or(0);
                            let rating = host.get("rating").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let available = host.get("available").and_then(|v| v.as_bool()).unwrap_or(false);
                            let colors = ["#7856ff", "#1d9bf0", "#f91880", "#00ba7c"];
                            let color = colors[i % colors.len()];
                            view! {
                                <div class="item-card">
                                    <div class="avatar avatar-lg" style={format!("background:{}", color)}>
                                        {format!("H{}", uid)}
                                    </div>
                                    <div class="item-info" style="flex:1;">
                                        <div class="item-name">{format!("Host #{uid}")}</div>
                                        <div class="item-meta">
                                            {format!("{lang} \u{00B7} \u{2B50} {rating:.1} \u{00B7} {rate} YSH/hr")}
                                        </div>
                                        <div style="margin-top:4px;">
                                            {if available {
                                                view! { <span class="badge badge-success">"Online"</span> }.into_any()
                                            } else {
                                                view! { <span class="badge" style="color:var(--text-dim);">"Offline"</span> }.into_any()
                                            }}
                                        </div>
                                    </div>
                                    <button class="btn btn-outline btn-sm" on:click=move |_| {
                                        api::go(&format!("/stream?host={uid}"));
                                    }>
                                        "Watch"
                                    </button>
                                </div>
                            }
                        }).collect_view()}
                    }.into_any()
                }
            }
        }}
    }
}
