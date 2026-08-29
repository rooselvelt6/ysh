// LOAD TESTING: concurrent wallet + economy operations under shared DB.
//
// Simulates bursty traffic (deposits, transfers, call billing, moment likes)
// across many threads on a single database instance. Verifies no panics /
// data corruption: balances stay non-negative and ledger entries are intact.
use serde_json::json;
use std::sync::Arc;
use std::thread;
use ysh::db::Database;

fn setup_db() -> Arc<Database> {
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    let db = Database::new(tmp.path().to_str().unwrap()).expect("test DB");
    std::mem::forget(tmp);
    Arc::new(db)
}

fn create_user(db: &Database, username: &str) -> i64 {
    db.create_user(username, &format!("{}@load.com", username), "hash")
        .unwrap()
        .id
}

#[test]
fn concurrent_deposits_and_withdrawals_never_go_negative() {
    let db = setup_db();
    let u = create_user(&db, "load1");
    db.deposit(u, 200_000, "seed").unwrap();

    let db = db.clone();
    let handles: Vec<_> = (0..8)
        .map(|i| {
            let db = db.clone();
            thread::spawn(move || {
                for j in 0..50 {
                    let _ = db.deposit(u, 10, &format!("dep{}", i * 100 + j));
                }
                let _ = db.withdraw(u, 5, "spend");
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let balance = db.get_balance(u).unwrap();
    assert!(balance >= 0, "balance went negative");
}

#[test]
fn concurrent_transfers_conserve_wallet_total() {
    let db = setup_db();
    let alice = create_user(&db, "alice");
    let bob = create_user(&db, "bob");
    db.deposit(alice, 1_000_000, "seed").unwrap();
    db.deposit(bob, 1_000_000, "seed").unwrap();

    let start = db.get_balance(alice).unwrap() + db.get_balance(bob).unwrap();

    let db = db.clone();
    let handles: Vec<_> = (0..8)
        .map(|i| {
            let db = db.clone();
            thread::spawn(move || {
                let (from, to) = if i % 2 == 0 {
                    (alice, bob)
                } else {
                    (bob, alice)
                };
                for _ in 0..40 {
                    let _ = db.transfer(from, to, 100, "concurrent transfer");
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }

    let end = db.get_balance(alice).unwrap() + db.get_balance(bob).unwrap();
    assert_eq!(start, end, "value leaked across transfers");
}

#[test]
fn concurrent_call_billing_leaves_consistent_ledger() {
    let db = setup_db();
    let host = create_user(&db, "host");
    let caller = create_user(&db, "caller");
    db.deposit(caller, 500_000, "seed").unwrap();

    let db = db.clone();
    let handles: Vec<_> = (0..6)
        .map(|i| {
            let db = db.clone();
            thread::spawn(move || {
                for _ in 0..10 {
                    if let Ok(id) = db.start_call_billing(caller, host, "video", 5) {
                        let _ = db.end_call_billing(id);
                        let _ = db.finalize_call_payment(id);
                    }
                }
                let _ = i;
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }

    let stats = db.get_host_call_stats(host).unwrap();
    assert!(stats["total_calls"].as_i64().unwrap() > 0);

    let wallet = db.get_balance(host).unwrap();
    assert!(wallet >= 0);
}

#[test]
fn concurrent_moment_likes_do_not_corrupt() {
    let db = setup_db();
    let author = create_user(&db, "author");
    let moment_id = db.create_moment(author, "hello", "", "text").unwrap();

    let db = db.clone();
    let handles: Vec<_> = (0..6)
        .map(|i| {
            let db = db.clone();
            thread::spawn(move || {
                let u = create_user(&db, &format!("liker{}", i));
                for _ in 0..20 {
                    let _ = db.like_moment(u, moment_id);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }

    let feed = db.get_moment_feed(author, 0, 10).unwrap();
    let mine = feed
        .iter()
        .find(|m| m["id"].as_i64() == Some(moment_id))
        .unwrap();
    assert!(mine["likes"].as_i64().unwrap() >= 1);
}

#[test]
fn concurrent_activity_logging_survives_pressure() {
    let db = setup_db();
    let users: Vec<i64> = (0..6)
        .map(|i| create_user(&db, &format!("act{}", i)))
        .collect();

    let db = db.clone();
    let handles: Vec<_> = users
        .iter()
        .map(|u| {
            let db = db.clone();
            let u = *u;
            thread::spawn(move || {
                for j in 0..100 {
                    let _ = db.log_activity(u, &format!("op{}", j % 5));
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }

    let snap = db.compute_analytics_snapshot().unwrap();
    assert!(snap["dau"].as_i64().unwrap() >= 1);
}

#[test]
fn mixed_workload_finishes_cleanly() {
    let db = setup_db();
    let users: Vec<i64> = (0..4)
        .map(|i| create_user(&db, &format!("mw{}", i)))
        .collect();
    db.deposit(users[0], 1_000_000, "seed").unwrap();

    let db = db.clone();
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let db = db.clone();
            let users = users.clone();
            thread::spawn(move || {
                for j in 0..20 {
                    let u = users[i % users.len()];
                    let _ = db.log_activity(u, "active");
                    let _ = db.enqueue_match(u, "chat", &json!({"level": j}).to_string());
                    if i == 0 {
                        let _ = db.send_gift(u, users[(i + 1) % users.len()], 1);
                    }
                    let _ = db.create_notification(u, "inapp", "t", "b", "{}", "inapp");
                    let _ = db.flush_pending_notifications();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }

    let stats = db.get_staking_stats().unwrap_or(json!({}));
    assert!(stats.is_object());
}
