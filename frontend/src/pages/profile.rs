use leptos::prelude::*;
use crate::api;
use crate::store;


#[component]
pub fn ProfilePage() -> impl IntoView {
    let (profile, set_profile) = signal(Option::<serde_json::Value>::None);
    let (loading, set_loading) = signal(true);
    let (editing, set_editing) = signal(false);
    let (display_name, set_display_name) = signal(String::new());
    let (bio, set_bio) = signal(String::new());
    let (country, set_country) = signal(String::new());

    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(val) = api::get::<serde_json::Value>("/profile").await {
            set_profile.set(Some(val));
        }
        set_loading.set(false);
    });

    let start_edit = move || {
        let p = profile.get();
        if let Some(p) = p {
            let prof = p.get("profile").cloned().unwrap_or_default();
            set_display_name.set(prof.get("display_name").and_then(|v| v.as_str()).unwrap_or("").to_string());
            set_bio.set(prof.get("bio").and_then(|v| v.as_str()).unwrap_or("").to_string());
            set_country.set(prof.get("country").and_then(|v| v.as_str()).unwrap_or("").to_string());
        }
        set_editing.set(true);
    };

    let save_profile = move |_: leptos::ev::MouseEvent| {
        let dn = display_name.get();
        let b = bio.get();
        let c = country.get();
        wasm_bindgen_futures::spawn_local(async move {
            let req = serde_json::json!({"display_name": dn, "bio": b, "country": c});
            let _ = api::post::<serde_json::Value>("/profile", &req).await;
            set_editing.set(false);
            if let Ok(val) = api::get::<serde_json::Value>("/profile").await {
                set_profile.set(Some(val));
            }
        });
    };

    view! {
        <div class="main-header">
            <h2>"Profile"</h2>
        </div>
        {move || {
            if loading.get() {
                view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else if editing.get() {
                view! {
                    <div style="padding:16px;">
                        <h3 style="font-weight:700;margin-bottom:16px;">"Edit Profile"</h3>
                        <div class="form-group" style="margin-bottom:16px;">
                            <label class="form-label">"Display Name"</label>
                            <input class="form-input" type="text" placeholder="Your name"
                                prop:value=move || display_name.get()
                                on:input=move |ev| set_display_name.set(event_target_value(&ev)) />
                        </div>
                        <div class="form-group" style="margin-bottom:16px;">
                            <label class="form-label">"Bio"</label>
                            <textarea class="form-textarea" placeholder="Tell us about yourself"
                                on:input=move |ev| set_bio.set(event_target_value(&ev))>{move || bio.get()}</textarea>
                        </div>
                        <div class="form-group" style="margin-bottom:16px;">
                            <label class="form-label">"Country"</label>
                            <input class="form-input" type="text" placeholder="Your country"
                                prop:value=move || country.get()
                                on:input=move |ev| set_country.set(event_target_value(&ev)) />
                        </div>
                        <div style="display:flex;gap:8px;">
                            <button class="btn btn-primary" style="flex:1;" on:click=save_profile>"Save"</button>
                            <button class="btn btn-outline" style="flex:1;" on:click=move |_| set_editing.set(false)>"Cancel"</button>
                        </div>
                    </div>
                }.into_any()
            } else {
                let p = profile.get().unwrap_or_default();
                let prof = p.get("profile").cloned().unwrap_or_default();
                let balance = p.get("wallet_balance").and_then(|v| v.as_i64()).unwrap_or(0);
                let display = prof.get("display_name").and_then(|v| v.as_str()).unwrap_or("Anonymous").to_string();
                let bio_text = prof.get("bio").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let country_text = prof.get("country").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let avatar_url = prof.get("avatar_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let initial = display.chars().next().unwrap_or('U').to_uppercase().to_string();

                view! {
                    <div class="profile-banner"></div>
                    <div class="profile-info-section">
                        <div class="profile-avatar-row">
                            <div class="avatar avatar-xl" style={format!("margin-top:-40px;background:#7856ff;")}>
                                {if avatar_url.is_empty() { view! { <span>{initial}</span> }.into_any() }
                                else { view! { <img src={avatar_url} alt="avatar" /> }.into_any() }}
                            </div>
                            <button class="btn btn-outline btn-sm" style="margin-top:50px;" on:click=move |_| start_edit()>"Edit profile"</button>
                        </div>
                        <div class="profile-name">{display}</div>
                        <div class="profile-handle">{move || store::get_user().map(|u| format!("@{}", u.username)).unwrap_or_else(|| "@user".into())}</div>
                        {if !bio_text.is_empty() { view! { <div class="profile-bio">{bio_text}</div> }.into_any() } else { view! {}.into_any() }}
                        <div class="profile-meta">
                            {if !country_text.is_empty() { view! { <span>{format!("\u{1F30D} {}", country_text)}</span> }.into_any() } else { view! {}.into_any() }}
                            <span>{format!("\u{2B50} {} YSH balance", balance)}</span>
                        </div>
                    </div>
                }.into_any()
            }
        }}
    }
}
