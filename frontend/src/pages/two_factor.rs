use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::store;

#[component]
pub fn TwoFactorPage() -> impl IntoView {
    let (code, set_code) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let c = code.get().trim().to_string();
        if c.len() != 6 { set_error.set(Some("Enter the 6-digit code".into())); return; }
        set_error.set(None);
        set_loading.set(true);
        spawn_local(async move {
            let temp = store::with_token(|t| t.map(|s| s.to_string())).unwrap_or_default();
            let req = serde_json::json!({"temp_token": temp, "code": c});
            match api::post::<serde_json::Value>("/login/2fa", &req).await {
                Ok(val) => {
                    if let (Some(at), Some(rt)) = (
                        val.get("access_token").and_then(|v| v.as_str()),
                        val.get("refresh_token").and_then(|v| v.as_str()),
                    ) {
                        store::save_tokens(at, rt);
                        if let Ok(ui) = api::get::<serde_json::Value>("/me").await {
                            store::save_user(&store::UserInfo {
                                user_id: ui["user_id"].as_i64().unwrap_or(0),
                                role: ui["role"].as_str().unwrap_or("user").to_string(),
                                username: ui["username"].as_str().unwrap_or("user").to_string(),
                            });
                        }
                        api::go("/");
                    }
                }
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    };

    let is_disabled = move || loading.get() || code.get().len() != 6;

    view! {
        <div class="auth-page">
            <div class="auth-card" style="text-align:center;">
                <div class="auth-logo"><h1>"YSH"</h1></div>
                <h1 class="auth-title">"Two-Factor Authentication"</h1>
                <p style="color:var(--text-dim);margin-bottom:32px;">"Enter the 6-digit code from your authenticator"</p>
                {move || error.get().map(|msg| view! { <div class="alert alert-error">{msg}</div> })}
                <form class="auth-form" on:submit=on_submit>
                    <div class="form-group">
                        <input class="form-input" type="text" inputmode="numeric" maxlength="6" placeholder="000000"
                            style="text-align:center;font-size:1.5rem;font-weight:700;letter-spacing:0.5rem;"
                            prop:value=move || code.get()
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                set_code.set(v.chars().filter(|c| c.is_ascii_digit()).take(6).collect());
                            } />
                    </div>
                    <button class="btn btn-primary" type="submit" prop:disabled=is_disabled>
                        {move || if loading.get() { "Verifying..." } else { "Verify" }}
                    </button>
                </form>
                <div class="form-footer"><a href="/recovery">"Use a recovery code instead"</a></div>
            </div>
        </div>
    }
}
