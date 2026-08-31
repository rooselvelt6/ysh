# Frontend — Routing

## Overview

SPA (Single Page Application) construida con Leptos + WASM. Router con rutas protegidas, auth guards, y shell de navegación.

**Framework:** Leptos v0.7+ con `leptos_router`
**HTTP Client:** `gloo-net`
**Storage:** `gloo-storage` (LocalStorage)

---

## Routes

### Public Routes (sin auth)

| Route | Componente | Descripción |
|-------|-----------|------------|
| `/login` | `LoginPage` | Formulario de login |
| `/register` | `RegisterPage` | Formulario de registro |
| `/2fa` | `TwoFactorPage` | Verificación 2FA |
| `/recovery` | `ForgotPasswordPage` | Recuperación con recovery code |

### Protected Routes (auth requerida)

| Route | Componente | Descripción |
|-------|-----------|------------|
| `/` | `DashboardPage` | Dashboard principal |
| `/discover` | `DiscoverPage` | Descubrir hosts |
| `/wallet` | `WalletPage` | Billetera |
| `/profile` | `ProfilePage` | Mi perfil |
| `/moments` | `MomentsPage` | Feed de posts |
| `/gifts` | `GiftsPage` | Catálogo de regalos |
| `/hosts` | `HostsPage` | Lista de hosts |
| `/agency` | `AgencyPage` | Mi agencia |
| `/chat` | `ChatPage` | Mensajes |
| `/notifications` | `NotificationsPage` | Notificaciones |
| `/stream` | `StreamPage` | WebRTC streaming |
| `/admin` | `AdminPage` | Panel admin (role=admin) |

---

## Route Structure

```
<Router>
  <Routes>
    // Public
    <Route path="/login" view=LoginPage />
    <Route path="/register" view=RegisterPage />
    <Route path="/2fa" view=TwoFactorPage />
    <Route path="/recovery" view=ForgotPasswordPage />

    // Protected (wrapped in ProtectedRoute)
    <ParentRoute path="/" view=AppShell>
      <Route path="/" view=DashboardPage />
      <Route path="/discover" view=DiscoverPage />
      <Route path="/wallet" view=WalletPage />
      <Route path="/profile" view=ProfilePage />
      <Route path="/moments" view=MomentsPage />
      <Route path="/gifts" view=GiftsPage />
      <Route path="/hosts" view=HostsPage />
      <Route path="/agency" view=AgencyPage />
      <Route path="/chat" view=ChatPage />
      <Route path="/notifications" view=NotificationsPage />
      <Route path="/stream" view=StreamPage />
      <Route path="/admin" view=AdminPage />
    </ParentRoute>
  </Routes>
</Router>
```

---

## ProtectedRoute Component

**Archivo:** `components/protected.rs`

```rust
// Lógica:
if store::is_logged_in() {
    <Outlet />  // renderiza la ruta
} else {
    <Redirect path="/login" />  // redirige a login
}
```

---

## Auth Flow

### Login

```
1. POST /login { username, password }
2. Si requires_2fa:
   a. Guardar temp_token en localStorage
   b. Redirect a /2fa
   c. POST /login/2fa { temp_token, code }
3. Si no:
   a. Guardar access_token + refresh_token en localStorage
   b. GET /me → guardar UserInfo en localStorage
   c. Redirect a /
```

### Register

```
1. POST /register { username, email, password }
2. Redirect a /login
```

### Token Refresh

```
1. En cada 401 response:
   a. POST /refresh { refresh_token }
   b. Si exitoso → retry request original
   c. Si falla → clear auth → redirect /login
```

### Logout

```
1. store::clear_auth() → elimina localStorage + thread-local
2. api::go("/login") → hard navigation
```

---

## Navigation

### Sidebar (Desktop)

Navegación izquierda con todos los links. Muestra "Admin" solo si `role == "admin"`.

### BottomNav (Mobile)

Bottom bar con: Home, Search, Alerts, Chat, Profile.

### api::go()

Hard navigation via `window.location.href`. No usa `use_navigate` (evita panic fuera de contexto Router).

---

## Store (State Management)

**Archivo:** `store.rs`

```rust
// Thread-local tokens
static ACCESS_TOKEN: RefCell<Option<String>>
static REFRESH_TOKEN: RefCell<Option<String>>
static USER_INFO: RefCell<Option<UserInfo>>

// LocalStorage keys
ysh_access_token
ysh_refresh_token
ysh_user_info
ysh_theme
```

### UserInfo
```rust
struct UserInfo {
    user_id: i64,
    role: String,
    username: String,
}
```

---

## API Client

**Archivo:** `api.rs`

```rust
fn base_url() -> String  // window.__YSH_API__ o current origin

async fn send<T>(method, path, body) -> Result<T, ApiError>
  // Auto-refresh en 401
  // Bearer token en header

fn post<T>(path, body) -> Result<T, ApiError>
fn get<T>(path) -> Result<T, ApiError>
fn del<T>(path) -> Result<T, ApiError>
fn go(path)  // window.location.href = path
```

### ApiError

```rust
enum ApiError {
    Network,
    Server { status: u16, message: String },
    Deserialize,
}
```

---

## WebSocket Client

**Archivo:** `api.rs:213`

```rust
struct WsClient {
    ws: WebSocket,
    send: Closure<dyn FnMut(...)>,
}
```

- Conecta a `ws(s)://{host}/ws?token={jwt}`
- Binary type: `Arraybuffer`
- Usado para WebRTC signaling en `/stream`
- Thread-local `WS_SEND` refcell

---

## Theme

- Dark/light toggle via `data-theme` attribute en `<html>`
- Persistido en LocalStorage (`ysh_theme`)
- Toggle desde sidebar
