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

/// Intenta renovar el access token usando el refresh token. Devuelve true si
/// se obtuvo un token nuevo; false si no hay sesion valida para renovar.
async fn try_refresh_token() -> bool {
    use crate::store::with_refresh_token;
    let Some(refresh) = with_refresh_token(|t| t.map(|s| s.to_string())) else {
        return false;
    };
    let url = format!("{}/refresh", base_url());
    let body = serde_json::json!({ "refresh_token": refresh });
    let json = serde_json::to_string(&body).unwrap_or_default();
    let resp = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(json)
        .map_err(|e| ApiError::Network(format!("Failed to create request: {e:?}")))
        .ok();
    let resp = match resp {
        Some(r) => match r.send().await {
            Ok(r) => r,
            Err(_) => return false,
        },
        None => return false,
    };
    if !resp.ok() {
        return false;
    }
    let val: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return false,
    };
    if let Some(at) = val.get("access_token").and_then(|v| v.as_str()) {
        crate::store::set_access_token(at);
        true
    } else {
        false
    }
}

async fn send(
    method: gloo_net::http::Method,
    path: &str,
    body: Option<&str>,
) -> Result<gloo_net::http::Response, ApiError> {
    let url = format!("{}{path}", base_url());

    let method_c = method.clone();
    let body_c = body.map(|s| s.to_string());

    let do_request = |url: String, token: Option<&str>| -> Result<gloo_net::http::Request, ApiError> {
        let mut builder = match method_c {
            gloo_net::http::Method::GET => Request::get(&url),
            gloo_net::http::Method::DELETE => Request::delete(&url),
            _ => Request::post(&url),
        };
        if let Some(token) = token {
            builder = builder.header("Authorization", token);
        }
        match &body_c {
            // Solo adjuntar body a metodos que lo permiten. Un GET/DELETE con
            // body (aunque sea vacio) lanza un TypeError en el navegador:
            // "Request with GET/HEAD method cannot have body".
            Some(json) => builder
                .header("Content-Type", "application/json")
                .body(json.clone())
                .map_err(|e| ApiError::Network(format!("Body error: {e:?}"))),
            None => builder
                .build()
                .map_err(|e| ApiError::Network(format!("Body error: {e:?}"))),
        }
    };

    let token_for_send = auth_header().map(|s| s.clone());
    let req = do_request(url.clone(), token_for_send.as_deref())?;
    let resp = req
        .send()
        .await
        .map_err(|e| ApiError::Network(format!("{e:?}")))?;

    if resp.status() == 401 && with_token(|t| t.is_some()) {
        if try_refresh_token().await {
            let new_token_for_send = auth_header().map(|s| s.clone());
            let req = do_request(url.clone(), new_token_for_send.as_deref())?;
            let resp = req
                .send()
                .await
                .map_err(|e| ApiError::Network(format!("{e:?}")))?;
            return Ok(resp);
        } else {
            handle_unauthorized();
        }
    }
    Ok(resp)
}

pub async fn post<T: DeserializeOwned>(path: &str, body: &impl serde::Serialize) -> Result<T, ApiError> {
    let json = serde_json::to_string(body)
        .map_err(|e| ApiError::Deserialize(format!("Serialize error: {e}")))?;
    let resp = send(gloo_net::http::Method::POST, path, Some(&json)).await?;

    if resp.ok() {
        resp.json::<T>()
            .await
            .map_err(|e| ApiError::Deserialize(format!("{e:?}")))
    } else {
        let status = resp.status();
        let msg = resp.text().await.unwrap_or_default();
        Err(ApiError::Server { status, message: msg })
    }
}

pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    let resp = send(gloo_net::http::Method::GET, path, None).await?;

    if resp.ok() {
        resp.json::<T>()
            .await
            .map_err(|e| ApiError::Deserialize(format!("{e:?}")))
    } else {
        let status = resp.status();
        let msg = resp.text().await.unwrap_or_default();
        Err(ApiError::Server { status, message: msg })
    }
}

pub fn go(path: &str) {
    if let Some(win) = web_sys::window() {
        let _ = win.location().set_href(path);
    }
}

pub async fn del<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    let resp = send(gloo_net::http::Method::DELETE, path, None).await?;

    if resp.ok() {
        resp.json::<T>()
            .await
            .map_err(|e| ApiError::Deserialize(format!("{e:?}")))
    } else {
        let status = resp.status();
        let msg = resp.text().await.unwrap_or_default();
        Err(ApiError::Server { status, message: msg })
    }
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
