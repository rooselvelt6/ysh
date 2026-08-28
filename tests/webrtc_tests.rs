use std::sync::Arc;
use ysh::db::Database;
use ysh::webrtc::{CallType, RoomManager};

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

fn mgr() -> RoomManager {
    RoomManager::new(2, 3, 8, 1000)
}

// ═══════════════════════════════════════════
// CALL RECORDS
// ═══════════════════════════════════════════

#[test]
fn call_record_roundtrip() {
    let db = setup_db();
    let alice = create_user(&db, "alice");
    let bob = create_user(&db, "bob");

    db.create_call_record("c1", "c1", alice, "p2p", &[alice, bob]).unwrap();
    let rec = db.get_call_record("c1").unwrap().unwrap();
    assert_eq!(rec.call_type, "p2p");
    assert_eq!(rec.participants, vec![alice, bob]);
    assert_eq!(rec.status, "ringing");
    assert!(db.get_call_record("nope").unwrap().is_none());
}

#[test]
fn join_flips_ringing_to_active() {
    let db = setup_db();
    let alice = create_user(&db, "alice");
    let bob = create_user(&db, "bob");
    db.create_call_record("c1", "c1", alice, "duo", &[alice]).unwrap();

    assert!(db.join_call("c1", bob).unwrap());
    let rec = db.get_call_record("c1").unwrap().unwrap();
    assert_eq!(rec.status, "active");
    assert!(rec.participants.contains(&bob));

    db.join_call("c1", bob).unwrap();
    let rec = db.get_call_record("c1").unwrap().unwrap();
    assert_eq!(rec.participants.len(), 2, "duplicate joins are ignored");
}

#[test]
fn leave_call_removes_participant() {
    let db = setup_db();
    let alice = create_user(&db, "alice");
    let bob = create_user(&db, "bob");
    db.create_call_record("c1", "c1", alice, "p2p", &[alice, bob]).unwrap();

    db.leave_call("c1", bob).unwrap();
    let rec = db.get_call_record("c1").unwrap().unwrap();
    assert!(!rec.participants.contains(&bob));
}

#[test]
fn screen_share_toggle() {
    let db = setup_db();
    let alice = create_user(&db, "alice");
    let bob = create_user(&db, "bob");
    db.create_call_record("c1", "c1", alice, "group", &[alice, bob]).unwrap();

    assert!(db.set_call_screen_share("c1", bob, true).unwrap());
    let rec = db.get_call_record("c1").unwrap().unwrap();
    assert!(rec.screen_share);
    assert_eq!(rec.screen_share_user, Some(bob));

    assert!(db.set_call_screen_share("c1", bob, false).unwrap());
    let rec = db.get_call_record("c1").unwrap().unwrap();
    assert!(!rec.screen_share);
}

#[test]
fn end_call_record_computes_duration() {
    let db = setup_db();
    let alice = create_user(&db, "alice");
    db.create_call_record("c1", "c1", alice, "p2p", &[alice]).unwrap();
    db.set_call_recording("c1", true, true).unwrap();

    let duration = db.end_call_record("c1").unwrap();
    assert!(duration >= 0);
    let rec = db.get_call_record("c1").unwrap().unwrap();
    assert_eq!(rec.status, "ended");
    assert!(!rec.recording);
    assert!(rec.ended_at.is_some());

    let again = db.end_call_record("c1").unwrap();
    assert_eq!(again, duration, "idempotent end");
}

#[test]
fn call_history_returns_joined_calls() {
    let db = setup_db();
    let alice = create_user(&db, "alice");
    let bob = create_user(&db, "bob");
    db.create_call_record("c1", "c1", alice, "p2p", &[alice, bob]).unwrap();
    db.create_call_record("c2", "c2", alice, "duo", &[alice, bob]).unwrap();

    let history = db.get_call_history(alice, 100).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0]["call_id"], "c2", "sorted newest first");
    assert_eq!(history[0]["call_type"], "duo");
}

#[test]
fn call_stats_counts_active_and_total() {
    let db = setup_db();
    let alice = create_user(&db, "alice");
    db.create_call_record("c1", "c1", alice, "p2p", &[alice]).unwrap();
    db.create_call_record("c2", "c2", alice, "live", &[alice]).unwrap();
    db.end_call_record("c1").unwrap();

    let stats = db.get_call_stats().unwrap();
    assert_eq!(stats["total_calls"], 2);
    assert_eq!(stats["active_calls"], 1);
}

// ═══════════════════════════════════════════
// QUALITY METRICS
// ═══════════════════════════════════════════

#[test]
fn quality_samples_and_aggregation() {
    let db = setup_db();
    let alice = create_user(&db, "alice");
    db.add_quality_sample("c1", alice, 1500.0, 0.5, 40.0, "1280x720", "h").unwrap();
    db.add_quality_sample("c1", alice, 2500.0, 1.5, 60.0, "1280x720", "f").unwrap();

    let samples = db.get_quality_metrics("c1").unwrap();
    assert_eq!(samples.len(), 2);

    let agg = db.aggregate_quality("c1").unwrap();
    assert_eq!(agg["samples"], 2);
    assert_eq!(agg["avg_bitrate_kbps"], 2000.0);
    assert_eq!(agg["avg_packet_loss_pct"], 1.0);
    assert_eq!(agg["avg_rtt_ms"], 50.0);
}

