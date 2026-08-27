use leptos::prelude::*;

#[component]
pub fn Badge(
    #[prop(into)] text: String,
    #[prop(default = "accent".to_string())] variant: String,
) -> impl IntoView {
    view! { <span class={format!("badge badge-{variant}")}>{text}</span> }
}

#[component]
pub fn VerificationBadge() -> impl IntoView {
    view! {
        <span class="verification-badge">
            <span>"\u{2713} Verified"</span>
        </span>
    }
}
