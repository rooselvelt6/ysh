use leptos::prelude::*;
use leptos_router::*;
use crate::store;

#[component]
pub fn ProtectedRoute() -> impl IntoView {
    let logged_in = Memo::new(move |_| store::is_logged_in());

    view! {
        {move || {
            if logged_in.get() {
                view! { <Outlet/> }.into_any()
            } else {
                navigate("/login");
                view! { <div>"Redirecting..."</div> }.into_any()
            }
        }}
    }
}
