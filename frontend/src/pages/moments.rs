use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::store;
use crate::components::ui::toast::ToastCtx;

#[component]
pub fn MomentsPage() -> impl IntoView {
    let (moments, set_moments) = signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = signal(true);
    let (new_content, set_new_content) = signal(String::new());
    let (posting, set_posting) = signal(false);
    let (comment_target, set_comment_target) = signal(Option::<i64>::None);
    let (comment_text, set_comment_text) = signal(String::new());
    let (commenting, set_commenting) = signal(false);
    let toast = ToastCtx::use_();

    let load = {
        let set_moments = set_moments.clone();
        let set_loading = set_loading.clone();
        move || {
            set_loading.set(true);
            spawn_local(async move {
                if let Ok(val) = api::get::<serde_json::Value>("/moments").await {
                    if let Some(arr) = val.get("moments").and_then(|v| v.as_array()) {
                        set_moments.set(arr.clone());
                    }
                }
                set_loading.set(false);
            });
        }
    };
    load();

    let reload = {
        let set_moments = set_moments.clone();
        move || {
            spawn_local(async move {
                if let Ok(val) = api::get::<serde_json::Value>("/moments").await {
                    if let Some(arr) = val.get("moments").and_then(|v| v.as_array()) {
                        set_moments.set(arr.clone());
                    }
                }
            });
        }
    };

    let post_moment = move |_: leptos::ev::MouseEvent| {
        let content = new_content.get();
        if content.trim().is_empty() { return; }
        set_posting.set(true);
        let reload = reload.clone();
        spawn_local(async move {
            let req = serde_json::json!({"content": content, "media_type": "text"});
            match api::post::<serde_json::Value>("/moment", &req).await {
                Ok(_) => {
                    toast.success("Moment posted!");
                    set_new_content.set(String::new());
                    reload();
                }
                Err(e) => toast.error(format!("Failed: {e}")),
            }
            set_posting.set(false);
        });
    };

    let toggle_like = move |id: i64| {
        let reload = reload.clone();
        spawn_local(async move {
            match api::post::<serde_json::Value>(&format!("/moment/{id}/like"), &serde_json::json!({})).await {
                Ok(_) => reload(),
                Err(e) => toast.error(format!("Failed: {e}")),
            }
        });
    };

    let submit_comment = move |_: leptos::ev::MouseEvent| {
        let id = match comment_target.get() {
            Some(i) => i,
            None => return,
        };
        let text = comment_text.get().trim().to_string();
        if text.is_empty() {
            toast.error("Comment cannot be empty");
            return;
        }
        set_commenting.set(true);
        let reload = reload.clone();
        spawn_local(async move {
            let req = serde_json::json!({"content": text});
            match api::post::<serde_json::Value>(&format!("/moment/{id}/comment"), &req).await {
                Ok(_) => {
                    toast.success("Comment added!");
                    set_comment_target.set(None);
                    set_comment_text.set(String::new());
                    reload();
                }
                Err(e) => toast.error(format!("Failed: {e}")),
            }
            set_commenting.set(false);
        });
    };

    let post_disabled = move || posting.get() || new_content.get().trim().is_empty();

    view! {
        <div class="main-header">
            <h2>"Moments"</h2>
        </div>
        <div class="compose-box">
            <div class="avatar avatar-md" style="background:#7856ff;">
                {move || store::get_user().map(|u| u.username.chars().next().unwrap_or('U').to_uppercase().to_string()).unwrap_or_else(|| "U".into())}
            </div>
            <div class="compose-body">
                <textarea
                    class="compose-input"
                    placeholder="Share a moment..."
                    rows="2"
                    prop:value=move || new_content.get()
                    on:input=move |ev| set_new_content.set(event_target_value(&ev))
                ></textarea>
                <div class="compose-footer">
                    <div class="compose-actions">
                        <button class="compose-action-btn">"\u{1F4F7}"</button>
                        <button class="compose-action-btn">"\u{1F389}"</button>
                    </div>
                    <button
                        class="compose-submit"
                        on:click=post_moment
                        prop:disabled=post_disabled
                    >
                        {move || if posting.get() { "Posting..." } else { "Post" }}
                    </button>
                </div>
            </div>
        </div>

        // Comment modal
        {move || {
            if let Some(id) = comment_target.get() {
                view! {
                    <div class="modal-overlay" on:click=move |_| set_comment_target.set(None)>
                        <div class="modal" on:click=move |ev| ev.stop_propagation()>
                            <div class="modal-header">
                                <h2 class="modal-title">{format!("Comment on post #{id}")}</h2>
                                <button class="modal-close" on:click=move |_| set_comment_target.set(None)>"\u{00d7}"</button>
                            </div>
                            <div style="padding:0 16px 16px;">
                                <div class="form-group" style="margin-bottom:16px;">
                                    <label class="form-label">"Your comment"</label>
                                    <textarea class="form-textarea" placeholder="Write a comment..."
                                        prop:value=move || comment_text.get()
                                        on:input=move |ev| set_comment_text.set(event_target_value(&ev))></textarea>
                                </div>
                                <button class="btn btn-primary" on:click=submit_comment
                                    prop:disabled=move || commenting.get()>
                                    {move || if commenting.get() { "Posting..." } else { "Comment" }}
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { view! {}.into_any() }
        }}

        {move || {
            if loading.get() {
                view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else {
                let list = moments.get();
                if list.is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-state-icon">"\u{1F4F7}"</div>
                            <div class="empty-state-title">"No moments yet"</div>
                            <div class="empty-state-text">"Be the first to share something!"</div>
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
                            let my_id = id;
                            view! {
                                <div class="feed-post">
                                    <div class="post-avatar avatar-sm" style="background:#7856ff;">
                                        {post_user.chars().next().unwrap_or('U').to_uppercase().to_string()}
                                    </div>
                                    <div class="post-body">
                                        <div class="post-header">
                                            <span class="post-username">{post_user.clone()}</span>
                                            <span class="post-handle">{format!("@{}", &post_user)}</span>
                                            <span class="post-time">{short_time}</span>
                                        </div>
                                        <div class="post-content">{content}</div>
                                        <div class="post-actions">
                                            <button class="post-action gm-comment"
                                                on:click=move |_| set_comment_target.set(Some(my_id))>
                                                <span class="action-icon">"\u{1F4AC}"</span>
                                                <span>{comments.to_string()}</span>
                                            </button>
                                            <button class={format!("post-action like{}", if liked { " liked" } else { "" })}
                                                on:click=move |_| toggle_like(my_id)>
                                                <span class="action-icon">{if liked { "\u{2764}\u{FE0F}" } else { "\u{2661}" }}</span>
                                                <span>{likes.to_string()}</span>
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
