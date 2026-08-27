use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::store;


#[component]
pub fn DashboardPage() -> impl IntoView {
    let (moments, set_moments) = signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = signal(true);
    let (new_content, set_new_content) = signal(String::new());
    let (posting, set_posting) = signal(false);
    let (tab, set_tab) = signal(0u8); // 0=for you, 1=following

    let user = store::get_user();
    let _username = user.as_ref().map(|u| u.username.clone()).unwrap_or_else(|| "user".into());

    spawn_local(async move {
        if let Ok(val) = api::get::<serde_json::Value>("/moments").await {
            if let Some(arr) = val.get("moments").and_then(|v| v.as_array()) {
                set_moments.set(arr.clone());
            }
        }
        set_loading.set(false);
    });

    let post_moment = move |_: leptos::ev::MouseEvent| {
        let content = new_content.get();
        if content.trim().is_empty() { return; }
        set_posting.set(true);
        spawn_local(async move {
            let req = serde_json::json!({"content": content, "media_type": "text"});
            let _ = api::post::<serde_json::Value>("/moment", &req).await;
            set_new_content.set(String::new());
            if let Ok(val) = api::get::<serde_json::Value>("/moments").await {
                if let Some(arr) = val.get("moments").and_then(|v| v.as_array()) {
                    set_moments.set(arr.clone());
                }
            }
            set_posting.set(false);
        });
    };

    let toggle_like = move |id: i64| {
        spawn_local(async move {
            let _ = api::post::<serde_json::Value>(&format!("/moment/{id}/like"), &serde_json::json!({})).await;
            if let Ok(val) = api::get::<serde_json::Value>("/moments").await {
                if let Some(arr) = val.get("moments").and_then(|v| v.as_array()) {
                    set_moments.set(arr.clone());
                }
            }
        });
    };

    let post_disabled = move || posting.get() || new_content.get().trim().is_empty();

    let initial = {
        let u = store::get_user();
        u.map(|u| u.username.chars().next().unwrap_or('U').to_uppercase().to_string())
            .unwrap_or_else(|| "U".into())
    };

    view! {
        <div class="main-header">
            <h2>"Home"</h2>
            <div class="main-tabs">
                <button
                    class={move || format!("main-tab{}", if tab.get() == 0 { " active" } else { "" })}
                    on:click=move |_| set_tab.set(0)
                >
                    "For you"
                </button>
                <button
                    class={move || format!("main-tab{}", if tab.get() == 1 { " active" } else { "" })}
                    on:click=move |_| set_tab.set(1)
                >
                    "Following"
                </button>
            </div>
        </div>

        <div class="compose-box">
            <div class="post-avatar" style="background:#7856ff;">{initial}</div>
            <div class="compose-body">
                <textarea
                    class="compose-input"
                    placeholder="What's happening?"
                    rows="2"
                    prop:value=move || new_content.get()
                    on:input=move |ev| set_new_content.set(event_target_value(&ev))
                ></textarea>
                <div class="compose-footer">
                    <div class="compose-actions">
                        <button class="compose-action-btn">"\u{1F4F7}"</button>
                        <button class="compose-action-btn">"\u{1F3AC}"</button>
                        <button class="compose-action-btn">"\u{1F389}"</button>
                    </div>
                    <button
                        class="compose-submit"
                        on:click=post_moment
                        prop:disabled=post_disabled
                    >
                        "Post"
                    </button>
                </div>
            </div>
        </div>

        {move || {
            if loading.get() {
                view! {
                    <div class="loading-center">
                        <div class="spinner spinner-lg"></div>
                    </div>
                }.into_any()
            } else {
                let list = moments.get();
                if list.is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-state-icon">"\u{1F4F7}"</div>
                            <div class="empty-state-title">"No posts yet"</div>
                            <div class="empty-state-text">"When someone posts, it will show up here."</div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        {list.into_iter().map(|m| {
                            let id = m.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                            let content = m.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let post_user = m.get("username").and_then(|v| v.as_str()).unwrap_or("user").to_string();
                            let likes = m.get("likes").and_then(|v| v.as_i64()).unwrap_or(0);
                            let comments = m.get("comments").and_then(|v| v.as_i64()).unwrap_or(0);
                            let liked = m.get("liked").and_then(|v| v.as_bool()).unwrap_or(false);
                            let created = m.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let short_time = if created.len() >= 10 { created[5..10].to_string() } else { created.clone() };
                            let toggle = toggle_like.clone();
                            let user_initial = post_user.chars().next().unwrap_or('U').to_uppercase().to_string();
                            let colors = ["#7856ff", "#1d9bf0", "#f91880", "#00ba7c", "#ffd700"];
                            let color_idx = (id as usize) % colors.len();
                            view! {
                                <div class="feed-post">
                                    <div class="post-avatar" style={format!("background:{}", colors[color_idx])}>
                                        {user_initial}
                                    </div>
                                    <div class="post-body">
                                        <div class="post-header">
                                            <span class="post-username">{post_user.clone()}</span>
                                            <span class="post-handle">{format!("@{}", &post_user)}</span>
                                            <span class="post-time">{short_time}</span>
                                        </div>
                                        <div class="post-content">{content}</div>
                                        <div class="post-actions">
                                            <button class="post-action" on:click=move |_| {
                                                // reply placeholder
                                            }>
                                                <span class="action-icon">"\u{1F4AC}"</span>
                                                <span>{comments.to_string()}</span>
                                            </button>
                                            <button class={format!("post-action repost{}", "")}>
                                                <span class="action-icon">"\u{1F504}"</span>
                                            </button>
                                            <button
                                                class={format!("post-action like{}", if liked { " liked" } else { "" })}
                                                on:click=move |_| toggle(id)
                                            >
                                                <span class="action-icon">{if liked { "\u{2764}\u{FE0F}" } else { "\u{2661}" }}</span>
                                                <span>{likes.to_string()}</span>
                                            </button>
                                            <button class="post-action">
                                                <span class="action-icon">"\u{1F4E4}"</span>
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    }.into_any()
                }
            }
        }}
    }
}
