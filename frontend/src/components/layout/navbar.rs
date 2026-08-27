use leptos::prelude::*;
use crate::store;
use crate::api;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav class="navbar">
            <a href="/" class="navbar-brand">"YSH"<span>" Hub"</span></a>
            <ul class="navbar-nav">
                <li><a href="/">"Discover"</a></li>
                <li><a href="/moments">"Moments"</a></li>
                <li><a href="/chat">"Chat"</a></li>
                <li><a href="/wallet">"Wallet"</a></li>
                <li><a href="/profile">"Profile"</a></li>
                <li>
                    <button
                        class="btn-ghost btn-sm"
                        on:click=move |_| {
                            store::clear_auth();
                            api::go("/login");
                        }
                    >
                        "Logout"
                    </button>
                </li>
            </ul>
        </nav>
    }
}
