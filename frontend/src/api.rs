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
        Err(ApiError::Server { status, message: msg })
    }
}

pub fn go(path: &str) {
    let nav = leptos_router::hooks::use_navigate();
    nav(path, leptos_router::NavigateOptions::default());
}
