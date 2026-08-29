use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::store;
use crate::components::ui::toast::ToastCtx;

#[component]
pub fn AgencyPage() -> impl IntoView {
    let (agencies, set_agencies) = signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = signal(true);
    let (show_create, set_show_create) = signal(false);
    let (name, set_name) = signal(String::new());
    let (desc, set_desc) = signal(String::new());
    let (creating, set_creating) = signal(false);
    let (joining_id, set_joining_id) = signal(Option::<i64>::None);
    let toast = ToastCtx::use_();

    let load = {
        let set_agencies = set_agencies.clone();
        let set_loading = set_loading.clone();
        move || {
            set_loading.set(true);
            spawn_local(async move {
                if let Ok(val) = api::get::<serde_json::Value>("/agencies").await {
                    if let Some(arr) = val.get("agencies").and_then(|v| v.as_array()) {
                        set_agencies.set(arr.clone());
                    }
                }
                set_loading.set(false);
            });
        }
    };
    load();

    let create_agency = move |_: leptos::ev::MouseEvent| {
        let n = name.get().trim().to_string();
        if n.is_empty() {
            toast.error("Agency name is required");
            return;
        }
        let d = desc.get();
        set_creating.set(true);
        spawn_local(async move {
            let req = serde_json::json!({"name": n, "description": d});
            match api::post::<serde_json::Value>("/agency", &req).await {
                Ok(_) => {
                    toast.success("Agency created!");
                    set_show_create.set(false);
                    set_name.set(String::new());
                    set_desc.set(String::new());
                }
                Err(e) => toast.error(format!("Failed: {e}")),
            }
            set_creating.set(false);
            // reload
            if let Ok(val) = api::get::<serde_json::Value>("/agencies").await {
                if let Some(arr) = val.get("agencies").and_then(|v| v.as_array()) {
                    set_agencies.set(arr.clone());
                }
            }
        });
    };

    let join_agency = move |agency_id: i64| {
        let me = match store::get_user() {
            Some(u) => u.user_id,
            None => {
                toast.error("Not logged in");
                return;
            }
        };
        set_joining_id.set(Some(agency_id));
        spawn_local(async move {
            let req = serde_json::json!({"user_id": me, "role": "host"});
            match api::post::<serde_json::Value>(&format!("/agency/{agency_id}/members"), &req).await {
                Ok(_) => toast.success("Joined agency!"),
                Err(e) => toast.error(format!("Failed: {e}")),
            }
            set_joining_id.set(None);
        });
    };

    view! {
        <div class="main-header">
            <div style="display:flex;justify-content:space-between;align-items:center;">
                <h2>"Agencies"</h2>
                <button class="btn btn-primary btn-sm" style="width:auto;padding:8px 16px;"
                    on:click=move |_| set_show_create.set(true)>
                    "+ New"
                </button>
            </div>
        </div>

        // Create modal
        {move || {
            if show_create.get() {
                view! {
                    <div class="modal-overlay" on:click=move |_| set_show_create.set(false)>
                        <div class="modal" on:click=move |ev| { ev.stop_propagation(); }>
                            <div class="modal-header">
                                <h2 class="modal-title">"Create Agency"</h2>
                                <button class="modal-close" on:click=move |_| set_show_create.set(false)>"\u{00d7}"</button>
                            </div>
                            <div style="padding:0 16px 16px;">
                                <div class="form-group" style="margin-bottom:16px;">
                                    <label class="form-label">"Name"</label>
                                    <input class="form-input" type="text" placeholder="Agency name"
                                        prop:value=move || name.get()
                                        on:input=move |ev| set_name.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-group" style="margin-bottom:16px;">
                                    <label class="form-label">"Description"</label>
                                    <textarea class="form-textarea" placeholder="What is this agency about?"
                                        prop:value=move || desc.get()
                                        on:input=move |ev| set_desc.set(event_target_value(&ev))></textarea>
                                </div>
                                <button class="btn btn-primary" on:click=create_agency
                                    prop:disabled=move || creating.get()>
                                    {move || if creating.get() { "Creating..." } else { "Create Agency" }}
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
                    let me = store::get_user().map(|u| u.user_id);
                    view! {
                        {list.into_iter().map(|a| {
                            let id = a.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                            let name = a.get("name").and_then(|v| v.as_str()).unwrap_or("Agency").to_string();
                            let desc = a.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let members = a.get("member_count").and_then(|v| v.as_i64()).unwrap_or(0);
                            let members_arr = a.get("members").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                            let owner = a.get("owner_id").and_then(|v| v.as_i64());
                            let is_member = members_arr.iter().any(|m| m.get("user_id").and_then(|v| v.as_i64()) == me);
                            let is_owner = owner == me;
                            let join = join_agency.clone();
                            view! {
                                <div class="item-card" style="position:relative;">
                                    <div class="avatar avatar-lg" style="background:#7856ff;">
                                        {name.chars().next().unwrap_or('A').to_uppercase().to_string()}
                                    </div>
                                    <div class="item-info" style="flex:1;">
                                        <div class="item-name">{name}</div>
                                        <div class="item-meta">{format!("{members} members")}</div>
                                        {if !desc.is_empty() { view! { <div class="item-meta" style="margin-top:4px;">{desc}</div> }.into_any() } else { view! {}.into_any() }}
                                        {if is_owner { view! { <div class="badge badge-accent" style="margin-top:6px;">"Owner"</div> }.into_any() } else if is_member { view! { <div class="badge badge-success" style="margin-top:6px;">"Member"</div> }.into_any() } else { view! {}.into_any() }}
                                    </div>
                                    {if !is_member {
                                        view! {
                                            <button class="btn btn-outline btn-sm" on:click=move |_| join(id)>
                                                {move || if joining_id.get() == Some(id) { "Joining..." } else { "Join" }}
                                            </button>
                                        }.into_any()
                                    } else { view! {}.into_any() }}
                                </div>
                            }
                        }).collect_view()}
                    }.into_any()
                }
            }
        }}
    }
}
