use leptos::prelude::*;
use leptos_router::hooks::use_location;
use crate::store;
use crate::api;

#[component]
pub fn Sidebar() -> impl IntoView {
    let loc = use_location();
    let path = move || loc.pathname.get();

    let mut nav_items = vec![
        ("/", "\u{2302}", "Home"),
        ("/discover", "\u{1F50D}", "Discover"),
        ("/notifications", "\u{1F514}", "Notifications"),
        ("/chat", "\u{1F4AC}", "Messages"),
        ("/gifts", "\u{1F381}", "Gifts"),
        ("/wallet", "\u{1F4B0}", "Wallet"),
        ("/hosts", "\u{1F465}", "Hosts"),
        ("/agency", "\u{1F3E2}", "Agency"),
        ("/stream", "\u{1F4FA}", "Live"),
        ("/moments", "\u{1F4F7}", "Moments"),
        ("/profile", "\u{1F464}", "Profile"),
    ];
    if store::get_user().map(|u| u.role == "admin").unwrap_or(false) {
        nav_items.push(("/admin", "\u{1F6E1}\u{FE0F}", "Admin"));
    }

    view! {
        <aside class="sidebar">
            <div class="sidebar-logo">
                <a href="/">"YSH"</a>
            </div>
            <nav class="sidebar-nav">
                {nav_items.into_iter().map(|(href, icon, label)| {
                    view! {
                        <a
                            href={href}
                            class={move || {
                                let p = path();
                                let active = if p == href { " active" } else { "" };
                                format!("sidebar-link{}", active)
                            }}
                        >
                            <span class="icon">{icon}</span>
                            <span class="label">{label}</span>
                        </a>
                    }
                }).collect_view()}
                <div class="sidebar-compose">
                    <button
                        class="btn"
                        on:click=move |_| api::go("/moments")
                    >
                        <span>"Post"</span>
                    </button>
                </div>
            </nav>
            <div
                class="sidebar-profile"
                on:click=move |_| api::go("/profile")
            >
                <div class="avatar avatar-md" style="background:#7856ff;">
                    {move || {
                        let u = store::get_user();
                        let c = u.map(|u| u.username.chars().next().unwrap_or('U'));
                        c.unwrap_or('U').to_uppercase().to_string()
                    }}
                </div>
                <div style="flex:1;min-width:0;">
                    <div style="font-weight:700;font-size:0.9375rem;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">
                        {move || store::get_user().map(|u| u.username.clone()).unwrap_or_else(|| "Guest".into())}
                    </div>
                    <div style="color:var(--text-dim);font-size:0.8125rem;">
                        {move || store::get_user().map(|u| format!("@{}", u.username)).unwrap_or_else(|| "@user".into())}
                    </div>
                </div>
                <button
                    style="background:none;border:none;color:var(--text-dim);cursor:pointer;font-size:1.1rem;padding:4px;"
                    title="Logout"
                    on:click=move |_| {
                        store::clear_auth();
                        api::go("/login");
                    }
                >
                    "\u{2192}"
                </button>
            </div>
        </aside>
    }
}
