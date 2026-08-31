# Frontend — Componentes

## Overview

Componentes Leptos del frontend YSH organizados por categoría.

---

## Layout Components

### AppShell (`pages/layout_page.rs`)

Shell principal con Sidebar + RightSidebar + BottomNav + contenido.

### Sidebar (`components/layout/sidebar.rs`)

Desktop left navigation:
- User profile card
- Links a todas las rutas
- Link "Admin" visible solo con `role == "admin"`
- Logout button

### RightSidebar (`components/layout/right_sidebar.rs`)

Panel derecho estático: trending, search, follow widget (datos hardcodeados).

### BottomNav (`components/layout/bottom_nav.rs`)

Mobile bottom navigation:
- Home, Search, Alerts, Chat, Profile

---

## UI Primitives (`components/ui/`)

| Componente | Archivo | Props | Descripción |
|-----------|---------|-------|------------|
| `Avatar` | `avatar.rs` | `src`, `size` | Imagen de perfil |
| `Badge` | `badge.rs` | `variant`, `children` | Badge/label |
| `Empty` | `empty.rs` | `message` | Estado vacío |
| `Loading` | `loading.rs` | `size` | Spinner |
| `Modal` | `modal.rs` | `open`, `on_close`, `children` | Modal overlay |
| `Switch` | `switch_.rs` | `checked`, `on_change` | Toggle switch |
| `Tabs` | `tabs.rs` | `tabs`, `active` | Tab navigation |
| `Toast` | `toast.rs` | `message`, `variant` | Notificación flotante |

---

## Feature Components

### ProtectedRoute (`components/protected.rs`)

Auth guard — redirect a `/login` si no autenticado, renderiza `<Outlet/>` si sí.

### GiftEffectOverlay (`components/gift_effect.rs`)

Animated particle overlay para envío de regalos:
- 6 variantes visuales (según rareza del gift)
- Se muestra al enviar/previsualizar un gift

### ToastContainer (`components/ui/toast.rs`)

Global toast notification system:
- Variantes: success, error, info, warning
- Auto-dismiss: 4 segundos
- Context: `ToastCtx`

---

## Pages

| Página | Componente | Auth | Descripción |
|--------|-----------|------|------------|
| Login | `LoginPage` | No | Form username + password, redirect a /2fa si 2FA |
| Register | `RegisterPage` | No | Form username + email + password |
| 2FA | `TwoFactorPage` | No | Input 6 dígitos TOTP |
| Recovery | `ForgotPasswordPage` | No | Username + recovery code |
| Dashboard | `DashboardPage` | Yes | Feed principal |
| Discover | `DiscoverPage` | Yes | Explorar hosts disponibles |
| Wallet | `WalletPage` | Yes | Balance, deposit, withdraw, transfer |
| Profile | `ProfilePage` | Yes | Editar perfil |
| Moments | `MomentsPage` | Yes | Crear y ver posts |
| Gifts | `GiftsPage` | Yes | Catálogo y envío de regalos |
| Hosts | `HostsPage` | Yes | Lista de hosts |
| Agency | `AgencyPage` | Yes | Gestión de agencia |
| Chat | `ChatPage` | Yes | Mensajes directos |
| Notifications | `NotificationsPage` | Yes | Centro de notificaciones |
| Stream | `StreamPage` | Yes | WebRTC videollamadas |
| Admin | `AdminPage` | Yes (admin) | Panel de administración |

---

## State Management

### Thread-Local (runtime)

```rust
// Tokens
ACCESS_TOKEN: RefCell<Option<String>>
REFRESH_TOKEN: RefCell<Option<String>>

// User info
USER_INFO: RefCell<Option<UserInfo>>

// WebSocket sender
WS_SEND: RefCell<Option<Closure>>
```

### LocalStorage (persistencia)

```
ysh_access_token → JWT access token
ysh_refresh_token → JWT refresh token
ysh_user_info → JSON UserInfo
ysh_theme → "dark" | "light"
```

### Init Flow

```
window.onload → init_auth()
  → lee LocalStorage
  → hidrata thread-local state
  → si tokens válidos → render app protegido
  → si no → render login
```

---

## WebSocket Integration

- Conexión a `/ws?token={jwt}`
- Usado exclusivamente para WebRTC signaling en `/stream`
- Mensajes: `call_invite`, `ice_candidate`, `call_hangup`
- Thread-local `WS_SEND` para enviar mensajes

---

## Theme System

- Toggle dark/light via `data-theme` en `<html>`
- Persistido en LocalStorage
- CSS variables para colores
- Glass morphism + Aurora design theme

---

## Error Handling

- **ApiError::Network** → toast "Error de conexión"
- **ApiError::Server { 401 }** → auto-refresh → retry
- **ApiError::Server { 403 }** → toast "Acceso denegado"
- **ApiError::Server { 404 }** → toast "No encontrado"
- **ApiError::Server { 429 }** → toast "Demasiadas peticiones"
- **ApiError::Deserialize** → toast "Error de datos"

---

## Design System

- **Theme:** "Ambient Glass + Aurora" (glassmorphism)
- **Colores:** gradientes aurora (azul → púrpura → rosa)
- **Componentes:** bordes redondeados, backdrop-blur, sombras sutiles
- **Responsive:** desktop (sidebar) → mobile (bottom nav)
