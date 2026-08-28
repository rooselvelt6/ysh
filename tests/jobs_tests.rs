use serde_json::json;
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

// ═══════════════════════════════════════════
// PAYOUT WORKER
// ═══════════════════════════════════════════

#[test]
fn auto_processes_pending_payouts() {
    let db = setup_db();
    let u = create_user(&db, "payer");

    db.deposit(u, 100000, "test").unwrap();
    let pid = db.request_payout(u, 5000, "YSHT", "0xabc", "eth_network").unwrap();
    let result = db.auto_process_payouts().unwrap();

    assert_eq!(result["processed"].as_i64().unwrap(), 1);
    assert_eq!(result["total_amount"].as_i64().unwrap(), 5000);

    let payouts = db.get_user_payouts(u).unwrap();
    let p = &payouts[0];
    assert_eq!(p["status"], "completed");
    assert!(p["tx_hash"].as_str().unwrap().starts_with("0xauto"));

    // No pending left -> idempotent.
    let again = db.auto_process_payouts().unwrap();
    assert_eq!(again["processed"].as_i64().unwrap(), 0);
    let _ = pid;
}

#[test]
fn payout_worker_skips_already_processed() {
    let db = setup_db();
    let u = create_user(&db, "p2");
    db.deposit(u, 100000, "test").unwrap();
    db.request_payout(u, 1000, "YSH", "0xw", "eth").unwrap();
    let first = db.auto_process_payouts().unwrap();
    assert_eq!(first["processed"].as_i64().unwrap(), 1);
    let result = db.auto_process_payouts().unwrap();
    assert_eq!(result["processed"].as_i64().unwrap(), 0);
}

// ═══════════════════════════════════════════
// STAKING WORKER
// ═══════════════════════════════════════════

#[test]
fn staking_interest_accrues_over_time() {
    let db = setup_db();
    let u = create_user(&db, "staker");
    db.deposit(u, 1_000_000, "test").unwrap();

    let apy = 10.0; // 10% per year
    let stake_id = db.stake(u, 100_000, apy, 30).unwrap();

    // Force an old last_reward_calc so interest accrues immediately.
    let old = (chrono::Utc::now() - chrono::Duration::days(365 * 2))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    db.set_staking_recalc(stake_id, &old).unwrap();

    let result = db.compute_staking_interest().unwrap();
    assert_eq!(result["updated"].as_i64().unwrap(), 1);
    let interest = result["total_interest"].as_i64().unwrap();

    // 100_000 * 10% * 2 years = 20_000.
    assert!(interest >= 20_000, "expected ~20k interest, got {}", interest);
}

#[test]
fn staking_interest_ignores_inactive_stakes() {
    let db = setup_db();
    let u = create_user(&db, "staker2");
    db.deposit(u, 1_000_000, "test").unwrap();
    let _sid = db.stake(u, 100_000, 10.0, 1).unwrap();
    let result = db.compute_staking_interest().unwrap();
    // No time passed (cooldown) -> nothing updated.
    assert_eq!(result["updated"].as_i64().unwrap(), 0);
}

// ═══════════════════════════════════════════
// MODERATION WORKER
// ═══════════════════════════════════════════

#[test]
fn moderation_autoresolve_expired_items() {
    let db = setup_db();
    let u = create_user(&db, "reporter");

    db.enqueue_moderation_item("moment", 101, 0.9, "auto-severe").unwrap();
    db.enqueue_moderation_item("moment", 102, 0.1, "auto-mild").unwrap();

    // Age items by rewriting created_at to 8 days ago via direct db call.
    db.age_moderation_items(8 * 86400).unwrap();

    let result = db.auto_resolve_moderation(7 * 86400, 0.4, 0.8).unwrap();
    assert_eq!(result["actioned"].as_i64().unwrap(), 1);
    assert_eq!(result["dismissed"].as_i64().unwrap(), 1);

    let queue = db.get_moderation_queue(Some("pending")).unwrap();
    assert!(queue.is_empty());
    let _ = u;
}

#[test]
fn moderation_keeps_recent_items_pending() {
    let db = setup_db();
    db.enqueue_moderation_item("chat", 1, 0.9, "fresh").unwrap();
    let result = db.auto_resolve_moderation(7 * 86400, 0.4, 0.8).unwrap();
    assert_eq!(result["actioned"].as_i64().unwrap(), 0);
    assert_eq!(result["dismissed"].as_i64().unwrap(), 0);
}

// ═══════════════════════════════════════════
// CLEANUP WORKER
// ═══════════════════════════════════════════

