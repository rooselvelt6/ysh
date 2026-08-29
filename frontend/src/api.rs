use gloo_net::http::Request;
use serde::de::DeserializeOwned;

use crate::store::with_token;

#[derive(Debug, Clone)]
pub enum ApiError {
    Network(String),
    Server { status: u16, message: String },
    Deserialize(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(e) => write!(f, "Network error: {e}"),
            ApiError::Server { status, message } => {
                write!(f, "Server {status}: {message}")
            }
            ApiError::Deserialize(e) => write!(f, "Response error: {e}"),
        }
    }
}

fn base_url() -> String {
    let win = web_sys::window().expect("no window");
    let loc = win.location();
    // Check for YSH_API_URL env var (set via window.__YSH_API__)
    let env_api = js_sys::eval("window.__YSH_API__ || ''")
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    if !env_api.is_empty() {
        return format!("{env_api}/api/v1");
    }
    let protocol = loc.protocol().unwrap_or_default();
    let host = loc.host().unwrap_or_default();
    format!("{protocol}//{host}/api/v1")
}

fn auth_header() -> Option<String> {
    with_token(|t| t.map(|s| format!("Bearer {s}")))
}

fn handle_unauthorized() {
    crate::store::clear_auth();
    if let Some(win) = web_sys::window() {
        let _ = win.location().set_href("/login");
    }
}

fn is_unauthorized(status: u16) -> bool {
    status == 401 && with_token(|t| t.is_some())
}

pub async fn post<T: DeserializeOwned>(path: &str, body: &impl serde::Serialize) -> Result<T, ApiError> {
    let url = format!("{}{path}", base_url());
    let json = serde_json::to_string(body)
        .map_err(|e| ApiError::Deserialize(format!("Serialize error: {e}")))?;

    let mut builder = Request::post(&url).header("Content-Type", "application/json");
    if let Some(token) = auth_header() {
        builder = builder.header("Authorization", &token);
    }

    let resp = builder
        .body(json)
        .map_err(|e| ApiError::Network(format!("Failed to create request: {e:?}")))?;

    let resp = resp
        .send()
        .await
        .map_err(|e| ApiError::Network(format!("{e:?}")))?;

    if resp.ok() {
        resp.json::<T>()
            .await
            .map_err(|e| ApiError::Deserialize(format!("{e:?}")))
    } else {
        let status = resp.status();
        let msg = resp.text().await.unwrap_or_default();
        if is_unauthorized(status) {
            handle_unauthorized();
        }
        Err(ApiError::Server { status, message: msg })
    }
}

pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    let url = format!("{}{path}", base_url());

    let mut builder = Request::get(&url);
    if let Some(token) = auth_header() {
        builder = builder.header("Authorization", &token);
    }

    let resp = builder
        .send()
        .await
        .map_err(|e| ApiError::Network(format!("{e:?}")))?;

    if resp.ok() {
        resp.json::<T>()
            .await
            .map_err(|e| ApiError::Deserialize(format!("{e:?}")))
    } else {
        let status = resp.status();
        let msg = resp.text().await.unwrap_or_default();
        if is_unauthorized(status) {
            handle_unauthorized();
        }
        Err(ApiError::Server { status, message: msg })
    }
}

pub fn go(path: &str) {
    let nav = leptos_router::hooks::use_navigate();
    nav(path, leptos_router::NavigateOptions::default());
}

use std::cell::RefCell;

thread_local! {
    static WS_SEND: RefCell<Option<Box<dyn FnMut(String)>>> = RefCell::new(None);
}

pub fn ws_signaling_send(msg: &str) {
    WS_SEND.with(|s| {
        if let Some(ref mut send_fn) = *s.borrow_mut() {
            send_fn(msg.to_string());
        }
    });
}

pub fn ws_set_sender(sender: Box<dyn FnMut(String)>) {
    WS_SEND.with(|s| *s.borrow_mut() = Some(sender));
}

pub mod ws_signaling {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use std::cell::RefCell;
    use std::rc::Rc;

    pub type WsCallback = Box<dyn FnMut(String)>;

    pub struct WsClient {
        ws: Option<web_sys::WebSocket>,
        on_message: Rc<RefCell<Option<WsCallback>>>,
    }

    impl WsClient {
        pub fn new() -> Self {
            Self { ws: None, on_message: Rc::new(RefCell::new(None)) }
        }

        pub fn connect(&mut self, token: &str) -> Result<(), JsValue> {
            let win = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
            let loc = win.location();
            let protocol = if loc.protocol().unwrap_or_default() == "https:" { "wss:" } else { "ws:" };
            let host = loc.host().unwrap_or_default();
            let url = format!("{}//{}/ws?token={}", protocol, host, token);

            let ws = web_sys::WebSocket::new(&url)?;
            ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

            let on_message = self.on_message.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
                if let Some(data) = event.data().as_string() {
                    if let Some(ref mut cb) = *on_message.borrow_mut() {
                        cb(data);
                    }
                }
            }) as Box<dyn FnMut(_)>);
            ws.set_onmessage(Some(closure.as_ref().unchecked_ref()));
            closure.forget();

            self.ws = Some(ws);
            Ok(())
        }

        pub fn send(&self, msg: &str) {
            if let Some(ref ws) = self.ws {
                let _ = ws.send_with_str(msg);
            }
        }

        pub fn set_on_message(&self, cb: WsCallback) {
            *self.on_message.borrow_mut() = Some(cb);
        }

        pub fn close(&mut self) {
            if let Some(ref ws) = self.ws {
                let _ = ws.close();
            }
            self.ws = None;
        }
    }

    impl Drop for WsClient {
        fn drop(&mut self) {
            self.close();
        }
    }
}
