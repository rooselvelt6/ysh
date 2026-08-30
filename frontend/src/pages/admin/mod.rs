use leptos::prelude::*;

pub mod agencies;
pub mod calls;
pub mod i18n;
pub mod jobs;
pub mod moderation;
pub mod moments;
pub mod overview;
pub mod payouts;
pub mod users;
pub mod wallets;

use agencies::AgenciesTab;
use calls::CallsTab;
use i18n::I18nTab;
use jobs::JobsTab;
use moderation::ModerationTab;
use moments::MomentsTab;
use overview::OverviewTab;
use payouts::PayoutsTab;
use users::UsersTab;
use wallets::WalletsTab;

pub(crate) fn fmt_int(v: i64) -> String {
    let s = v.abs().to_string();
    let mut out: String = s
        .chars()
        .rev()
        .enumerate()
        .fold(String::new(), |mut acc, (i, c)| {
            if i > 0 && i % 3 == 0 {
                acc.push(',');
            }
            acc.push(c);
            acc
        });
    out = out.chars().rev().collect();
    if v < 0 {
        format!("-{out}")
    } else {
        out
    }
}

pub(crate) fn fmt_amt(unit_cents: i64) -> String {
    let sign = if unit_cents < 0 { "-" } else { "" };
    let a = unit_cents.abs();
    if a == 0 {
        "0.00".to_string()
    } else {
        format!("{sign}{}.{:02}", fmt_int(a / 100), a % 100)
    }
}

pub(crate) fn fmt_pct(v: f64) -> String {
    format!("{v:.1}%")
}

pub(crate) fn fmt_bytes(n: u64) -> String {
    if n >= 1024 * 1024 {
        format!("{:.1} MB", n as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.0} KB", n as f64 / 1024.0)
    }
}

pub(crate) fn num(v: &serde_json::Value, key: &str) -> i64 {
    v.get(key).and_then(|x| x.as_i64()).unwrap_or(0)
}

pub(crate) fn f(v: &serde_json::Value, key: &str) -> f64 {
    v.get(key).and_then(|x| x.as_f64()).unwrap_or(0.0)
}

pub(crate) fn s(v: &serde_json::Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").to_string()
}

pub(crate) fn truncate(text: &str, max: usize) -> String {
    let t = text.trim();
    if t.chars().count() <= max {
        t.to_string()
    } else {
        let mut s: String = t.chars().take(max).collect();
        s.push_str("…");
        s
    }
}

#[component]
pub fn AdminPage() -> impl IntoView {
    let is_admin = crate::store::get_user()
        .map(|u| u.role == "admin")
        .unwrap_or(false);

    let (tab, set_tab) = signal(0usize);

    let tabs: Vec<(&str, &str)> = vec![
        ("\u{1F4CA}", "Resumen"),
        ("\u{1F465}", "Usuarios"),
        ("\u{1F4B0}", "Wallets & TX"),
        ("\u{1F4F7}", "Momentos"),
        ("\u{1F3E2}", "Agencias"),
        ("\u{260E}\u{FE0F}", "Llamadas"),
        ("\u{1F4E9}", "Pagos"),
        ("\u{2696}\u{FE0F}", "Moderación"),
        ("\u{1F4C8}", "Jobs"),
        ("\u{1F310}", "Idioma"),
    ];

    view! {
        <div class="main-header">
            <div style="display:flex;justify-content:space-between;align-items:center;gap:8px;flex-wrap:wrap;">
                <div>
                    <h2>"Panel Administrativo"</h2>
                    <div style="color:var(--text-dim);font-size:0.8125rem;">"Control total de la plataforma YSH"</div>
                </div>
            </div>
        </div>

        {move || {
            if !is_admin {
                return view! {
                    <div class="empty-state" style="margin-top:24px;">
                        <div class="empty-state-icon">"\u{1F512}"</div>
                        <div class="empty-state-title">"Acceso administrativo requerido"</div>
                        <div class="empty-state-text">"Solo los administradores pueden ver este panel."</div>
                    </div>
                }.into_any();
            }
            match tab.get() {
                0 => view! { <OverviewTab /> }.into_any(),
                1 => view! { <UsersTab /> }.into_any(),
                2 => view! { <WalletsTab /> }.into_any(),
                3 => view! { <MomentsTab /> }.into_any(),
                4 => view! { <AgenciesTab /> }.into_any(),
                5 => view! { <CallsTab /> }.into_any(),
                6 => view! { <PayoutsTab /> }.into_any(),
                7 => view! { <ModerationTab /> }.into_any(),
                8 => view! { <JobsTab /> }.into_any(),
                _ => view! { <I18nTab /> }.into_any(),
            }
        }}

        <div class="tabs" style="margin-top:18px;flex-wrap:wrap;">
            {tabs.into_iter().enumerate().map(|(idx, (icon, label))| {
                view! {
                    <button
                        class={move || format!("tab{}", if tab.get() == idx { " active" } else { "" })}
                        on:click=move |_| set_tab.set(idx)
                    >
                        <span>{icon}</span> " " {label}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}