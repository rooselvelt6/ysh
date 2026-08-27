use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn BottomNav() -> impl IntoView {
    let loc = use_location();
    let path = move || loc.pathname.get();

    let items = vec![
        ("/", "\u{2302}", "Home"),
        ("/discover", "\u{1F50D}", "Search"),
        ("/notifications", "\u{1F514}", "Alerts"),
        ("/chat", "\u{1F4AC}", "Chat"),
        ("/profile", "\u{1F464}", "Profile"),
    ];

    view! {
        <nav class="bottom-nav">
            <div class="bottom-nav-items">
                {items.into_iter().map(|(href, icon, label)| {
                    view! {
                        <a
                            href={href}
                            class={move || {
                                let p = path();
                                let active = if p == href { " active" } else { "" };
                                format!("bottom-nav-item{}", active)
                            }}
                        >
                            <span class="bottom-nav-icon">{icon}</span>
                            <span>{label}</span>
                        </a>
                    }
                }).collect_view()}
            </div>
        </nav>
    }
}
