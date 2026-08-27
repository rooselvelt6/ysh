use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::store;

#[component]
pub fn ForgotPasswordPage() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (code, set_code) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let (u, c) = (username.get(), code.get());
        if u.is_empty() || c.is_empty() { set_error.set(Some("All fields required".into())); return; }
        set_error.set(None);
        set_loading.set(true);
        spawn_local(async move {
            let req = serde_json::json!({"username": u, "code": c});
            match api::post::<serde_json::Value>("/2fa/recovery/verify", &req).await {
                Ok(val) => {
                    if let (Some(at), Some(rt)) = (
                        val.get("access_token").and_then(|v| v.as_str()),
                        val.get("refresh_token").and_then(|v| v.as_str()),
                    ) { store::save_tokens(at, rt); api::go("/"); }
                }
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    };

    let is_loading = move || loading.get();

    view! {
        <div class="card">
            <h1 class="card-title">"Recovery Code"</h1>
            <p class="card-subtitle">"Enter your username and a recovery code"</p>
            {move || error.get().map(|msg| view! { <div class="alert alert-error">{msg}</div> })}
            <form on:submit=on_submit>
                <div class="form-group">
                    <label class="form-label">"Username"</label>
                    <input class="form-input" type="text" placeholder="your_username"
                        prop:value=move || username.get() on:input=move |ev| set_username.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label class="form-label">"Recovery code"</label>
                    <input class="form-input" type="text" placeholder="xxxx-xxxx-xxxx"
                        prop:value=move || code.get() on:input=move |ev| set_code.set(event_target_value(&ev)) />
                </div>
                <button class="btn btn-primary" type="submit" prop:disabled=is_loading>
                    {move || if loading.get() { view! { <span class="spinner"></span> " Signing in..." }.into_any() } else { view! { "Sign in" }.into_any() }}
                </button>
            </form>
            <div class="form-footer"><a href="/2fa">"Back to 2FA"</a> " · " <a href="/login">"Back to login"</a></div>
        </div>
    }
}
