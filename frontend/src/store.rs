use std::cell::RefCell;
use gloo_storage::{LocalStorage, Storage};

const TOKEN_KEY: &str = "ysh_access_token";
const REFRESH_KEY: &str = "ysh_refresh_token";
const USER_KEY: &str = "ysh_user_info";
const THEME_KEY: &str = "ysh_theme";

thread_local! {
    static AUTH_TOKEN: RefCell<Option<String>> = const { RefCell::new(None) };
    static REFRESH_TOKEN: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserInfo {
    pub user_id: i64,
    pub role: String,
    pub username: String,
}

pub fn init_auth() {
    AUTH_TOKEN.with(|t| {
        if t.borrow().is_none() {
            if let Ok(token) = LocalStorage::get::<String>(TOKEN_KEY) {
                *t.borrow_mut() = Some(token);
            }
        }
    });
    REFRESH_TOKEN.with(|t| {
        if t.borrow().is_none() {
            if let Ok(token) = LocalStorage::get::<String>(REFRESH_KEY) {
                *t.borrow_mut() = Some(token);
            }
        }
    });
}

pub fn save_tokens(access: &str, refresh: &str) {
    AUTH_TOKEN.with(|t| *t.borrow_mut() = Some(access.to_string()));
    REFRESH_TOKEN.with(|t| *t.borrow_mut() = Some(refresh.to_string()));
    let _ = LocalStorage::set(TOKEN_KEY, access);
    let _ = LocalStorage::set(REFRESH_KEY, refresh);
}

pub fn save_user(user: &UserInfo) {
    let _ = LocalStorage::set(USER_KEY, user);
}

pub fn get_user() -> Option<UserInfo> {
    LocalStorage::get(USER_KEY).ok()
}

pub fn clear_auth() {
    AUTH_TOKEN.with(|t| *t.borrow_mut() = None);
    REFRESH_TOKEN.with(|t| *t.borrow_mut() = None);
    LocalStorage::delete(TOKEN_KEY);
    LocalStorage::delete(REFRESH_KEY);
    LocalStorage::delete(USER_KEY);
}

pub fn is_logged_in() -> bool {
    AUTH_TOKEN.with(|t| t.borrow().is_some())
}

pub fn with_token<F, R>(f: F) -> R
where
    F: FnOnce(Option<&str>) -> R,
{
    AUTH_TOKEN.with(|t| {
        let borrow = t.borrow();
        f(borrow.as_deref())
    })
}

pub fn init_theme() -> bool {
    let dark = LocalStorage::get::<bool>(THEME_KEY).unwrap_or(true);
    apply_theme(dark);
    dark
}

pub fn toggle_theme() -> bool {
    let current = LocalStorage::get::<bool>(THEME_KEY).unwrap_or(true);
    let new = !current;
    let _ = LocalStorage::set(THEME_KEY, new);
    apply_theme(new);
    new
}

pub fn is_dark() -> bool {
    LocalStorage::get::<bool>(THEME_KEY).unwrap_or(true)
}

fn apply_theme(dark: bool) {
    if let Some(win) = web_sys::window() {
        if let Some(doc) = win.document() {
            if let Some(el) = doc.document_element() {
                if dark {
                    let _ = el.remove_attribute("data-theme");
                } else {
                    let _ = el.set_attribute("data-theme", "light");
                }
            }
        }
    }
}
