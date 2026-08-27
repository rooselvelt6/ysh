use leptos::prelude::*;

#[component]
pub fn Modal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] title: String,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        {move || {
            if open.get() {
                view! {
                    <div class="modal-overlay">
                        <div class="modal">
                            <div class="modal-header">
                                <h2 class="modal-title">{title.clone()}</h2>
                            </div>
                            {children()}
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {}.into_any()
            }
        }}
    }
}