// ═══════════════════════════════════════════
// RECORDINGS (opt-in, encrypted)
// ═══════════════════════════════════════════

#[test]
fn recording_lifecycle() {
    let db = setup_db();
    let alice = create_user(&db, "alice");
    db.create_call_record("c1", "c1", alice, "group", &[alice]).unwrap();

    let seg = db.start_call_recording("c1", "enc://recordings/c1/1", true, 0).unwrap();
    assert!(seg > 0);

    let rec = db.get_call_record("c1").unwrap().unwrap();
    assert!(rec.recording);
    assert!(rec.recording_encrypted);

    db.finalize_call_recording("c1", seg).unwrap();
    let segments = db.list_call_recordings("c1").unwrap();
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].status, "finalized");
    assert!(segments[0].encrypted);

    let rec = db.get_call_record("c1").unwrap().unwrap();
    assert!(!rec.recording, "finalize clears recording flag");
}

// ═══════════════════════════════════════════
// BILLING POR DURACIÓN + WALLET DEBIT
// ═══════════════════════════════════════════

#[test]
fn billing_payment_debits_caller_credits_host() {
    let db = setup_db();
    let caller = create_user(&db, "caller");
    let host = create_user(&db, "host");
    db.deposit(caller, 5000, "seed").unwrap();
    db.deposit(host, 100, "seed").unwrap();

    let billing_id = db.start_call_billing(caller, host, "flash", 100).unwrap();
    let (total, host_earnings, platform_fee) = db.end_call_billing(billing_id).unwrap();
    assert_eq!(total, 100, "min 1 minute at 100/min");
    assert_eq!(host_earnings + platform_fee, total);
    assert_eq!(host_earnings, 70);

    let res = db.finalize_call_payment(billing_id).unwrap();
    assert_eq!(res["caller_balance"], 4900);
    assert_eq!(res["host_earnings"], 70);

    let caller_bal = db.get_balance(caller).unwrap();
    let host_bal = db.get_balance(host).unwrap();
    assert_eq!(caller_bal, 4900);
    assert_eq!(host_bal, 170);
}

#[test]
fn billing_payment_is_idempotent_single_charge() {
    let db = setup_db();
    let caller = create_user(&db, "caller");
    let host = create_user(&db, "host");
    db.deposit(caller, 5000, "seed").unwrap();

    let billing_id = db.start_call_billing(caller, host, "p2p", 100).unwrap();
    db.end_call_billing(billing_id).unwrap();
    db.finalize_call_payment(billing_id).unwrap();

    let res = db.finalize_call_payment(billing_id);
    assert!(res.is_err(), "second payment must fail");
    assert_eq!(db.get_balance(caller).unwrap(), 4900, "no double charge");
}

#[test]
fn billing_payment_rejects_when_insufficient_balance() {
    let db = setup_db();
    let caller = create_user(&db, "caller");
    let host = create_user(&db, "host");
    db.deposit(caller, 50, "seed").unwrap();

    let billing_id = db.start_call_billing(caller, host, "p2p", 100).unwrap();
    db.end_call_billing(billing_id).unwrap();
    assert!(db.finalize_call_payment(billing_id).is_err());
}

// ═══════════════════════════════════════════
// FLASH / RANDOM PEER + ROOM COORDINATION
// ═══════════════════════════════════════════

#[test]
fn find_random_peer_excludes_self_and_busy_users() {
    let db = setup_db();
    let alice = create_user(&db, "alice");
    let bob = create_user(&db, "bob");

    assert_eq!(db.find_random_peer(alice).unwrap(), Some(bob));
    assert_eq!(db.find_random_peer(bob).unwrap(), Some(alice));

    db.create_call_record("busy", "busy", bob, "p2p", &[bob]).unwrap();
    assert_eq!(db.find_random_peer(alice).unwrap(), None, "bob is in a call");
}

#[test]
fn room_start_join_leave_lifecycle_via_manager() {
    let mut m = mgr();
    m.create_room("live1", CallType::Live, 1, Some("Mi stream".into()));
    assert!(m.join("live1", 2).accepted);
    assert!(m.join("live1", 3).accepted);
    assert_eq!(m.get_room("live1").unwrap().viewers.len(), 2);

    let out = m.leave("live1", 2).unwrap();
    assert!(!out.room_empty);
    assert_eq!(out.viewer_count, 1);

    let live = m.list_live();
    assert_eq!(live[0]["title"], "Mi stream");
    assert_eq!(live[0]["viewers"], 1);
}

#[test]
fn room_group_capacity_enforced() {
    let mut m = mgr();
    m.create_room("g1", CallType::Group, 1, None);
    for u in 2..=8 {
        assert!(m.join("g1", u).accepted);
    }
    assert!(!m.join("g1", 9).accepted);
    assert_eq!(m.get_room("g1").unwrap().participants.len(), 8);
}

#[test]
fn live_viewer_cap_respected() {
    let mut m = RoomManager::new(2, 3, 8, 5);
    m.create_room("l1", CallType::Live, 1, None);
    for v in 10..15 {
        assert!(m.join("l1", v).accepted);
    }
    assert!(!m.join("l1", 15).accepted);
}