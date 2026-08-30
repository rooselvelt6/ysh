use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::ui::toast::ToastCtx;

use super::{s, truncate};

#[component]
pub fn I18nTab() -> impl IntoView {
    let (loading, set_loading) = signal(true);
    let (overrides, set_overrides) = signal(Vec::<serde_json::Value>::new());
    let (locales, set_locales) = signal(Vec::<serde_json::Value>::new());
    let (locale_in, set_locale_in) = signal(String::new());
    let (key_in, set_key_in) = signal(String::new());
    let (value_in, set_value_in) = signal(String::new());
    let toast = ToastCtx::use_();

    let reload = {
        let set_loading = set_loading.clone();
        move || {
            set_loading.set(true);
            spawn_local(async move {
                if let Ok(v) = api::get::<serde_json::Value>("/admin/i18n").await {
                    if let Some(arr) = v.get("overrides").and_then(|x| x.as_array()) {
                        set_overrides.set(arr.clone());
                    }
                    if let Some(arr) = v.get("locales").and_then(|x| x.as_array()) {
                        set_locales.set(arr.clone());
                    }
                    if locale_in.get().is_empty() {
                        if let Some(first) = v.get("locales").and_then(|a| a.as_array()).and_then(|a| a.first()).and_then(|x| x.as_str()) {
                            set_locale_in.set(first.to_string());
                        }
                    }
                }
                set_loading.set(false);
            });
        }
    };
    reload();

    let save = move |_: leptos::ev::SubmitEvent| {
        let locale = locale_in.get();
        let key = key_in.get();
        let value = value_in.get();
        if key.trim().is_empty() {
            toast.error("La clave es obligatoria");
            return;
        }
        let reload = reload.clone();
        spawn_local(async move {
            let body = serde_json::json!({"locale": locale, "key": key, "value": value});
            match api::post::<serde_json::Value>("/admin/i18n", &body).await {
                Ok(_) => {
                    toast.success("Override guardado");
                    set_key_in.set(String::new());
                    set_value_in.set(String::new());
                    reload();
                }
                Err(e) => toast.error(format!("Failed: {e}")),
            }
        });
    };

    view! {
        {move || {
            if loading.get() {
                return view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any();
            }
            let ov = overrides.get();
            view! {
                <div style="margin-top:16px;">
                    <div class="panel">
                        <div class="panel-title">"Nuevo override de traducción"</div>
                        <form on:submit=save style="display:flex;gap:8px;flex-wrap:wrap;align-items:center;">
                            <select
                                style="padding:8px 12px;border-radius:10px;border:1px solid var(--border);background:var(--surface);color:var(--text);"
                                prop:value=move || locale_in.get()
                                on:change=move |ev| set_locale_in.set(event_target_value(&ev))
                            >
                                {locales.get().iter().map(|l| {
                                    let code = l.as_str().unwrap_or("").to_string();
                                    view! {
                                        <option value={code.clone()} selected=code==locale_in.get()>{code.clone()}</option>
                                    }
                                }).collect_view()}
                                {if locales.get().is_empty() {
                                    view! { <option value="es">"es"</option> }.into_any()
                                } else {
                                    view! {}.into_any()
                                }}
                            </select>
                            <input
                                type="text"
                                placeholder="clave (p.ej. auth.login)"
                                value=move || key_in.get()
                                on:input=move |ev| set_key_in.set(event_target_value(&ev))
                                style="flex:1;min-width:180px;padding:8px 12px;border:1px solid var(--border);border-radius:10px;background:var(--surface);color:var(--text);"
                            />
                            <input
                                type="text"
                                placeholder="traducción"
                                value=move || value_in.get()
                                on:input=move |ev| set_value_in.set(event_target_value(&ev))
                                style="flex:1;min-width:180px;padding:8px 12px;border:1px solid var(--border);border-radius:10px;background:var(--surface);color:var(--text);"
                            />
                            <button class="btn btn-primary" type="submit">"Guardar"</button>
                        </form>
                    </div>

                    <div class="panel">
                        <div class="panel-title">"Overrides actuales"</div>
                        {if ov.is_empty() {
                            view! { <div class="item-meta">"Sin overrides definidos."</div> }.into_any()
                        } else {
                            view! {
                                <table class="data-table">
                                    <thead>
                                        <tr><th>"Locale"</th><th>"Clave"</th><th>"Valor"</th><th></th></tr>
                                    </thead>
                                    <tbody>
                                        {ov.iter().map(|o| {
                                            let o = o.clone();
                                            let locale = s(&o, "locale");
                                            let key = s(&o, "key");
                                            let reload = reload.clone();
                                            view! {
                                                <tr>
                                                    <td><span class="badge">{locale.clone()}</span></td>
                                                    <td><strong>{key.clone()}</strong></td>
                                                    <td style="color:var(--text-dim);">{truncate(&s(&o, "value"), 60)}</td>
                                                    <td>
                                                        <button
                                                            class="btn btn-sm"
                                                            style="background:var(--danger,#dc3545);color:#fff;"
                                                            on:click={let reload = reload.clone(); move |_| {
                                                                let reload = reload.clone();
                                                                let reload2 = reload.clone();
                                                                let lc = locale.clone();
                                                                let kc = key.clone();
                                                                spawn_local(async move {
                                                                    match api::del::<serde_json::Value>(&format!("/admin/i18n/{}/{}", lc, kc)).await {
                                                                        Ok(_) => { toast.success("Override eliminado"); reload(); }
                                                                        Err(e) => { toast.error(format!("Failed: {e}")); reload2(); }
                                                                    }
                                                                });
                                                            }}
                                                        >
                                                            "\u{1F5D1}"
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_any()
                        }}
                    </div>
                </div>
            }.into_any()
        }}
    }
}