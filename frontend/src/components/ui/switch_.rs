use leptos::prelude::*;

#[component]
pub fn Switch(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_change: Callback<bool>,
    #[prop(optional, into)] label: String,
) -> impl IntoView {
    let uid = format!("sw-{:p}", &checked as *const _);
    let uid2 = uid.clone();
    view! {
        <label class="switch" for={uid}>
            <input
                type="checkbox"
                id={uid2}
                prop:checked=move || checked.get()
                on:change=move |ev| {
                    let checked = event_target_checked(&ev);
                    on_change.run(checked);
                }
            />
            <span class="switch-track"></span>
            <span class="switch-thumb"></span>
            {if !label.is_empty() {
                view! { <span style="margin-left:0.5rem;font-size:0.9rem;">{label}</span> }.into_any()
            } else {
                view! {}.into_any()
            }}
        </label>
    }
}
