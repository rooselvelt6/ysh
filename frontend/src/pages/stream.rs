use leptos::prelude::*;

#[component]
pub fn StreamPage() -> impl IntoView {
    let (chat_msg, set_chat_msg) = signal(String::new());
    let (chat_msgs, set_chat_msgs) = signal(Vec::<String>::new());

    let send_msg = move |_: leptos::ev::MouseEvent| {
        let msg = chat_msg.get();
        if !msg.is_empty() {
            set_chat_msgs.update(|v| v.push(msg));
            set_chat_msg.set(String::new());
        }
    };

    view! {
        <div class="main-header">
            <h2>"Live Stream"</h2>
        </div>
        <div class="stream-video">
            "\u{1F4FA} Stream will appear here (WebRTC)"
        </div>
        <div class="stream-controls">
            <button class="btn btn-outline btn-sm">"\u{1F3A4} Mic"</button>
            <button class="btn btn-outline btn-sm">"\u{1F3A5} Cam"</button>
            <button class="btn btn-outline btn-sm">"\u{1F50A} Sound"</button>
            <button class="btn btn-danger btn-sm">"End"</button>
        </div>
        <div class="stream-chat">
            <div style="font-weight:700;padding-bottom:12px;border-bottom:1px solid var(--border);">"Live Chat"</div>
            {move || {
                let msgs = chat_msgs.get();
                if msgs.is_empty() {
                    view! { <div style="color:var(--text-dim);font-size:0.875rem;padding:16px 0;">"No messages yet"</div> }.into_any()
                } else {
                    view! {
                        {msgs.into_iter().map(|m| {
                            view! { <div style="padding:6px 0;font-size:0.9375rem;">{m}</div> }
                        }).collect_view()}
                    }.into_any()
                }
            }}
            <div class="stream-chat-input">
                <input class="form-input" type="text" placeholder="Type a message..."
                    prop:value=move || chat_msg.get()
                    on:input=move |ev| set_chat_msg.set(event_target_value(&ev))
                    on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                        if ev.key() == "Enter" { send_msg(leptos::ev::MouseEvent::new("click").unwrap()); }
                    } />
                <button class="btn btn-primary btn-sm" style="width:auto;" on:click=send_msg>"Send"</button>
            </div>
        </div>
    }
}
