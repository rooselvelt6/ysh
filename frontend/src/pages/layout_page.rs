use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::store;
use crate::components::layout::{navbar::Navbar, sidebar::Sidebar, bottom_nav::BottomNav};
use crate::components::ui::toast::{ToastContainer, ToastCtx};

#[component]
pub fn AppShell() -> impl IntoView {
    store::init_auth();
    store::init_theme();
    let logged_in = Memo::new(move |_| store::is_logged_in());

    view! {
        {move || {
            if logged_in.get() {
                view! {
                    <ToastContainer/>
                    <Navbar/>
                    <div class="app-body">
                        <Sidebar/>
                        <main class="app-main">
                            <crate::pages::dashboard::DashboardPage/>
                        </main>
                    </div>
                    <BottomNav/>
                }.into_any()
            } else {
                view! {
                    <crate::pages::login::LoginPage/>
                }.into_any()
            }
        }}
    }
}
