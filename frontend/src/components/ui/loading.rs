use leptos::prelude::*;

#[component]
pub fn LoadingSpinner(
    #[prop(default = false)] large: bool,
    #[prop(default = "Loading...".to_string())] text: String,
) -> impl IntoView {
    let size = if large { "spinner-lg" } else { "" };
    view! {
        <div class="loading-center">
            <div style="text-align:center;">
                <div class={format!("spinner {size}")}></div>
                <div style="margin-top:0.75rem;color:var(--text-secondary);font-size:0.85rem;">{text}</div>
            </div>
        </div>
    }
}
