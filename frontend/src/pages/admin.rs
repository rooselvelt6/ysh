use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::store;

fn fmt_int(v: i64) -> String {
    let s = v.abs().to_string();
    let mut out: String = s.chars().rev().enumerate().fold(String::new(), |mut acc, (i, c)| {
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

fn fmt_bytes(n: u64) -> String {
    if n >= 1024 * 1024 {
        format!("{:.1} MB", n as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.0} KB", n as f64 / 1024.0)
    }
}

fn fmt_pct(v: f64) -> String {
    format!("{v:.1}%")
}

fn num(v: &serde_json::Value, key: &str) -> i64 {
    v.get(key).and_then(|x| x.as_i64()).unwrap_or(0)
}

fn str_(v: &serde_json::Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").to_string()
}

#[component]
pub fn AdminPage() -> impl IntoView {
    let is_admin = store::get_user().map(|u| u.role == "admin").unwrap_or(false);

    let (tab, set_tab) = signal(0u8);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(String::new());

    let (realtime, set_realtime) = signal(serde_json::Value::Null);
    let (health, set_health) = signal(serde_json::Value::Null);
    let (users, set_users) = signal(serde_json::Value::Null);
    let (revenue, set_revenue) = signal(serde_json::Value::Null);
    let (agencies, set_agencies) = signal(Vec::<serde_json::Value>::new());
    let (hosts, set_hosts) = signal(Vec::<serde_json::Value>::new());
    let (geo, set_geo) = signal(serde_json::Value::Null);
    let (moderation, set_moderation) = signal(serde_json::Value::Null);
    let (snapshots, set_snapshots) = signal(Vec::<serde_json::Value>::new());

    let load = move || {
        set_loading.set(true);
        spawn_local(async move {
            let mut had_error = false;
            match api::get::<serde_json::Value>("/admin/analytics/realtime").await {
                Ok(v) => set_realtime.set(v),
                Err(e) => { had_error = true; set_error.set(e.to_string()); }
            }
            if let Ok(v) = api::get::<serde_json::Value>("/admin/analytics/health").await {
                set_health.set(v);
            }
            if let Ok(v) = api::get::<serde_json::Value>("/admin/analytics/users?days=14").await {
                set_users.set(v);
            }
            if let Ok(v) = api::get::<serde_json::Value>("/admin/analytics/revenue?days=30").await {
                set_revenue.set(v);
            }
            if let Ok(v) = api::get::<serde_json::Value>("/admin/analytics/agencies").await {
                if let Some(a) = v.get("agencies").and_then(|x| x.as_array()) {
                    set_agencies.set(a.clone());
                }
            }
            if let Ok(v) = api::get::<serde_json::Value>("/admin/analytics/hosts?limit=10").await {
                if let Some(a) = v.get("leaderboard").and_then(|x| x.as_array()) {
                    set_hosts.set(a.clone());
                }
            }
            if let Ok(v) = api::get::<serde_json::Value>("/admin/analytics/geo").await {
                set_geo.set(v);
            }
            if let Ok(v) = api::get::<serde_json::Value>("/admin/analytics/moderation").await {
                set_moderation.set(v);
            }
            if let Ok(v) = api::get::<serde_json::Value>("/admin/analytics/snapshots?limit=7").await {
                if let Some(a) = v.get("snapshots").and_then(|x| x.as_array()) {
                    set_snapshots.set(a.clone());
                }
            }
            if had_error {
                set_error.set(String::new());
            }
            set_loading.set(false);
        });
    };
    load();

    let tabs: Vec<(&str, &str)> = vec![
        ("\u{26A1}", "Realtime"),
        ("\u{1F465}", "Users"),
        ("\u{1F4B8}", "Revenue"),
        ("\u{1F3E2}", "Agencies"),
        ("\u{1F465}\u{FE0F}", "Hosts"),
        ("\u{1F30D}", "Geo"),
        ("\u{2696}\u{FE0F}", "Moderation"),
    ];

    view! {
        <div class="main-header">
            <div style="display:flex;justify-content:space-between;align-items:center;gap:8px;flex-wrap:wrap;">
                <div>
                    <h2>"Analytics"</h2>
                    <div style="color:var(--text-dim);font-size:0.8125rem;">"Admin dashboard"</div>
                </div>
                <div style="display:flex;gap:8px;">
                    <a class="btn btn-outline btn-sm" href="/admin/analytics/export?dataset=users&format=csv">
                        "Users CSV"
                    </a>
                    <a class="btn btn-outline btn-sm" href="/admin/analytics/export?dataset=revenue&format=csv">
                        "Revenue CSV"
                    </a>
                    <button class="btn btn-primary btn-sm" on:click=move |_| load()>
                        "\u{1F504} Refresh"
                    </button>
                </div>
            </div>
        </div>

        {move || {
            if !is_admin {
                return view! {
                    <div class="empty-state" style="margin-top:24px;">
                        <div class="empty-state-icon">"\u{1F512}"</div>
                        <div class="empty-state-title">"Admin access required"</div>
                        <div class="empty-state-text">"This dashboard is only available to administrators."</div>
                    </div>
                }.into_any();
            }
            if !error.get().is_empty() {
                show_error_alert(&error.get()).into_any()
            } else if loading.get() {
                view! { <div class="loading-center"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else {
                match tab.get() {
                    0 => realtime_view(realtime, health).into_any(),
                    1 => users_view(users, snapshots).into_any(),
                    2 => revenue_view(revenue).into_any(),
                    3 => agencies_view(agencies).into_any(),
                    4 => hosts_view(hosts).into_any(),
                    5 => geo_view(geo).into_any(),
                    _ => moderation_view(moderation).into_any(),
                }
            }
        }}

        <div style="position:sticky;bottom:0;background:var(--bg);padding:10px 0;border-top:1px solid var(--border);margin-top:12px;">
            <div class="tabs">
                {tabs.into_iter().enumerate().map(|(idx, (icon, label))| {
                    view! {
                        <button
                            class={move || format!("tab{}", if tab.get() as usize == idx { " active" } else { "" })}
                            on:click=move |_| set_tab.set(idx as u8)
                        >
                            <span>{icon}</span> " " {label}
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

fn show_error_alert(msg: &str) -> impl IntoView {
    view! {
        <div class="alert alert-error" style="margin-top:16px;">
            {
                msg.to_string()
            }
        </div>
    }.into_any()
}

fn stat_grid(entries: Vec<(&'static str, String)>) -> impl IntoView {
    view! {
        <div class="stat-grid">
            {entries.into_iter().map(|(label, value)| {
                view! {
                    <div class="stat-tile">
                        <div class="stat-value">{value}</div>
                        <div class="stat-label">{label}</div>
                    </div>
                }
            }).collect_view()}
        </div>
    }.into_any()
}

fn realtime_view(realtime: ReadSignal<serde_json::Value>, health: ReadSignal<serde_json::Value>) -> impl IntoView {
    let rt = || {
        let v = realtime.get();
        let online = num(&v, "online_users");
        let rooms = num(&v, "active_rooms");
        let cache = num(&v, "cache_entries");
        let cache_bytes = v.get("cache_bytes").and_then(|x| x.as_u64()).unwrap_or(0);
        let today_tx = v.get("today").and_then(|t| t.get("transactions")).and_then(|x| x.as_i64()).unwrap_or(0);
        let today_gifts = v.get("today").and_then(|t| t.get("gifts")).and_then(|x| x.as_i64()).unwrap_or(0);
        let db_active = v.get("db").and_then(|d| d.get("active_calls")).and_then(|x| x.as_i64()).unwrap_or(0);
        let db_pending = v.get("db").and_then(|d| d.get("pending_reports")).and_then(|x| x.as_i64()).unwrap_or(0);
        (online, rooms, cache, cache_bytes, today_tx, today_gifts, db_active, db_pending)
    };
    let h = move || {
        let v = health.get();
        let uptime = v.get("uptime_secs").and_then(|x| x.as_f64()).unwrap_or(0.0);
        let cpu = v.get("cpu_usage_pct").and_then(|x| x.as_f64()).unwrap_or(0.0);
        let mem_pct = v.get("memory").and_then(|m| m.get("used_pct").and_then(|x| x.as_f64())).unwrap_or(0.0);
        let db_size = v.get("db_size_bytes").and_then(|x| x.as_u64()).unwrap_or(0);
        let threads = num(&v, "threads");
        (uptime, cpu, mem_pct, db_size, threads)
    };

    let (online, rooms, cache, cache_bytes, today_tx, today_gifts, db_active, db_pending) = rt();
    let (uptime, cpu, mem_pct, db_size, threads) = h();

    view! {
        <div>
            <div style="margin-top:16px;">
                {stat_grid(vec![
                    ("Online users", fmt_int(online)),
                    ("Active rooms", fmt_int(rooms)),
                    ("Active calls", fmt_int(db_active)),
                    ("Pending reports", fmt_int(db_pending)),
                    ("Cache entries", fmt_int(cache)),
                    ("Cache size", fmt_bytes(cache_bytes)),
                    ("Today: transactions", fmt_int(today_tx)),
                    ("Today: gifts", fmt_int(today_gifts)),
                ])}
            </div>
            <div class="panel">
                <div class="panel-title">"System health"</div>
                <div class="stat-grid">
                    {vec![
                        ("Uptime (s)", format!("{uptime:.0}")),
                        ("CPU usage", fmt_pct(cpu)),
                        ("Memory used", fmt_pct(mem_pct)),
                        ("DB size", fmt_bytes(db_size)),
                        ("Threads", fmt_int(threads)),
                    ].into_iter().map(|(label, value)| {
                        view! {
                            <div class="stat-tile">
                                <div class="stat-value">{value}</div>
                                <div class="stat-label">{label}</div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }.into_any()
}

fn users_view(users: ReadSignal<serde_json::Value>, snapshots: ReadSignal<Vec<serde_json::Value>>) -> impl IntoView {
    let summary = move || {
        let v = users.get();
        let mau = v.get("summary").and_then(|s| s.get("mau")).and_then(|x| x.as_i64()).unwrap_or(0);
        let dau = v.get("summary").and_then(|s| s.get("total_dau_last_day")).and_then(|x| x.as_i64()).unwrap_or(0);
        (mau, dau)
    };
    let (mau, dau) = summary();
    let days = move || {
        users.get().get("days").and_then(|x| x.as_array()).cloned().unwrap_or_default()
    };
    let day_list = days();
    let max_dau = day_list.iter().map(|d| num(d, "dau")).max().unwrap_or(1).max(1);

    view! {
        <div>
            <div style="margin-top:16px;">
                {stat_grid(vec![
                    ("MAU (30d)", fmt_int(mau)),
                    ("DAU (today)", fmt_int(dau)),
                ])}
            </div>
            <div class="panel">
                <div class="panel-title">"DAU — last 14 days"</div>
                <div class="mini-bars">
                    {day_list.iter().map(|d| {
                        let dau = num(d, "dau");
                        let h = if max_dau > 0 { (dau as f64 / max_dau as f64 * 100.0).max(2.0) } else { 2.0 };
                        view! {
                            <div class="mini-bar" style={format!("height:{}%", h)} title={format!("{}: {}", str_(d, "date"), dau)}></div>
                        }
                    }).collect_view()}
                </div>
                <div style="display:flex;justify-content:space-between;color:var(--text-dim);font-size:0.75rem;margin-top:6px;">
                    <span>{day_list.first().map(|d| str_(d, "date")).unwrap_or_default()}</span>
                    <span>{day_list.last().map(|d| str_(d, "date")).unwrap_or_default()}</span>
                </div>
            </div>
            {if day_list.is_empty() {
                view! {}.into_any()
            } else {
                view! {
                    <div class="panel">
                        <div class="panel-title">"Per-day"</div>
                        <table class="data-table">
                            <thead>
                                <tr><th>"Date"</th><th>"DAU"</th><th>"New signups"</th><th>"Retention"</th><th>"Churn"</th></tr>
                            </thead>
                            <tbody>
                                {day_list.iter().rev().map(|d| {
                                    let date = str_(d, "date");
                                    let right = d.get("retention")
                                        .and_then(|r| if r.is_string() { None } else { r.as_f64() })
                                        .map(fmt_pct);
                                    let churn = d.get("churn")
                                        .and_then(|c| if c.is_string() { None } else { c.as_f64() })
                                        .map(fmt_pct);
                                    view! {
                                        <tr>
                                            <td>{date}</td>
                                            <td>{fmt_int(num(d, "dau"))}</td>
                                            <td>{fmt_int(num(d, "new_users"))}</td>
                                            <td>{right.unwrap_or_else(|| "\u{2013}".into())}</td>
                                            <td>{churn.unwrap_or_else(|| "\u{2013}".into())}</td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                }.into_any()
            }}
            {if snapshots.get().is_empty() {
                view! {}.into_any()
            } else {
                view! {
                    <div class="panel">
                        <div class="panel-title">"Daily snapshots (worker)"</div>
                        <table class="data-table">
                            <thead>
                                <tr><th>"Date"</th><th>"DAU"</th><th>"MAU"</th><th>"New"</th><th>"Tx"</th><th>"Gifts"</th><th>"Calls"</th><th>"Msgs"</th></tr>
                            </thead>
                            <tbody>
                                {snapshots.get().iter().map(|s| {
                                    view! {
                                        <tr>
                                            <td>{str_(s, "date")}</td>
                                            <td>{fmt_int(num(s, "dau"))}</td>
                                            <td>{fmt_int(num(s, "mau"))}</td>
                                            <td>{fmt_int(num(s, "new_users"))}</td>
                                            <td>{fmt_int(num(s, "transactions"))}</td>
                                            <td>{fmt_int(num(s, "gifts"))}</td>
                                            <td>{fmt_int(num(s, "calls"))}</td>
                                            <td>{fmt_int(num(s, "messages"))}</td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                }.into_any()
            }}
        </div>
    }.into_any()
}

fn revenue_view(revenue: ReadSignal<serde_json::Value>) -> impl IntoView {
    let r = move || {
        let v = revenue.get();
        (
            num(&v, "gross_volume"),
            num(&v, "transactions"),
            num(&v, "gifts"),
            num(&v, "calls_fees"),
            num(&v, "arpu"),
            num(&v, "ltv"),
            num(&v, "active_users"),
            num(&v, "total_users"),
        )
    };
    let (gross, tx, gifts, fees, arpu, ltv, active, total) = r();
    view! {
        <div style="margin-top:16px;">
            {stat_grid(vec![
                ("Gross volume", fmt_int(gross)),
                ("Transactions", fmt_int(tx)),
                ("Gifts", fmt_int(gifts)),
                ("Call fees", fmt_int(fees)),
                ("ARPU", fmt_int(arpu)),
                ("LTV (12m)", fmt_int(ltv)),
                ("Active users", fmt_int(active)),
                ("Total users", fmt_int(total)),
            ])}
        </div>
    }.into_any()
}

fn agencies_view(agencies: ReadSignal<Vec<serde_json::Value>>) -> impl IntoView {
    let list = agencies.get();
    if list.is_empty() {
        return view! {
            <div class="empty-state" style="margin-top:24px;">
                <div class="empty-state-icon">"\u{1F3E2}"</div>
                <div class="empty-state-title">"No agencies"</div>
                <div class="empty-state-text">"Agency performance will appear here."</div>
            </div>
        }.into_any();
    }
    view! {
        <div class="panel" style="margin-top:16px;">
            <div class="panel-title">"Agency performance"</div>
            <table class="data-table">
                <thead>
                    <tr><th>"Agency"</th><th>"Members"</th><th>"Member revenue"</th><th>"Member calls"</th></tr>
                </thead>
                <tbody>
                    {list.iter().map(|a| {
                        view! {
                            <tr>
                                <td>{str_(a, "name")}</td>
                                <td>{fmt_int(num(a, "members"))}</td>
                                <td>{fmt_int(num(a, "member_revenue"))}</td>
                                <td>{fmt_int(num(a, "member_calls"))}</td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }.into_any()
}

fn hosts_view(hosts: ReadSignal<Vec<serde_json::Value>>) -> impl IntoView {
    let list = hosts.get();
    if list.is_empty() {
        return view! {
            <div class="empty-state" style="margin-top:24px;">
                <div class="empty-state-icon">"\u{1F465}"</div>
                <div class="empty-state-title">"No host activity"</div>
                <div class="empty-state-text">"Completed calls will seed this leaderboard."</div>
            </div>
        }.into_any();
    }
    view! {
        <div class="panel" style="margin-top:16px;">
            <div class="panel-title">"Host leaderboard"</div>
            <table class="data-table">
                <thead>
                    <tr><th>"# "</th><th>"Host"</th><th>"Calls"</th><th>"Earnings"</th></tr>
                </thead>
                <tbody>
                    {list.iter().enumerate().map(|(i, h)| {
                        view! {
                            <tr>
                                <td>{fmt_int((i + 1) as i64)}</td>
                                <td>{format!("#{}", num(h, "host_id"))}</td>
                                <td>{fmt_int(num(h, "calls"))}</td>
                                <td>{fmt_int(num(h, "earnings"))}</td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }.into_any()
}

fn geo_view(geo: ReadSignal<serde_json::Value>) -> impl IntoView {
    let dist = move || {
        geo.get().get("distribution").and_then(|x| x.as_array()).cloned().unwrap_or_default()
    };
    let list = dist();
    let total = num(&geo.get(), "total_users");
    if list.is_empty() {
        return view! {
            <div class="empty-state" style="margin-top:24px;">
                <div class="empty-state-icon">"\u{1F30D}"</div>
                <div class="empty-state-title">"No regions yet"</div>
                <div class="empty-state-text">"Users set their region from their profile."</div>
            </div>
        }.into_any();
    }
    let max_users = list.iter().map(|g| num(g, "users")).max().unwrap_or(1).max(1);
    view! {
        <div style="margin-top:16px;">
            {stat_grid(vec![("Total users mapped", fmt_int(total))])}
            <div class="panel">
                <div class="panel-title">"User regions"</div>
                {list.iter().map(|g| {
                    let users = num(g, "users");
                    let pct = num(g, "pct");
                    let width = (users as f64 / max_users as f64 * 100.0).max(1.0);
                    let bar_text = format!("{users} ({pct}%)");
                    view! {
                        <div class="bar-row">
                            <div class="bar-label">{str_(g, "region")}</div>
                            <div class="bar-track"><div class="bar-fill" style={format!("width:{}%", width)}></div></div>
                            <div class="bar-value">{bar_text}</div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }.into_any()
}

fn moderation_view(moderation: ReadSignal<serde_json::Value>) -> impl IntoView {
    let counts = move || {
        let v = moderation.get();
        let queue = v.get("queue").and_then(|q| q.as_object()).cloned().unwrap_or_default();
        let reports = v.get("reports").and_then(|q| q.as_object()).cloned().unwrap_or_default();
        let appeals = v.get("appeals").and_then(|q| q.as_object()).cloned().unwrap_or_default();
        let shadow = num(&v, "active_shadow_bans");
        (queue, reports, appeals, shadow)
    };
    let (queue, reports, appeals, shadow) = counts();

    fn kv_list(map: serde_json::Map<String, serde_json::Value>) -> Vec<(String, i64)> {
        let mut v: Vec<(String, i64)> = map.iter()
            .map(|(k, val)| (k.clone(), val.as_i64().unwrap_or(0)))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v
    }

    let queue_rows = kv_list(queue);
    let report_rows = kv_list(reports);
    let appeal_rows = kv_list(appeals);

    view! {
        <div style="margin-top:16px;">
            {stat_grid(vec![("Active shadow bans", fmt_int(shadow))])}
            <div class="panel">
                <div class="panel-title">"Moderation queue"</div>
                {if queue_rows.is_empty() {
                    view! { <div class="item-meta">"No items in queue."</div> }.into_any()
                } else {
                    view! {
                        {queue_rows.into_iter().map(|(k, v)| {
                            view! {
                                <div class="bar-row">
                                    <div class="bar-label">{if k.is_empty() { "unknown".to_string() } else { k }}</div>
                                    <div class="bar-value" style="text-align:right;">{fmt_int(v)}</div>
                                </div>
                            }
                        }).collect_view()}
                    }.into_any()
                }}
            </div>
            <div class="panel">
                <div class="panel-title">"Reports"</div>
                {if report_rows.is_empty() {
                    view! { <div class="item-meta">"No reports."</div> }.into_any()
                } else {
                    view! {
                        {report_rows.into_iter().map(|(k, v)| {
                            view! {
                                <div class="bar-row">
                                    <div class="bar-label">{if k.is_empty() { "unknown".to_string() } else { k }}</div>
                                    <div class="bar-value" style="text-align:right;">{fmt_int(v)}</div>
                                </div>
                            }
                        }).collect_view()}
                    }.into_any()
                }}
            </div>
            <div class="panel">
                <div class="panel-title">"Appeals"</div>
                {if appeal_rows.is_empty() {
                    view! { <div class="item-meta">"No appeals."</div> }.into_any()
                } else {
                    view! {
                        {appeal_rows.into_iter().map(|(k, v)| {
                            view! {
                                <div class="bar-row">
                                    <div class="bar-label">{if k.is_empty() { "unknown".to_string() } else { k }}</div>
                                    <div class="bar-value" style="text-align:right;">{fmt_int(v)}</div>
                                </div>
                            }
                        }).collect_view()}
                    }.into_any()
                }}
            </div>
        </div>
    }.into_any()
}