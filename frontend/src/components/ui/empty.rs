use leptos::prelude::*;

#[component]
pub fn EmptyState(
    #[prop(default = "\u{1F4ED}".to_string())] icon: String,
    #[prop(into)] title: String,
    #[prop(optional, into)] text: String,
    #[prop(optional)] action: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    view! {
        <div class="empty-state">
            <div class="empty-state-icon">{icon}</div>
            <div class="empty-state-title">{title}</div>
            {if !text.is_empty() {
                view! { <div class="empty-state-text">{text}</div> }.into_any()
            } else {
                view! {}.into_any()
            }}
            {match action {
                Some(f) => view! { <button class="btn btn-primary btn-sm" on:click=move |_| f()>"Action"</button> }.into_any(),
                None => view! {}.into_any(),
            }}
        </div>
    }
}
