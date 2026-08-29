# Phase 11: Leptos WASM Frontend — Auth Module

## Overview

Build a Client-Side Rendering (CSR) SPA frontend using Leptos 0.8 + Trunk + Tailwind CSS. This phase focuses on the **Auth module** (login, register, 2FA) as the entry point to the entire platform.

## Architecture

```
frontend/
├── Cargo.toml
├── Trunk.toml                  # Trunk build config
├── tailwind.config.js          # Tailwind CSS config
├── input.css                   # Tailwind entry point
├── index.html                  # HTML shell
├── src/
│   ├── main.rs                 # WASM entry point
│   ├── lib.rs                  # App root + Router
│   ├── api.rs                  # API client (fetch wrapper, auth headers)
│   ├── store.rs                # Global state (auth token, user info)
│   ├── components/
│   │   ├── mod.rs
│   │   ├── layout.rs           # App shell (nav, sidebar, footer)
│   │   ├── navbar.rs           # Top navigation bar
│   │   └── protected.rs        # Auth guard component
│   └── pages/
│       ├── mod.rs
│       ├── login.rs            # Login page
│       ├── register.rs         # Register page
│       ├── two_factor.rs       # 2FA verification page
│       ├── forgot_password.rs  # Recovery code login
│       └── dashboard.rs        # Post-login landing (stub)
└── public/
    └── favicon.svg
```

## Dependencies (Cargo.toml)

```toml
[package]
name = "ysh-frontend"
version = "0.1.0"
edition = "2021"

[dependencies]
leptos = { version = "0.8", features = [] }
leptos_router = { version = "0.8" }
leptos_meta = { version = "0.8" }
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
web-sys = { version = "0.3", features = ["Window", "LocalStorage", "Request", "RequestInit", "RequestMode", "Response", "Headers"] }
gloo-net = { version = "0.6", features = ["http"] }
gloo-storage = "0.3"
log = "0.4"
console_log = "1"
console_error_panic_hook = "0.1"

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

## API Client Design (`api.rs`)

```rust
// Core fetch wrapper that:
// - Adds JWT Bearer token from store
// - Returns typed JSON responses
// - Handles errors (401 → redirect to login, network errors)

pub struct ApiClient {
    base_url: String,  // e.g. "http://localhost:3000/api/v1"
}

impl ApiClient {
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError>;
    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T, ApiError>;
}
```

## State Management (`store.rs`)

```rust
// Signal-based reactive state
// - auth_token: Option<String> (stored in localStorage)
// - user: Option<UserInfo> (id, username, role)
// - is_logged_in: bool (derived signal)
```

## Routes

| Path | Component | Auth Required |
|------|-----------|---------------|
| `/` | Dashboard (stub) | Yes |
| `/login` | LoginPage | No |
| `/register` | RegisterPage | No |
| `/2fa` | TwoFactorPage | No (temp_token) |
| `/recovery` | ForgotPasswordPage | No |

## Auth Flow

```
1. User visits /login
2. POST /api/v1/login { username, password }
3. Response:
   - Without 2FA → { access_token, refresh_token } → save to localStorage → redirect /
   - With 2FA → { requires_2fa: true, temp_token } → redirect /2fa
4. User enters TOTP code on /2fa
5. POST /api/v1/login/2fa { temp_token, code }
6. Response → { access_token, refresh_token } → save → redirect /
```

## Page Designs

### Login Page
- Centered card layout
- Username + password fields
- "Login" button (primary)
- "Forgot password?" link → /recovery
- "Don't have an account? Register" link → /register
- Error toast for invalid credentials

### Register Page
- Centered card layout
- Username (3-32 chars), email, password (min 8 chars) fields
- Confirm password field
- "Register" button (primary)
- "Already have an account? Login" link → /login
- Success → redirect to /login with success message

### 2FA Page
- Centered card layout
- 6-digit code input (auto-submit on 6 digits)
- "Verify" button
- "Use recovery code" link → /recovery
- Error toast for invalid code

### Recovery Code Page
- Centered card layout
- Username + recovery code fields
- "Login" button
- Error toast for invalid code

## Styling (Tailwind)

- Dark theme by default (gray-900 bg, gray-50 text)
- Accent color: violet-600 (buttons, links)
- Card: rounded-2xl, shadow-xl, bg-white/5 backdrop-blur
- Inputs: rounded-lg, bg-gray-800, border-gray-700, focus:ring-violet-500
- Responsive: mobile-first, max-w-md for auth cards

## Build & Dev

```bash
# Install toolchain (one-time)
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
npm install -g tailwindcss

# Dev server
cd frontend
trunk serve --open
# Opens at http://127.0.0.1:8080

# Production build
trunk build --release
# Output in dist/
```

## Trunk.toml

```toml
[build]
target = "index.html"
watch = ["src", "index.html", "input.css"]

[serve]
address = "127.0.0.1"
port = 8080
```

## Implementation Order

1. **Toolchain setup** — install wasm32, trunk, tailwindcss
2. **Project scaffold** — Cargo.toml, Trunk.toml, index.html, tailwind config
3. **App shell** — main.rs, lib.rs, Router with routes
4. **API client** — api.rs with fetch wrapper
5. **State store** — store.rs with auth signals
6. **Layout components** — navbar, protected route guard
7. **Login page** — full auth flow with error handling
8. **Register page** — form validation, success redirect
9. **2FA page** — code input, verify flow
10. **Recovery page** — recovery code login
11. **Dashboard stub** — placeholder after login
12. **Test** — verify all flows work against running backend
