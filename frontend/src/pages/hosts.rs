use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::store;
use crate::components::ui::toast::ToastCtx;

#[component]
pub fn HostsPage() -> impl IntoView {
    let (hosts, set_hosts) = signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = signal(true);
    let (show_register, set_show_register) = signal(false);
    let (langs, set_langs) = signal(String::new());
    let (rate, set_rate) = signal(String::new());
    let (busy, set_busy) = signal(false);
    let toast = ToastCtx::use_();

    let load = {
        let set_hosts = set_hosts.clone();
        let set_loading = set_loading.clone();
        move || {
            set_loading.set(true);
            spawn_local(async move {
                if let Ok(val) = api::get::<serde_json::Value>("/hosts").await {
                    if let Some(arr) = val.get("hosts").and_then(|v| v.as_array()) {
                        set_hosts.set(arr.clone());
                    }
                }
                set_loading.set(false);
            });
        }
    };
    load();

    let register_host = move |_: leptos::ev::MouseEvent| {
        let languages = langs.get().trim().to_string();
        if languages.is_empty() {
            toast.error("Enter your languages");
            return;
        }
        let hourly_rate: i64 = match rate.get().trim().parse() {
            Ok(n) => n,
            Err(_) => {
                toast.error("Enter a valid hourly rate");
                return;
            }
        };
        set_busy.set(true);
        spawn_local(async move {
            let req = serde_json::json!({"languages": languages, "hourly_rate": hourly_rate});
            match api::post::<serde_json::Value>("/host", &req).await {
                Ok(_) => {
                    toast.success("Host profile created!");
                    set_show_register.set(false);
                    set_langs.set(String::new());
                    set_rate.set(String::new());
                }
                Err(e) => toast.error(format!("Failed: {e}")),
            }
            set_busy.set(false);
            if let Ok(val) = api::get::<serde_json::Value>("/hosts").await {
                if let Some(arr) = val.get("hosts").and_then(|v| v.as_array()) {
                    set_hosts.set(arr.clone());
                }
            }
        });
    };

    let is_self = move |uid: i64| -> bool {
        store::get_user().map(|u| u.user_id == uid).unwrap_or(false)
    };

    view! {
        <div class="main-header">
            <div style="display:flex;justify-content:space-between;align-items:center;">
                <h2>"Hosts"</h2>
                <button class="btn btn-primary btn-sm" style="width:auto;padding:8px 16px;"
                    on:click=move |_| set_show_register.set(true)>
                    "Become a Host"
                </button>
            </div>
        </div>

        {move || {
            if show_register.get() {
                view! {
                    <div class="modal-overlay" on:click=move |_| set_show_register.set(false)>
                        <div class="modal" on:click=move |ev| ev.stop_propagation()>
                            <div class="modal-header">
                                <h2 class="modal-title">"Register as Host"</h2>
                                <button class="modal-close" on:click=move |_| set_show_register.set(false)>"\u{00d7}"</button>
                            </div>
                            <div style="padding:0 16px 16px;">
                                <div class="form-group" style="margin-bottom:16px;">
                                    <label class="form-label">"Languages"</label>
                                    <input class="form-input" type="text" placeholder="en, es, fr"
                                        prop:value=move || langs.get()
                                        on:input=move |ev| set_langs.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-group" style="margin-bottom:16px;">
                                    <label class="form-label">"Hourly rate (YSH)"</label>
                                    <input class="form-input" type="text" placeholder="e.g. 50"
                                        prop:value=move || rate.get()
                                        on:input=move |ev| set_rate.set(event_target_value(&ev)) />
                                </div>
                                <button class="btn btn-primary" on:click=register_host
                                    prop:disabled=move || busy.get()>
                                    {move || if busy.get() { "Saving..." } else { "Save profile" }}
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
                let h = hosts.get();
                if h.is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-state-icon">"\u{1F465}"</div>
                            <div class="empty-state-title">"No hosts found"</div>
                            <div class="empty-state-text">"Become the first host!"</div>
                        </div>
                    }.into_any()
                } else {
                    let colors = ["#7856ff", "#1d9bf0", "#f91880", "#00ba7c"];
                    let is_self = is_self.clone();
                    view! {
                        <div class="item-grid">
                            {h.into_iter().enumerate().map(|(i, host)| {
                                let uid = host.get("user_id").and_then(|v| v.as_i64()).unwrap_or(i as i64);
                                let lang = host.get("languages").and_then(|v| v.as_str()).unwrap_or("N/A").to_string();
                                let rate = host.get("hourly_rate").and_then(|v| v.as_i64()).unwrap_or(0);
                                let available = host.get("available").and_then(|v| v.as_bool()).unwrap_or(false);
                                let rating = host.get("rating").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let color = colors[i % colors.len()];
                                let self_uid = uid;
                                let msg_uid = uid;
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
                                        </div>
                                        <div style="display:flex;flex-direction:column;align-items:flex-end;gap:8px;">
                                            {if available {
                                                view! { <span class="badge badge-success">"Online"</span> }.into_any()
                                            } else {
                                                view! { <span class="badge" style="color:var(--text-dim);">"Offline"</span> }.into_any()
                                            }}
                                            {if !is_self(self_uid) {
                                                view! {
                                                    <button class="btn btn-outline btn-sm"
                                                        on:click=move |_| {
                                                            let uid = msg_uid;
                                                            spawn_local(async move {
                                                                let req = serde_json::json!({"user_id": uid});
                                                                match api::post::<serde_json::Value>("/chat/session", &req).await {
                                                                    Ok(_) => { toast.success("Chat started!"); api::go("/chat"); }
                                                                    Err(e) => toast.error(format!("Failed: {e}")),
                                                                }
                                                            });
                                                        }>
                                                        "\u{1F4AC} Message"
                                                    </button>
                                                }.into_any()
                                            } else { view! {}.into_any() }}
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }
            }
        }}
    }
}
