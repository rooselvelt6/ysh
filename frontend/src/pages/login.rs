use leptos::prelude::*;
use crate::store;
use crate::api;

#[component]
pub fn LoginPage() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_loading.set(true);
        let user = username.get();
        let pass = password.get();
        wasm_bindgen_futures::spawn_local(async move {
            let req = serde_json::json!({"username": user, "password": pass});
            match api::post::<serde_json::Value>("/login", &req).await {
                Ok(val) => {
                    if val.get("requires_2fa").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let temp = val.get("temp_token").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        store::save_tokens(&temp, "");
                        api::go("/2fa");
                    } else if let (Some(at), Some(rt)) = (
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

    let is_disabled = move || loading.get();

    view! {
        <div class="auth-page">
            <div class="auth-card">
                <div class="auth-logo"><h1>"YSH"</h1></div>
                <h1 class="auth-title">"Sign in to YSH"</h1>
                {move || error.get().map(|msg| view! { <div class="alert alert-error">{msg}</div> })}
                <form class="auth-form" on:submit=on_submit>
                    <div class="form-group">
                        <input class="form-input" type="text" placeholder="Username"
                            prop:value=move || username.get()
                            on:input=move |ev| set_username.set(event_target_value(&ev)) />
                    </div>
                    <div class="form-group">
                        <input class="form-input" type="password" placeholder="Password"
                            prop:value=move || password.get()
                            on:input=move |ev| set_password.set(event_target_value(&ev)) />
                    </div>
                    <button class="btn btn-primary" type="submit" prop:disabled=is_disabled>
                        {move || if loading.get() { "Signing in..." } else { "Sign in" }}
                    </button>
                </form>
                <div class="form-footer"><a href="/recovery">"Forgot password?"</a></div>
                <div class="form-footer">"Don't have an account? " <a href="/register">"Sign up"</a></div>
            </div>
        </div>
    }
}
