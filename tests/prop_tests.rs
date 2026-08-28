// PROPERTY-BASED TESTING (proptest)
//
// Verifies invariants that must hold for arbitrary inputs:
//  - Staking interest never goes negative and respects an upper bound.
//  - Moderation resolution is deterministic and idempotent.
//  - Serde roundtrips for core economy records.
//  - Wallet arithmetic never overflows / underflows from random amounts.
use proptest::prelude::*;
use std::sync::Arc;
use ysh::db::Database;

fn setup_db() -> Arc<Database> {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
    std::mem::forget(tmp);
    Arc::new(db)
}

fn create_user(db: &Database, username: &str) -> i64 {
    db.create_user(username, &format!("{}@prop.com", username), "hash").unwrap().id
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn staking_interest_bounded_and_non_negative(
        amount in 1i64..1_000_000,
        apy in 0.0f64..50.0,
        days in 1u32..1095,
    ) {
        let db = setup_db();
        let u = create_user(&db, "prop_stake");
        db.deposit(u, 2_000_000, "seed").unwrap();
        let sid = db.stake(u, amount, apy, 30).unwrap();
        let old = (chrono::Utc::now() - chrono::Duration::days(i64::from(days)))
            .format("%Y-%m-%dT%H:%M:%SZ").to_string();
        db.set_staking_recalc(sid, &old).unwrap();
        let res = db.compute_staking_interest().unwrap();
        let interest = res["total_interest"].as_i64().unwrap();
        prop_assert!(interest >= 0, "negative interest accrued");
        // Simple interest over the period must not exceed a generous cap:
        // amount * apy/100 * days/365, plus 10% slop for floating error.
        let expected = (amount as f64) * (apy / 100.0) * (days as f64 / 365.0);
        prop_assert!(interest as f64 <= expected * 1.10 + 1.0,
            "interest {} exceeded bound {}", interest, expected);
    }

    #[test]
    fn moderation_resolution_idempotent(sev in 0.0f64..1.0f64) {
        let db = setup_db();
        db.enqueue_moderation_item("chat", 1, sev, "p").unwrap();
        db.age_moderation_items(10 * 86400).unwrap();
        let r1 = db.auto_resolve_moderation(7 * 86400, 0.4, 0.8).unwrap();
        let r2 = db.auto_resolve_moderation(7 * 86400, 0.4, 0.8).unwrap();
        // Mid-severity items are intentionally kept for human review.
        let expected_first = if sev <= 0.4 { 1 } else if sev >= 0.8 { 1 } else { 0 };
        let d1 = r1["dismissed"].as_i64().unwrap() + r1["actioned"].as_i64().unwrap();
        prop_assert_eq!(d1, expected_first);
        let d2 = r2["dismissed"].as_i64().unwrap() + r2["actioned"].as_i64().unwrap();
        prop_assert_eq!(d2, 0, "auto-resolve is not idempotent");
    }

    #[test]
    fn wallet_balance_never_overflows(
        initial in 1i64..1_000_000,
        deposit in 1i64..1_000_000,
        withdrawal in 1i64..1_000_000,
    ) {
        let db = setup_db();
        let u = create_user(&db, "prop_wallet");
        db.deposit(u, initial, "seed").unwrap();
        let _ = db.deposit(u, deposit, "d").unwrap();
        let total = initial + deposit;
        let w = withdrawal.min(total);
        let bal = db.withdraw(u, w, "x").unwrap();
        prop_assert_eq!(bal, total - w);
        prop_assert!(bal >= 0);
    }

    #[test]
    fn notification_inapp_always_delivered(n in 0u32..6) {
        let db = setup_db();
        let u = create_user(&db, "prop_notif");
        // Pre-existing push notification that may absorb the flush budget.
        db.create_notification(u, "x", "t", "b", "{}", "push").unwrap();
        for _ in 0..n {
            let _ = db.flush_pending_notifications().unwrap();
        }
        // A fresh in-app notification must always be delivered on the next sweep.
        db.create_notification(u, "x", "t", "b", "{}", "inapp").unwrap();
        let res = db.flush_pending_notifications().unwrap();
        prop_assert!(res["sent"].as_i64().unwrap() >= 1, "inapp notif should be sent");
        let notifs = db.get_notifications(u, 10).unwrap();
        let inapp = notifs.iter().find(|x| x["channel"] == "inapp").unwrap();
        prop_assert_eq!(inapp["status"].as_str(), Some("sent"));
    }
}

#[test]
fn econ_records_roundtrip_serde() {
    use ysh::db::{Payout, Staking, Transaction};
    let db = setup_db();
    let u = create_user(&db, "rt");

    let tx = Transaction {
        id: 7,
        user_id: u,
        tx_type: "deposit".into(),
        amount: 1234,
        description: "roundtrip".into(),
        target_user_id: Some(u),
        created_at: "2026-01-01T00:00:00Z".into(),
    };
    let json = serde_json::to_string(&tx).unwrap();
    let back: Transaction = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, 7);
    assert_eq!(back.amount, 1234);

    let st = Staking {
        id: 1, user_id: u, amount: 5000, apy_rate: 0.05, status: "active".into(),
        staked_at: "2026-01-01T00:00:00Z".into(), unlocks_at: "2026-06-01T00:00:00Z".into(),
        rewards_earned: 0, last_reward_calc: "2026-01-01T00:00:00Z".into(),
    };
    let back_st: Staking = serde_json::from_str(&serde_json::to_string(&st).unwrap()).unwrap();
    assert_eq!(back_st.amount, 5000);

    let p = Payout {
        id: 1, user_id: u, amount: 100, currency: "YSH".into(), wallet_address: "0x1".into(),
        network: "eth".into(), status: "pending".into(), tx_hash: None, requested_at: "t".into(),
        processed_at: None, admin_id: None, notes: String::new(),
    };
    let back_p: Payout = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
    assert!(back_p.tx_hash.is_none());

    // Older stored users missing the `region` field must still deserialize.
    let legacy = r#"{"id":1,"username":"old","email":"a@b","password_hash":"h","role":"user",
        "created_at":"t","failed_login_attempts":0,"locked_until":null,"totp_secret":null,
        "totp_enabled":false,"kyc_level":0,"do_not_sell":false}"#;
    let old_user: ysh::db::User = serde_json::from_str(legacy).unwrap();
    assert_eq!(old_user.region, "");
}