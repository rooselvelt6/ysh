use leptos::prelude::*;
use crate::api;

#[component]
pub fn RegisterPage() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (confirm, set_confirm) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let (u, m, p, c) = (username.get(), email.get(), password.get(), confirm.get());
        if u.len() < 3 || u.len() > 32 { set_error.set(Some("Username must be 3-32 characters".into())); return; }
        if !m.contains('@') { set_error.set(Some("Invalid email".into())); return; }
        if p.len() < 8 { set_error.set(Some("Password must be at least 8 chars".into())); return; }
        if p != c { set_error.set(Some("Passwords do not match".into())); return; }
        set_loading.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let req = serde_json::json!({"username": u, "email": m, "password": p});
            match api::post::<serde_json::Value>("/register", &req).await {
                Ok(_) => api::go("/login"),
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
                <h1 class="auth-title">"Create your account"</h1>
                {move || error.get().map(|msg| view! { <div class="alert alert-error">{msg}</div> })}
                <form class="auth-form" on:submit=on_submit>
                    <div class="form-group">
                        <input class="form-input" type="text" placeholder="Username"
                            prop:value=move || username.get()
                            on:input=move |ev| set_username.set(event_target_value(&ev)) />
                    </div>
                    <div class="form-group">
                        <input class="form-input" type="email" placeholder="Email"
                            prop:value=move || email.get()
                            on:input=move |ev| set_email.set(event_target_value(&ev)) />
                    </div>
                    <div class="form-group">
                        <input class="form-input" type="password" placeholder="Password"
                            prop:value=move || password.get()
                            on:input=move |ev| set_password.set(event_target_value(&ev)) />
                    </div>
                    <div class="form-group">
                        <input class="form-input" type="password" placeholder="Confirm password"
                            prop:value=move || confirm.get()
                            on:input=move |ev| set_confirm.set(event_target_value(&ev)) />
                    </div>
                        <button class="btn btn-primary" type="submit" prop:disabled=is_disabled>
                        {move || if loading.get() { "Creating..." } else { "Create account" }}
                    </button>
                </form>
                <div class="form-footer">"Already have an account? " <a href="/login">"Sign in"</a></div>
            </div>
        </div>
    }
}
