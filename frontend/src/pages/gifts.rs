use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::components::ui::toast::ToastCtx;

#[derive(Clone, Copy, PartialEq, Eq)]
enum GiftTab { Catalog, Received, Sent, Nfts }

#[component]
pub fn GiftsPage() -> impl IntoView {
    let (catalog, set_catalog) = signal(Vec::<serde_json::Value>::new());
    let (received, set_received) = signal(Vec::<serde_json::Value>::new());
    let (sent, set_sent) = signal(Vec::<serde_json::Value>::new());
    let (nfts, set_nfts) = signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = signal(true);
    let (tab, set_tab) = signal(GiftTab::Catalog);
    let (send_target, set_send_target) = signal(Option::<i64>::None); // selected gift id to send
    let (recipient, set_recipient) = signal(String::new());
    let (busy, set_busy) = signal(false);
    let toast = ToastCtx::use_();

    let load_catalog = {
        let set_catalog = set_catalog.clone();
        move || {
            spawn_local(async move {
                if let Ok(val) = api::get::<serde_json::Value>("/gifts/catalog").await {
                    if let Some(arr) = val.get("gifts").and_then(|v| v.as_array()) {
                        set_catalog.set(arr.clone());
                    }
                }
            });
        }
    };
    let load_received = {
        let set_received = set_received.clone();
        move || {
            spawn_local(async move {
                if let Ok(val) = api::get::<serde_json::Value>("/gifts/received").await {
                    if let Some(arr) = val.get("gifts").and_then(|v| v.as_array()) {
                        set_received.set(arr.clone());
                    }
                }
            });
        }
    };
    let load_sent = {
        let set_sent = set_sent.clone();
        move || {
            spawn_local(async move {
                if let Ok(val) = api::get::<serde_json::Value>("/gifts/sent").await {
                    if let Some(arr) = val.get("gifts").and_then(|v| v.as_array()) {
                        set_sent.set(arr.clone());
                    }
                }
            });
        }
    };
    let load_nfts = {
        let set_nfts = set_nfts.clone();
        move || {
            spawn_local(async move {
                if let Ok(val) = api::get::<serde_json::Value>("/gifts/nft").await {
                    if let Some(arr) = val.get("nfts").and_then(|v| v.as_array()) {
                        set_nfts.set(arr.clone());
                    }
                    if let Some(arr) = val.get("gifts").and_then(|v| v.as_array()) {
                        set_nfts.set(arr.clone());
                    }
                }
            });
        }
    };

    let load_all = {
        let load_catalog = load_catalog.clone();
        let load_received = load_received.clone();
        let load_sent = load_sent.clone();
        let load_nfts = load_nfts.clone();
        let set_loading = set_loading.clone();
        move || {
            load_catalog();
            load_received();
            load_sent();
            load_nfts();
            set_loading.set(false);
        }
    };
    load_all();

    let send_gift = move |_: leptos::ev::MouseEvent| {
        let gift_id = match send_target.get() {
            Some(g) => g,
            None => {
                toast.error("Select a gift first");
                return;
            }
        };
        let to_user: i64 = match recipient.get().trim().parse() {
            Ok(n) => n,
            Err(_) => {
                toast.error("Enter a valid recipient user ID");
                return;
            }
        };
        set_busy.set(true);
        spawn_local(async move {
            let req = serde_json::json!({"gift_id": gift_id});
            match api::post::<serde_json::Value>(&format!("/gifts/send/{to_user}"), &req).await {
                Ok(_) => {
                    toast.success("Gift sent!");
                    set_send_target.set(None);
                    set_recipient.set(String::new());
                }
                Err(e) => toast.error(format!("Failed: {e}")),
            }
            set_busy.set(false);
        });
    };

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
            <button class={move || format!("main-tab{}", if tab.get() == GiftTab::Catalog { " active" } else { "" })}
                on:click=move |_| set_tab.set(GiftTab::Catalog)>"Catalog"</button>
            <button class={move || format!("main-tab{}", if tab.get() == GiftTab::Received { " active" } else { "" })}
                on:click=move |_| set_tab.set(GiftTab::Received)>"Received"</button>
            <button class={move || format!("main-tab{}", if tab.get() == GiftTab::Sent { " active" } else { "" })}
                on:click=move |_| set_tab.set(GiftTab::Sent)>"Sent"</button>
            <button class={move || format!("main-tab{}", if tab.get() == GiftTab::Nfts { " active" } else { "" })}
                on:click=move |_| set_tab.set(GiftTab::Nfts)>"NFTs"</button>
        </div>

        // Send modal
        {move || {
            if let Some(gift_id) = send_target.get() {
                let name = catalog.get().iter()
                    .find(|g| g.get("id").and_then(|v| v.as_i64()) == Some(gift_id))
                    .and_then(|g| g.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string().into())
                    .unwrap_or_else(|| "This gift".into());
                view! {
                    <div class="modal-overlay" on:click=move |_| set_send_target.set(None)>
                        <div class="modal" on:click=move |ev| ev.stop_propagation()>
                            <div class="modal-header">
                                <h2 class="modal-title">{format!("Send {name}")}</h2>
                                <button class="modal-close" on:click=move |_| set_send_target.set(None)>"\u{00d7}"</button>
                            </div>
                            <div style="padding:0 16px 16px;">
                                <div class="form-group" style="margin-bottom:16px;">
                                    <label class="form-label">"Recipient user ID"</label>
                                    <input class="form-input" type="text" placeholder="e.g. 2"
                                        prop:value=move || recipient.get()
                                        on:input=move |ev| set_recipient.set(event_target_value(&ev)) />
                                </div>
                                <button class="btn btn-primary" on:click=send_gift
                                    prop:disabled=move || busy.get()>
                                    {move || if busy.get() { "Sending..." } else { "Send gift" }}
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else { view! {}.into_any() }
        }}

        {move || {
            if loading.get() {
                view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else {
                match tab.get() {
                    GiftTab::Catalog => {
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
                                            <div class="gift-card">
                                                <div class="gift-icon">{icon}</div>
                                                <div class="gift-name">{name}</div>
                                                <div class="gift-price">{format!("{price} YSH")}</div>
                                                <span class={format!("badge badge-{}", rarity)}>{rarity.clone()}</span>
                                                <button class="btn btn-outline btn-sm" style="margin-top:10px;width:100%;"
                                                    on:click=move |_| set_send_target.set(Some(id))>
                                                    "Send"
                                                </button>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }
                    GiftTab::Received | GiftTab::Sent | GiftTab::Nfts => {
                        let list = match tab.get() {
                            GiftTab::Received => received.get(),
                            GiftTab::Sent => sent.get(),
                            GiftTab::Nfts => nfts.get(),
                            _ => Vec::new(),
                        };
                        if list.is_empty() {
                            view! {
                                <div class="empty-state">
                                    <div class="empty-state-title">"Nothing here yet"</div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="list">
                                    {list.into_iter().map(|item| {
                                        let name = item.get("gift_name").and_then(|v| v.as_str())
                                            .or_else(|| item.get("name").and_then(|v| v.as_str()))
                                            .unwrap_or("Gift").to_string();
                                        let from = item.get("from_user").and_then(|v| v.as_i64()).unwrap_or(0);
                                        let from_name = item.get("from_username").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let rarity = item.get("rarity").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let when = item.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let icon = gift_icons(&rarity).to_string();
                                        view! {
                                            <div class="list-item">
                                                <div class="gift-icon">{icon}</div>
                                                <div class="list-item-body">
                                                    <div class="list-item-title">{name}</div>
                                                    <div class="list-item-text">
                                                        {if tab.get() == GiftTab::Received {
                                                            format!("from user #{from}{}", if from_name.is_empty() { String::new() } else { format!(" ({from_name})") })
                                                        } else if tab.get() == GiftTab::Nfts {
                                                            format!("NFT \u{00B7} {rarity}")
                                                        } else {
                                                            format!("to user #{from}")
                                                        }}
                                                    </div>
                                                </div>
                                                <div class="list-item-time">{when}</div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }
                }
            }
        }}
    }
}
