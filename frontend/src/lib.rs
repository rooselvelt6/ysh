pub mod api;
pub mod store;
pub mod components;
pub mod pages;

use leptos::prelude::*;
use leptos_router::{components::*, path};

use crate::components::ui::toast::{ToastCtx, ToastContainer};
use crate::components::layout::sidebar::Sidebar;
use crate::components::layout::right_sidebar::RightSidebar;
use crate::components::layout::bottom_nav::BottomNav;
use crate::pages::login::LoginPage;
use crate::pages::register::RegisterPage;
use crate::pages::two_factor::TwoFactorPage;
use crate::pages::forgot_password::ForgotPasswordPage;
use crate::pages::dashboard::DashboardPage;
use crate::pages::discover::DiscoverPage;
use crate::pages::wallet::WalletPage;
use crate::pages::profile::ProfilePage;
use crate::pages::moments::MomentsPage;
use crate::pages::gifts::GiftsPage;
use crate::pages::hosts::HostsPage;
use crate::pages::agency::AgencyPage;
use crate::pages::chat::ChatPage;
use crate::pages::notifications::NotificationsPage;
use crate::pages::stream::StreamPage;
use crate::pages::admin::AdminPage;

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).ok();
    store::init_auth();
    mount_to_body(App);
}

#[component]
pub fn App() -> impl IntoView {
    let _ = ToastCtx::provide();
    let _ = store::init_theme();

    view! {
        <ToastContainer/>
        <Router>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/login") view=LoginPage/>
                <Route path=path!("/register") view=RegisterPage/>
                <Route path=path!("/2fa") view=TwoFactorPage/>
                <Route path=path!("/recovery") view=ForgotPasswordPage/>
                <Route path=path!("/") view=Home/>
                <Route path=path!("/discover") view=DiscoverShell/>
                <Route path=path!("/wallet") view=WalletShell/>
                <Route path=path!("/profile") view=ProfileShell/>
                <Route path=path!("/moments") view=MomentsShell/>
                <Route path=path!("/gifts") view=GiftsShell/>
                <Route path=path!("/hosts") view=HostsShell/>
                <Route path=path!("/agency") view=AgencyShell/>
                <Route path=path!("/chat") view=ChatShell/>
                <Route path=path!("/notifications") view=NotificationsShell/>
                <Route path=path!("/stream") view=StreamShell/>
                <Route path=path!("/admin") view=AdminShell/>
            </Routes>
        </Router>
    }
}

#[component]
fn Home() -> impl IntoView {
    view! {
        <div class="app-shell">
            <Sidebar/>
            <main class="app-main">
                <DashboardPage/>
            </main>
            <RightSidebar/>
        </div>
        <BottomNav/>
    }
}

#[component]
fn DiscoverShell() -> impl IntoView {
    view! {
        <div class="app-shell">
            <Sidebar/>
            <main class="app-main">
                <DiscoverPage/>
            </main>
            <RightSidebar/>
        </div>
        <BottomNav/>
    }
}

#[component]
fn WalletShell() -> impl IntoView {
    view! {
        <div class="app-shell">
            <Sidebar/>
            <main class="app-main">
                <WalletPage/>
            </main>
            <RightSidebar/>
        </div>
        <BottomNav/>
    }
}

#[component]
fn ProfileShell() -> impl IntoView {
    view! {
        <div class="app-shell">
            <Sidebar/>
            <main class="app-main">
                <ProfilePage/>
            </main>
            <RightSidebar/>
        </div>
        <BottomNav/>
    }
}

#[component]
fn MomentsShell() -> impl IntoView {
    view! {
        <div class="app-shell">
            <Sidebar/>
            <main class="app-main">
                <MomentsPage/>
            </main>
            <RightSidebar/>
        </div>
        <BottomNav/>
    }
}

#[component]
fn GiftsShell() -> impl IntoView {
    view! {
        <div class="app-shell">
            <Sidebar/>
            <main class="app-main">
                <GiftsPage/>
            </main>
            <RightSidebar/>
        </div>
        <BottomNav/>
    }
}

#[component]
fn HostsShell() -> impl IntoView {
    view! {
        <div class="app-shell">
            <Sidebar/>
            <main class="app-main">
                <HostsPage/>
            </main>
            <RightSidebar/>
        </div>
        <BottomNav/>
    }
}

#[component]
fn AgencyShell() -> impl IntoView {
    view! {
        <div class="app-shell">
            <Sidebar/>
            <main class="app-main">
                <AgencyPage/>
            </main>
            <RightSidebar/>
        </div>
        <BottomNav/>
    }
}

#[component]
fn ChatShell() -> impl IntoView {
    view! {
        <div class="app-shell">
            <Sidebar/>
            <main class="app-main">
                <ChatPage/>
            </main>
            <RightSidebar/>
        </div>
        <BottomNav/>
    }
}

#[component]
fn NotificationsShell() -> impl IntoView {
    view! {
        <div class="app-shell">
            <Sidebar/>
            <main class="app-main">
                <NotificationsPage/>
            </main>
            <RightSidebar/>
        </div>
        <BottomNav/>
    }
}

#[component]
fn StreamShell() -> impl IntoView {
    view! {
        <div class="app-shell">
            <Sidebar/>
            <main class="app-main">
                <StreamPage/>
            </main>
            <RightSidebar/>
        </div>
        <BottomNav/>
    }
}

#[component]
fn AdminShell() -> impl IntoView {
    view! {
        <div class="app-shell">
            <Sidebar/>
            <main class="app-main">
                <AdminPage/>
            </main>
            <RightSidebar/>
        </div>
        <BottomNav/>
    }
}
