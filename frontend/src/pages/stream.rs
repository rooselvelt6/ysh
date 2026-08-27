use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use crate::store;

// Call states
#[derive(Clone, PartialEq)]
enum CallState {
    Idle,
    Ringing,
    Connecting,
    Connected,
    Ended,
}

// WebRTC objects are stored in thread-local storage so they remain accessible
// from async callbacks without forcing leptos' render closures to capture
// non-Send values (RtcPeerConnection / MediaStream are not Send).
thread_local! {
    static PEER_CONNECTION: RefCell<Option<web_sys::RtcPeerConnection>> = RefCell::new(None);
    static LOCAL_STREAM: RefCell<Option<web_sys::MediaStream>> = RefCell::new(None);
}

#[component]
pub fn StreamPage() -> impl IntoView {
    let (call_state, set_call_state) = signal(CallState::Idle);
    let (peer_id_input, set_peer_id_input) = signal(String::new());
    let (call_type, set_call_type) = signal("video".to_string());
    let (is_muted, set_is_muted) = signal(false);
    let (is_video_off, set_is_video_off) = signal(false);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (status_msg, set_status_msg) = signal("Enter a user ID to start a call".to_string());
    let (remote_video_url, set_remote_video_url) = signal(Option::<String>::None);
    let (local_video_url, set_local_video_url) = signal(Option::<String>::None);
    let (duration, set_duration) = signal(0u32);

    // Acquire local media
    let acquire_media = {
        let set_local_video_url = set_local_video_url.clone();
        let set_error_msg = set_error_msg.clone();
        move || {
            let window = web_sys::window().unwrap();
            let navigator = window.navigator();
            let media_devices = navigator.media_devices().unwrap();
            let constraints = web_sys::MediaStreamConstraints::new();
            constraints.set_audio(&JsValue::from_bool(true));
            constraints.set_video(&JsValue::from_bool(true));

            let local_video_url2 = set_local_video_url.clone();
            let set_error_msg2 = set_error_msg.clone();

            let success_cb = Closure::wrap(Box::new(move |stream: JsValue| {
                let stream: web_sys::MediaStream = stream.into();
                if let Ok(url) = web_sys::Url::create_object_url_with_blob(&stream.clone().unchecked_into::<web_sys::Blob>()) {
                    local_video_url2.set(Some(url));
                }
                LOCAL_STREAM.with(|s| *s.borrow_mut() = Some(stream));
            }) as Box<dyn FnMut(JsValue)>);

            let error_cb = Closure::wrap(Box::new(move |err: JsValue| {
                set_error_msg2.set(Some(format!("Media error: {:?}", err)));
            }) as Box<dyn FnMut(JsValue)>);

            let _ = media_devices.get_user_media_with_constraints(&constraints)
                .map(|p| {
                    let _ = p.then2(&success_cb, &error_cb);
                });
            success_cb.forget();
            error_cb.forget();
        }
    };

    // Acquire media on mount
    let acquired = std::rc::Rc::new(std::cell::Cell::new(false));
    if !acquired.get() {
        acquired.set(true);
        acquire_media();
    }

    // Start call
    let start_call = {
        let set_call_state = set_call_state.clone();
        let peer_id_input = peer_id_input.clone();
        let call_type = call_type.clone();
        let set_error_msg = set_error_msg.clone();
        let set_status_msg = set_status_msg.clone();
        let set_remote_video_url = set_remote_video_url.clone();
        move |_: leptos::ev::MouseEvent| {
            let target_id: i64 = match peer_id_input.get().trim().parse() {
                Ok(id) => id,
                Err(_) => {
                    set_error_msg.set(Some("Enter a valid user ID".into()));
                    return;
                }
            };
            set_error_msg.set(None);
            set_call_state.set(CallState::Connecting);
            set_status_msg.set(format!("Calling user {}...", target_id));

            // Get JWT token for WS
            let token = store::with_token(|t| t.unwrap_or("").to_string());
            if token.is_empty() {
                set_error_msg.set(Some("Not logged in".into()));
                set_call_state.set(CallState::Idle);
                return;
            }

            // Create RTCPeerConnection with public STUN servers
            let rtc_config = web_sys::RtcConfiguration::new();
            let ice_server = web_sys::RtcIceServer::new();
            ice_server.set_urls_str("stun:stun.l.google.com:19302");
            let ice_servers = js_sys::Array::of1(&ice_server);
            rtc_config.set_ice_servers(&ice_servers);
            let pc = web_sys::RtcPeerConnection::new_with_configuration(&rtc_config).unwrap();

            // Add local tracks
            if let Some(ref stream) = LOCAL_STREAM.with(|s| s.borrow().clone()) {
                let tracks = stream.get_tracks();
                for i in 0..tracks.length() {
                    if let Some(track) = tracks.get(i).dyn_ref::<web_sys::MediaStreamTrack>() {
                        let _ = pc.add_track(track, stream, &js_sys::Array::new());
                    }
                }
            }

            // Handle onicecandidate
            let ice_cb = Closure::wrap(Box::new(move |ev: web_sys::RtcPeerConnectionIceEvent| {
                if let Some(candidate) = ev.candidate() {
                    let candidate_str = candidate.candidate();
                    let sdp_mid = candidate.sdp_mid().unwrap_or_default();
                    let sdp_m_line_index = candidate.sdp_m_line_index().unwrap_or(0);
                    let msg = serde_json::json!({
                        "type": "ice_candidate",
                        "peer_id": target_id,
                        "candidate": candidate_str,
                        "sdp_mid": sdp_mid,
                        "sdp_m_line_index": sdp_m_line_index
                    });
                    crate::api::ws_signaling_send(&msg.to_string());
                }
            }) as Box<dyn FnMut(_)>);
            pc.set_onicecandidate(Some(ice_cb.as_ref().unchecked_ref()));
            ice_cb.forget();

            // Handle ontrack
            let track_cb = Closure::wrap(Box::new(move |ev: web_sys::RtcTrackEvent| {
                let stream_val = ev.streams().get(0);
                if let Ok(stream) = stream_val.dyn_into::<web_sys::MediaStream>() {
                    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&stream.unchecked_into::<web_sys::Blob>()) {
                        set_remote_video_url.set(Some(url));
                    }
                }
            }) as Box<dyn FnMut(_)>);
            pc.set_ontrack(Some(track_cb.as_ref().unchecked_ref()));
            track_cb.forget();

            PEER_CONNECTION.with(|p| *p.borrow_mut() = Some(pc));

            // Connect WebSocket and send call_invite
            let call_type_str = call_type.get();
            let msg = serde_json::json!({
                "type": "call_invite",
                "target_user_id": target_id,
                "call_type": call_type_str
            });
            crate::api::ws_signaling_send(&msg.to_string());
        }
    };

    // Hangup
    let hangup = {
        let set_call_state = set_call_state.clone();
        let set_status_msg = set_status_msg.clone();
        let set_remote_video_url = set_remote_video_url.clone();
        move |_: leptos::ev::MouseEvent| {
            // Close peer connection
            if let Some(pc) = PEER_CONNECTION.with(|p| p.borrow_mut().take()) {
                let _ = pc.close();
            }
            // Stop local tracks
            if let Some(ref stream) = LOCAL_STREAM.with(|s| s.borrow().clone()) {
                let tracks = stream.get_tracks();
                for i in 0..tracks.length() {
                    if let Some(track) = tracks.get(i).dyn_ref::<web_sys::MediaStreamTrack>() {
                        track.stop();
                    }
                }
            }
            set_call_state.set(CallState::Idle);
            set_status_msg.set("Call ended".into());
            set_remote_video_url.set(None);
            set_duration.set(0);

            // Send hangup
            let msg = r#"{"type":"call_hangup","peer_id":0}"#;
            crate::api::ws_signaling_send(msg);
        }
    };

    // Toggle mute
    let toggle_mute = {
        let set_is_muted = set_is_muted.clone();
        move |_: leptos::ev::MouseEvent| {
            let new_muted = !is_muted.get();
            set_is_muted.set(new_muted);
            if let Some(ref stream) = LOCAL_STREAM.with(|s| s.borrow().clone()) {
                let audio_tracks = stream.get_audio_tracks();
                for i in 0..audio_tracks.length() {
                    if let Some(track) = audio_tracks.get(i).dyn_ref::<web_sys::MediaStreamTrack>() {
                        let _ = track.set_enabled(!new_muted);
                    }
                }
            }
        }
    };

    // Toggle video
    let toggle_video = {
        let set_is_video_off = set_is_video_off.clone();
        move |_: leptos::ev::MouseEvent| {
            let new_off = !is_video_off.get();
            set_is_video_off.set(new_off);
            if let Some(ref stream) = LOCAL_STREAM.with(|s| s.borrow().clone()) {
                let video_tracks = stream.get_video_tracks();
                for i in 0..video_tracks.length() {
                    if let Some(track) = video_tracks.get(i).dyn_ref::<web_sys::MediaStreamTrack>() {
                        let _ = track.set_enabled(!new_off);
                    }
                }
            }
        }
    };

    view! {
        <div class="main-header">
            <h2>"Live Call"</h2>
        </div>

        {move || error_msg.get().map(|e| view! { <div class="alert alert-error" style="margin:8px 16px;">{e}</div> })}

        <div class="stream-video" style="position:relative;background:#0a0a0a;">
            {move || {
                if let Some(ref url) = remote_video_url.get() {
                    view! {
                        <video
                            src={url.clone()}
                            autoplay="true"
                            playsinline="true"
                            style="width:100%;height:100%;object-fit:cover;"
                        ></video>
                    }.into_any()
                } else if let Some(ref url) = local_video_url.get() {
                    view! {
                        <video
                            src={url.clone()}
                            autoplay="true"
                            playsinline="true"
                            muted="true"
                            style="width:100%;height:100%;object-fit:cover;transform:scaleX(-1);"
                        ></video>
                    }.into_any()
                } else {
                    view! { <div style="color:var(--text-dim);font-size:1.1rem;">"Camera preview"</div> }.into_any()
                }
            }}
            // PiP local video when in call
            {move || {
                if call_state.get() == CallState::Connected {
                    if let Some(ref url) = local_video_url.get() {
                        view! {
                            <div style="position:absolute;bottom:16px;right:16px;width:160px;height:120px;border-radius:12px;overflow:hidden;border:2px solid var(--border);background:#000;">
                                <video
                                    src={url.clone()}
                                    autoplay="true"
                                    playsinline="true"
                                    muted="true"
                                    style="width:100%;height:100%;object-fit:cover;transform:scaleX(-1);"
                                ></video>
                            </div>
                        }.into_any()
                    } else { view! {}.into_any() }
                } else { view! {}.into_any() }
            }}
            // Duration overlay
            {move || {
                if call_state.get() == CallState::Connected {
                    let d = duration.get();
                    let mins = d / 60;
                    let secs = d % 60;
                    view! {
                        <div style="position:absolute;top:16px;left:16px;background:rgba(0,0,0,0.7);padding:4px 12px;border-radius:9999px;font-size:0.875rem;font-weight:600;color:#00ba7c;">
                            {format!("{:02}:{:02}", mins, secs)}
                        </div>
                    }.into_any()
                } else { view! {}.into_any() }
            }}
            // Status text
            <div style="position:absolute;bottom:16px;left:50%;transform:translateX(-50%);color:var(--text-dim);font-size:0.9rem;">
                {move || status_msg.get()}
            </div>
        </div>

        // Controls
        {move || {
            match call_state.get() {
                CallState::Idle => view! {
                    <div style="padding:16px;">
                        <div style="display:flex;gap:8px;margin-bottom:12px;">
                            <input class="form-input" type="text" placeholder="User ID to call"
                                prop:value=move || peer_id_input.get()
                                on:input=move |ev| set_peer_id_input.set(event_target_value(&ev))
                                style="flex:1;" />
                            <select class="form-input" style="width:120px;"
                                on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    set_call_type.set(v);
                                }>
                                <option value="video">"Video"</option>
                                <option value="audio">"Audio"</option>
                            </select>
                        </div>
                        <div style="display:flex;gap:8px;">
                            <button class="btn btn-primary" style="flex:1;" on:click=start_call>
                                "\u{1F4F9} Start Call"
                            </button>
                        </div>
                    </div>
                }.into_any(),

                CallState::Connecting | CallState::Ringing => view! {
                    <div class="stream-controls">
                        <button class="btn btn-danger btn-sm" on:click=hangup>"Cancel"</button>
                    </div>
                }.into_any(),

                CallState::Connected => view! {
                    <div class="stream-controls" style="display:flex;gap:12px;justify-content:center;padding:16px;">
                        <button class="btn btn-outline btn-sm" on:click=toggle_mute>
                            {move || if is_muted.get() { "Unmute" } else { "Mute" }}
                        </button>
                        <button class="btn btn-outline btn-sm" on:click=toggle_video>
                            {move || if is_video_off.get() { "Cam On" } else { "Cam Off" }}
                        </button>
                        <button class="btn btn-danger btn-sm" on:click=hangup>"End Call"</button>
                    </div>
                }.into_any(),

                CallState::Ended => view! {
                    <div style="padding:16px;text-align:center;">
                        <button class="btn btn-primary" on:click=move |_| set_call_state.set(CallState::Idle)>
                            "Back"
                        </button>
                    </div>
                }.into_any(),
            }
        }}
    }
}
