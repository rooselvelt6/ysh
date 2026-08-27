use leptos::prelude::*;
use crate::api;

#[component]
pub fn GiftsPage() -> impl IntoView {
    let (catalog, set_catalog) = signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = signal(true);
    let (selected, set_selected) = signal(Option::<i64>::None);

    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(val) = api::get::<serde_json::Value>("/gifts/catalog").await {
            if let Some(arr) = val.get("gifts").and_then(|v| v.as_array()) {
                set_catalog.set(arr.clone());
            }
        }
        set_loading.set(false);
    });

    let gift_icons = |rarity: &str| -> &str {
        match rarity {
            "common" => "\u{26AA}",
            "rare" => "\u{1F535}",
            "epic" => "\u{1F7E3}",
            "legendary" => "\u{1F451}",
            _ => "\u{2753}",
        }
    };

    view! {
        <div class="main-header">
            <h2>"Gift Shop"</h2>
        </div>
        <div class="main-tabs">
            <button class="main-tab active">"Catalog"</button>
            <button class="main-tab">"Received"</button>
            <button class="main-tab">"Sent"</button>
            <button class="main-tab">"NFTs"</button>
        </div>
        {move || {
            if loading.get() {
                view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else {
                let gifts = catalog.get();
                if gifts.is_empty() {
                    view! { <div class="empty-state"><div class="empty-state-title">"No gifts in catalog"</div></div> }.into_any()
                } else {
                    view! {
                        <div class="gift-grid">
                            {gifts.into_iter().map(|g| {
                                let id = g.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                                let name = g.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let price = g.get("price").and_then(|v| v.as_i64()).unwrap_or(0);
                                let rarity = g.get("rarity").and_then(|v| v.as_str()).unwrap_or("common").to_string();
                                let icon = gift_icons(&rarity).to_string();
                                view! {
                                    <div class={move || format!("gift-card{}", if selected.get() == Some(id) { " selected" } else { "" })}
                                        on:click=move |_| set_selected.set(Some(id))>
                                        <div class="gift-icon">{icon}</div>
                                        <div class="gift-name">{name}</div>
                                        <div class="gift-price">{format!("{price} YSH")}</div>
                                        <span class={format!("badge badge-{}", rarity)}>{rarity.clone()}</span>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }
            }
        }}
    }
}
