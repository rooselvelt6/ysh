use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_navigate;
use crate::store;

#[component]
pub fn ProtectedRoute() -> impl IntoView {
    let navigate = use_navigate();

    Effect::new(move |_| {
        if !store::is_logged_in() {
            navigate("/login", Default::default());
        }
    });

    let logged_in = move || store::is_logged_in();

    view! {
        <Show when=logged_in fallback=|| ()>
            <Outlet/>
        </Show>
    }
}