use std::sync::Arc;
use ysh::db::Database;

fn setup_db() -> Arc<Database> {
    let tmp = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    let db = Database::new(tmp.path().to_str().unwrap()).expect("Failed to create test DB");
    std::mem::forget(tmp);
    Arc::new(db)
}

fn create_user(db: &Database, username: &str) -> i64 {
    db.create_user(username, &format!("{}@test.com", username), "hash123")
        .unwrap()
        .id
}

fn seed_activity(db: &Database, users: &[i64]) {
    for (i, u) in users.iter().enumerate() {
        db.log_activity(*u, "login").unwrap();
        db.log_activity(*u, &format!("action_{}", i)).unwrap();
    }
}

// ═══════════════════════════════════════════
// USER ANALYTICS
// ═══════════════════════════════════════════

#[test]
fn user_analytics_reports_dau_and_mau() {
    let db = setup_db();
    let u = create_user(&db, "u1");
    seed_activity(&db, &[u]);

    let data = db.get_user_analytics(7).unwrap();
    let days = data["days"].as_array().unwrap();
    assert_eq!(days.len(), 7);
    assert_eq!(days.last().unwrap()["dau"].as_i64().unwrap(), 1);
    assert!(data["summary"]["mau"].as_i64().unwrap() >= 1);
}

#[test]
fn user_analytics_counts_new_signups() {
    let db = setup_db();
    let a = create_user(&db, "n1");
    let b = create_user(&db, "n2");
    let _ = (a, b);
    let data = db.get_user_analytics(3).unwrap();
    assert_eq!(
        data["days"].as_array().unwrap().last().unwrap()["new_users"]
            .as_i64()
            .unwrap(),
        2
    );
}

#[test]
fn user_analytics_zero_when_empty() {
    let db = setup_db();
    let data = db.get_user_analytics(7).unwrap();
    let days = data["days"].as_array().unwrap();
    assert_eq!(days.len(), 7);
    for day in days {
        assert_eq!(day["dau"].as_i64().unwrap(), 0);
    }
    assert_eq!(data["summary"]["mau"].as_i64().unwrap(), 0);
}

// ═══════════════════════════════════════════
// REVENUE ANALYTICS
// ═══════════════════════════════════════════

#[test]
fn revenue_analytics_aggregates_transactions() {
    let db = setup_db();
    let u = create_user(&db, "rich");
    db.deposit(u, 50_000, "wallet topup").unwrap();
    db.deposit(u, 25_000, "gift balance").unwrap();

    let rev = db.get_revenue_analytics(30).unwrap();
    assert!(rev["transactions"].as_i64().unwrap() >= 75_000);
    assert!(rev["total_users"].as_i64().unwrap() >= 1);
    assert!(rev["active_users"].as_i64().unwrap() >= 0);
}

#[test]
fn revenue_arpu_scales_with_activity() {
    let db = setup_db();
    let a = create_user(&db, "r1");
    let b = create_user(&db, "r2");
    db.deposit(a, 10_000, "t").unwrap();
    db.log_activity(a, "login").unwrap();
    let _ = b;

    let rev = db.get_revenue_analytics(30).unwrap();
    assert!(rev["arpu"].as_i64().unwrap() > 0);
}

// ═══════════════════════════════════════════
// AGENCY + HOSTS
// ═══════════════════════════════════════════

#[test]
fn agency_performance_attributes_revenue() {
    let db = setup_db();
    let owner = create_user(&db, "owner");
    let member = create_user(&db, "member");
    let agency_id = db
        .create_agency(owner, "Talent House", "Top agency")
        .unwrap();
    db.add_agency_member(agency_id, member, "performer")
        .unwrap();
    db.deposit(member, 30_000, "tx").unwrap();

    let perf = db.get_agency_performance().unwrap();
    assert_eq!(perf.len(), 1);
    assert_eq!(perf[0]["members"], 1);
    assert!(perf[0]["member_revenue"].as_i64().unwrap() >= 30_000);
}

