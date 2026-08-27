use leptos::prelude::*;

#[derive(Clone, PartialEq)]
pub struct TabItem {
    pub id: String,
    pub label: String,
}

#[component]
pub fn Tabs(
    items: Vec<TabItem>,
    #[prop(into)] active: Signal<String>,
    #[prop(into)] on_change: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="tabs">
            {items.into_iter().map(|item| {
                let id_clone = item.id.clone();
                let label = item.label;
                let is_active = active.clone();
                let on_change = on_change.clone();
                view! {
                    <button
                        class={move || format!("tab {}", if is_active.get() == id_clone { "active" } else { "" })}
                        on:click=move |_| on_change.run(item.id.clone())
                    >
                        {label}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}