#[test]
fn cleanup_removes_stale_and_old_data() {
    let db = setup_db();
    let u = create_user(&db, "clean");

    // Stale match queue entry.
    db.enqueue_match(u, "chat", &json!({}).to_string()).unwrap();
    db.age_match_queue(40 * 60).unwrap(); // 40 minutes old

    // Old activity.
    db.log_activity(u, "login").unwrap();
    db.age_activity(40).unwrap(); // 40 days old

    let result = db.cleanup_expired(30, 7).unwrap();
    assert!(result["removed"].as_i64().unwrap() >= 1);
}

// ═══════════════════════════════════════════
// NOTIFICATION WORKER
// ═══════════════════════════════════════════

#[test]
fn notification_flush_marks_sent() {
    let db = setup_db();
    let u = create_user(&db, "nuser");
    let nid = db.create_notification(u, "inapp", "Hi", "Body", "{}", "inapp").unwrap();
    assert_eq!(nid, 1);

    let result = db.flush_pending_notifications().unwrap();
    assert_eq!(result["sent"].as_i64().unwrap(), 1);

    let notifications = db.get_notifications(u, 10).unwrap();
    assert_eq!(notifications[0]["status"], "sent");
    assert!(notifications[0]["sent_at"].is_string());

    let again = db.flush_pending_notifications().unwrap();
    assert_eq!(again["sent"].as_i64().unwrap(), 0);
}

#[test]
fn notification_fails_after_retries() {
    let db = setup_db();
    let u = create_user(&db, "nuser2");
    db.create_notification(u, "inapp", "T", "B", "{}", "push").unwrap();
    assert_eq!(db.flush_pending_notifications().unwrap()["pending"].as_i64().unwrap(), 1);
    assert_eq!(db.flush_pending_notifications().unwrap()["pending"].as_i64().unwrap(), 1);
    // 3rd sweep: retry budget exhausted -> moves to failed state.
    let result = db.flush_pending_notifications().unwrap();
    assert_eq!(result["failed"].as_i64().unwrap(), 1);
}

// ═══════════════════════════════════════════
// ANALYTICS WORKER
// ═══════════════════════════════════════════

#[test]
fn analytics_snapshot_records_activity_counts() {
    let db = setup_db();
    let u1 = create_user(&db, "a1");
    let u2 = create_user(&db, "a2");

    db.log_activity(u1, "login").unwrap();
    db.log_activity(u2, "login").unwrap();
    db.log_activity(u2, "call").unwrap();

    let snap = db.compute_analytics_snapshot().unwrap();
    assert_eq!(snap["dau"], 2);
    assert!(snap["mau"].as_i64().unwrap() >= 2);
    assert_eq!(snap["new_users"].as_i64().unwrap(), 2);
    assert_eq!(snap["date"].as_str().unwrap(), &chrono::Utc::now().format("%Y-%m-%d").to_string());

    // Persisted + retrievable.
    let rows = db.list_analytics_snapshots(5).unwrap();
    assert!(!rows.is_empty());
    assert_eq!(rows[0]["date"].as_str(), snap["date"].as_str());
}

#[test]
fn log_activity_dedupes_same_action() {
    let db = setup_db();
    let u = create_user(&db, "dedupe");
    let other = create_user(&db, "dedupe2");
    db.log_activity(u, "login").unwrap();
    db.log_activity(u, "login").unwrap();
    db.log_activity(other, "login").unwrap();
    let snap = db.compute_analytics_snapshot().unwrap();
    assert_eq!(snap["dau"], 2);
}

// ═══════════════════════════════════════════
// ACTIVITY + REGION
// ═══════════════════════════════════════════

#[test]
fn region_setting_roundtrip() {
    let db = setup_db();
    let u = create_user(&db, "geo");
    assert_eq!(db.find_user_by_id(u).unwrap().unwrap().region, "unknown");
    db.set_user_region(u, "ES-Catalunya").unwrap();
    let user = db.find_user_by_id(u).unwrap().unwrap();
    assert_eq!(user.region, "ES-Catalunya");
}

#[test]
fn geodistribution_buckets_regions() {
    let db = setup_db();
    let a = create_user(&db, "g1");
    let b = create_user(&db, "g2");
    db.set_user_region(a, "MX").unwrap();
    db.set_user_region(b, "MX").unwrap();
    let geo = db.get_geo_distribution().unwrap();
    let mx = geo["distribution"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["region"] == "MX")
        .unwrap();
    assert_eq!(mx["users"], 2);
    assert_eq!(geo["total_users"].as_i64().unwrap(), 2);
}