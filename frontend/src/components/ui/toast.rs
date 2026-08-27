use leptos::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static TOAST_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq)]
pub enum ToastKind { Success, Error, Info, Warning }

#[derive(Debug, Clone)]
pub struct ToastMsg {
    pub id: usize,
    pub kind: ToastKind,
    pub text: String,
}

#[derive(Clone, Copy)]
pub struct ToastCtx {
    pub toasts: RwSignal<Vec<ToastMsg>>,
}

impl ToastCtx {
    pub fn provide() -> Self {
        let ctx = Self { toasts: RwSignal::new(Vec::new()) };
        provide_context(ctx);
        ctx
    }

    pub fn use_() -> Self {
        use_context::<ToastCtx>().expect("ToastCtx not provided")
    }

    pub fn success(&self, text: impl Into<String>) {
        self.add(ToastKind::Success, text);
    }
    pub fn error(&self, text: impl Into<String>) {
        self.add(ToastKind::Error, text);
    }
    pub fn info(&self, text: impl Into<String>) {
        self.add(ToastKind::Info, text);
    }
    pub fn warning(&self, text: impl Into<String>) {
        self.add(ToastKind::Warning, text);
    }

    fn add(&self, kind: ToastKind, text: impl Into<String>) {
        let id = TOAST_ID.fetch_add(1, Ordering::Relaxed);
        let msg = ToastMsg { id, kind, text: text.into() };
        self.toasts.update(|v| v.push(msg));
    }

    pub fn dismiss(&self, id: usize) {
        self.toasts.update(|v| v.retain(|t| t.id != id));
    }
}

#[component]
pub fn ToastContainer() -> impl IntoView {
    let ctx = ToastCtx::use_();
    let toasts = ctx.toasts;

    view! {
        <div class="toast-container">
            {move || toasts.get().into_iter().map(|t| {
                let id = t.id;
                let kind_class = match t.kind {
                    ToastKind::Success => "toast-success",
                    ToastKind::Error => "toast-error",
                    ToastKind::Info => "toast-info",
                    ToastKind::Warning => "toast-warning",
                };
                let icon = match t.kind {
                    ToastKind::Success => "\u{2713}",
                    ToastKind::Error => "\u{2717}",
                    ToastKind::Info => "\u{2139}",
                    ToastKind::Warning => "\u{26A0}",
                };
                let c = ctx.clone();
                // auto dismiss after 4s
                gloo_timers::callback::Timeout::new(4000, move || c.dismiss(id)).forget();
                view! {
                    <div class={format!("toast {kind_class}")}>
                        <span>{icon}</span>
                        <span>{t.text}</span>
                        <button class="toast-dismiss" on:click=move |_| ctx.dismiss(id)>"\u{00d7}"</button>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