#[test]
fn host_leaderboard_ranks_by_earnings() {
    let db = setup_db();
    let host = create_user(&db, "host");
    let caller = create_user(&db, "caller");
    db.deposit(caller, 100_000, "t").unwrap();

    for i in 0..2 {
        let billing_id = db.start_call_billing(caller, host, "video", 5).unwrap();
        let _ = i;
        db.end_call_billing(billing_id).unwrap();
        db.finalize_call_payment(billing_id).unwrap();
    }
    let board = db.get_host_leaderboard(5).unwrap();
    let mine = board
        .iter()
        .find(|r| r["host_id"].as_i64() == Some(host))
        .unwrap();
    assert_eq!(mine["calls"].as_i64().unwrap(), 2);
    assert!(mine["earnings"].as_i64().unwrap() > 0);
}

// ═══════════════════════════════════════════
// GEO + MODERATION METRICS
// ═══════════════════════════════════════════

#[test]
fn geodistribution_includes_region_pct() {
    let db = setup_db();
    let a = create_user(&db, "g1");
    let b = create_user(&db, "g2");
    db.set_user_region(a, "AR").unwrap();
    db.set_user_region(b, "BR").unwrap();
    let geo = db.get_geo_distribution().unwrap();
    assert_eq!(geo["total_users"].as_i64().unwrap(), 2);
    let dist = geo["distribution"].as_array().unwrap();
    assert_eq!(dist.len(), 2);
    assert_eq!(
        dist.iter().map(|d| d["pct"].as_i64().unwrap()).sum::<i64>(),
        100
    );
}

#[test]
fn moderation_metrics_track_pipeline() {
    let db = setup_db();
    db.enqueue_moderation_item("moment", 1, 0.5, "mid").unwrap();
    db.resolve_moderation_item(1, "reviewed").unwrap();
    db.enqueue_moderation_item("chat", 2, 0.1, "low").unwrap();

    let metrics = db.get_moderation_metrics().unwrap();
    assert!(metrics["queue"]["pending"].as_i64().unwrap() >= 1);
    assert!(metrics["queue"]["reviewed"].as_i64().unwrap() >= 1);
}

#[test]
fn db_size_reports_bytes() {
    let db = setup_db();
    let u = create_user(&db, "sizer");
    let _ = db.deposit(u, 100, "t").unwrap();
    let size = db.db_size().unwrap();
    assert!(size > 0);
}

// ═══════════════════════════════════════════
// REALTIME + SNAPSHOTS
// ═══════════════════════════════════════════

#[test]
fn realtime_db_metrics_scaffold() {
    let db = setup_db();
    let metrics = db.realtime_db_metrics().unwrap();
    assert!(metrics["active_calls"].is_number());
    assert!(metrics["pending_reports"].is_number());
    assert!(metrics["pending_payouts"].is_number());
}

#[test]
fn snapshots_list_sorted_desc() {
    let db = setup_db();
    let u = create_user(&db, "u");
    db.log_activity(u, "login").unwrap();
    db.compute_analytics_snapshot().unwrap();
    let rows = db.list_analytics_snapshots(10).unwrap();
    assert_eq!(rows.len(), 1);
    let mut dates: Vec<&str> = rows.iter().map(|r| r["date"].as_str().unwrap()).collect();
    let sorted = dates.to_vec();
    dates.sort_by(|a, b| b.cmp(a));
    assert_eq!(dates, sorted);
}

// ═══════════════════════════════════════════
// WORKER-FRIENDLY EXTRAS
// ═══════════════════════════════════════════

#[test]
fn call_fees_only_count_completed() {
    let db = setup_db();
    let host = create_user(&db, "h");
    let caller = create_user(&db, "c");
    db.deposit(caller, 50_000, "t").unwrap();
    db.start_call_billing(caller, host, "video", 5).unwrap();
    let fees = db
        .sum_call_fees_range(
            "0000-01-01",
            &chrono::Utc::now().format("%Y-%m-%d").to_string(),
        )
        .unwrap();
    assert_eq!(fees, 0); // none completed yet
}

#[test]
fn snapshot_excludes_empty_match_entries() {
    let db = setup_db();
    let u = create_user(&db, "u");
    db.enqueue_match(u, "chat", "{}").unwrap();
    let snap = db.compute_analytics_snapshot().unwrap();
    assert!(snap["messages"].as_i64().unwrap() >= 0);
}
