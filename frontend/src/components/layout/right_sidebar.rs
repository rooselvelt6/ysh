use leptos::prelude::*;

#[component]
pub fn RightSidebar() -> impl IntoView {
    let trending = vec![
        ("YSH Live", "Trending \u{00B7} Entertainment"),
        ("Gifts", "Trending \u{00B7} 1.2K posts"),
        ("New Hosts", "Trending \u{00B7} Community"),
        ("Live Streaming", "Trending \u{00B7} Technology"),
    ];

    let who_to_follow = vec![
        ("Maria", "@maria_live", "Live host"),
        ("Alex", "@alex_stream", "Content creator"),
        ("YSH Official", "@ysh_official", "Platform updates"),
    ];

    view! {
        <div class="right-sidebar">
            <div class="right-sidebar-inner">
                <div class="search-box">
                    <div class="search-wrapper">
                        <span class="search-icon">"\u{1F50D}"</span>
                        <input
                            class="search-input"
                            type="text"
                            placeholder="Search YSH"
                        />
                    </div>
                </div>

                <div class="widget">
                    <div class="widget-header">"Trends for you"</div>
                    {trending.into_iter().map(|(title, meta)| {
                        view! {
                            <div class="widget-item">
                                <div class="widget-label">{meta}</div>
                                <div class="widget-title">{title}</div>
                            </div>
                        }
                    }).collect_view()}
                    <div class="widget-footer">"Show more"</div>
                </div>

                <div class="widget">
                    <div class="widget-header">"Who to follow"</div>
                    {who_to_follow.into_iter().map(|(name, handle, _bio)| {
                        view! {
                            <div class="widget-item" style="display:flex;align-items:center;gap:12px;">
                                <div class="avatar avatar-sm" style="background:#7856ff;">
                                    {name.chars().next().unwrap_or('U').to_uppercase().to_string()}
                                </div>
                                <div style="flex:1;min-width:0;">
                                    <div class="widget-title" style="font-size:0.875rem;">{name}</div>
                                    <div class="widget-meta">{handle}</div>
                                </div>
                                <button class="btn btn-outline btn-sm" style="flex-shrink:0;border-radius:9999px;padding:6px 16px;font-size:0.8125rem;">
                                    "Follow"
                                </button>
                            </div>
                        }
                    }).collect_view()}
                    <div class="widget-footer">"Show more"</div>
                </div>

                <div style="padding:12px 16px;color:var(--text-dim);font-size:0.8125rem;line-height:1.8;">
                    "\u{00A9} 2026 YSH \u{00B7} Terms \u{00B7} Privacy \u{00B7} Cookies"
                </div>
            </div>
        </div>
    }
}
