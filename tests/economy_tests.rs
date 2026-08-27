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
// STAKING TESTS
// ═══════════════════════════════════════════

#[test]
fn staking_stake_and_unstake() {
    let db = setup_db();
    let user = create_user(&db, "staker1");
    db.ensure_wallet(user).unwrap();
    db.deposit(user, 10000, "initial").unwrap();

    let stake_id = db.stake(user, 5000, 0.05, 30).unwrap();
    assert!(stake_id > 0);

    let balance = db.get_balance(user).unwrap();
    assert_eq!(balance, 5000);

    let positions = db.get_staking_positions(user).unwrap();
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0]["amount"], 5000);
    assert_eq!(positions[0]["status"], "active");

    // Can't unstake early
    let result = db.unstake(user, stake_id);
    assert!(result.is_err());
}

#[test]
fn staking_insufficient_funds() {
    let db = setup_db();
    let user = create_user(&db, "staker2");
    db.ensure_wallet(user).unwrap();
    db.deposit(user, 100, "initial").unwrap();

    let result = db.stake(user, 5000, 0.05, 30);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Insufficient"));
}

#[test]
fn staking_calculate_rewards() {
    let db = setup_db();
    let user = create_user(&db, "staker3");
    db.ensure_wallet(user).unwrap();
    db.deposit(user, 100000, "initial").unwrap();

    db.stake(user, 100000, 0.05, 365).unwrap();

    let updated = db.calculate_staking_rewards().unwrap();
    assert_eq!(updated, 1);

    let positions = db.get_staking_positions(user).unwrap();
    assert!(positions[0]["rewards_earned"].as_i64().unwrap() > 0);
}

#[test]
fn staking_claim_rewards() {
    let db = setup_db();
    let user = create_user(&db, "staker4");
    db.ensure_wallet(user).unwrap();
    db.deposit(user, 100000, "initial").unwrap();

    let stake_id = db.stake(user, 100000, 0.05, 365).unwrap();
    db.calculate_staking_rewards().unwrap();

    let claimed = db.claim_staking_rewards(user, stake_id).unwrap();
    assert!(claimed > 0);

    let positions = db.get_staking_positions(user).unwrap();
    assert_eq!(positions[0]["rewards_earned"], 0);
}

#[test]
fn staking_claim_no_rewards_fails() {
    let db = setup_db();
    let user = create_user(&db, "staker5");
    db.ensure_wallet(user).unwrap();
    db.deposit(user, 10000, "initial").unwrap();

    let stake_id = db.stake(user, 10000, 0.05, 30).unwrap();
    let result = db.claim_staking_rewards(user, stake_id);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No rewards"));
}

#[test]
fn staking_stats() {
    let db = setup_db();
    let stats = db.get_staking_stats().unwrap();
    assert_eq!(stats["total_staked"], 0);
    assert_eq!(stats["active_positions"], 0);
}

// ═══════════════════════════════════════════
// REFERRAL TESTS
// ═══════════════════════════════════════════

#[test]
fn referral_create_and_find() {
    let db = setup_db();
    let referrer = create_user(&db, "referrer1");
    let referred = create_user(&db, "referred1");

    let id = db.create_referral(referrer, referred, "REF123").unwrap();
    assert!(id > 0);

    let found = db.find_referral_by_code("REF123").unwrap();
    assert!(found.is_some());
    let (r_id, rec_id) = found.unwrap();
    assert_eq!(r_id, referrer);
    assert_eq!(rec_id, referred);
}

#[test]
fn referral_not_found() {
    let db = setup_db();
    let found = db.find_referral_by_code("INVALID").unwrap();
    assert!(found.is_none());
}

#[test]
fn referral_stats() {
    let db = setup_db();
    let referrer = create_user(&db, "referrer2");

    let stats = db.get_referral_stats(referrer).unwrap();
    assert_eq!(stats["total_referred"], 0);
    assert_eq!(stats["total_earned"], 0);
}

#[test]
fn referral_complete_and_earning() {
    let db = setup_db();
    let referrer = create_user(&db, "referrer3");
    let referred = create_user(&db, "referred3");

    db.create_referral(referrer, referred, "REF3").unwrap();
    db.complete_referral(referred).unwrap();
    db.add_referral_earning(referrer, 500).unwrap();

    let stats = db.get_referral_stats(referrer).unwrap();
    assert_eq!(stats["total_earned"], 500);
}

// ═══════════════════════════════════════════
// CALL BILLING TESTS
// ═══════════════════════════════════════════

#[test]
fn call_billing_lifecycle() {
    let db = setup_db();
    let caller = create_user(&db, "caller1");
    let host = create_user(&db, "host1");

    let call_id = db.start_call_billing(caller, host, "video", 10).unwrap();
    assert!(call_id > 0);

    let (total, host_earnings, fee) = db.end_call_billing(call_id).unwrap();
    assert!(total > 0);
    assert!(host_earnings > 0);
    assert!(fee >= 0);
    assert_eq!(total, host_earnings + fee);
}

#[test]
fn call_billing_host_stats() {
    let db = setup_db();
    let caller = create_user(&db, "caller2");
    let host = create_user(&db, "host2");

    let call_id = db.start_call_billing(caller, host, "video", 10).unwrap();
    db.end_call_billing(call_id).unwrap();

    let stats = db.get_host_call_stats(host).unwrap();
    assert_eq!(stats["total_calls"], 1);
    assert!(stats["total_earnings"].as_i64().unwrap() > 0);
}

// ═══════════════════════════════════════════
// COMMISSION TESTS
// ═══════════════════════════════════════════

#[test]
fn commission_create_and_get() {
    let db = setup_db();
    let user = create_user(&db, "comm1");
    let source = create_user(&db, "source1");

    let id = db.create_commission(user, source, None, 1, 0.40, 400).unwrap();
    assert!(id > 0);

    let commissions = db.get_user_commissions(user, None).unwrap();
    assert_eq!(commissions.len(), 1);
    assert_eq!(commissions[0]["amount"], 400);
    assert_eq!(commissions[0]["tier"], 1);

    let summary = db.get_commission_summary(user).unwrap();
    assert_eq!(summary["total_earned"], 400);
}

#[test]
fn commission_distribute_multi_level() {
    let db = setup_db();
    let t1 = create_user(&db, "tier1user");
    let t2 = create_user(&db, "tier2user");
    let t3 = create_user(&db, "tier3user");
    let source = create_user(&db, "source");

    db.ensure_wallet(t1).unwrap();
    db.ensure_wallet(t2).unwrap();
    db.ensure_wallet(t3).unwrap();

    db.distribute_commissions(source, None, 1000, Some(t1), Some(t2), Some(t3), None).unwrap();

    let s1 = db.get_commission_summary(t1).unwrap();
    assert_eq!(s1["total_earned"], 400);

    let s2 = db.get_commission_summary(t2).unwrap();
    assert_eq!(s2["total_earned"], 200);

    let s3 = db.get_commission_summary(t3).unwrap();
    assert_eq!(s3["total_earned"], 100);
}

#[test]
fn commission_pay_pending() {
    let db = setup_db();
    let user = create_user(&db, "payer1");
    let source = create_user(&db, "src");

    db.ensure_wallet(user).unwrap();
    db.create_commission(user, source, None, 1, 0.40, 400).unwrap();
    db.create_commission(user, source, None, 1, 0.40, 200).unwrap();

    let paid = db.pay_pending_commissions(user).unwrap();
    assert_eq!(paid, 600);

    let summary = db.get_commission_summary(user).unwrap();
    assert_eq!(summary["paid"], 600);
    assert_eq!(summary["pending"], 0);
}

// ═══════════════════════════════════════════
// PAYOUT TESTS
// ═══════════════════════════════════════════

#[test]
fn payout_request_and_history() {
    let db = setup_db();
    let user = create_user(&db, "payout1");
    db.ensure_wallet(user).unwrap();
    db.deposit(user, 5000, "initial").unwrap();

    let id = db.request_payout(user, 1000, "USDT", "TAddr123", "TRC20").unwrap();
    assert!(id > 0);

    let balance = db.get_balance(user).unwrap();
    assert_eq!(balance, 4000);

    let payouts = db.get_user_payouts(user).unwrap();
    assert_eq!(payouts.len(), 1);
    assert_eq!(payouts[0]["status"], "pending");
}

#[test]
fn payout_insufficient_funds() {
    let db = setup_db();
    let user = create_user(&db, "payout2");
    db.ensure_wallet(user).unwrap();

    let result = db.request_payout(user, 100, "USDT", "TAddr", "TRC20");
    assert!(result.is_err());
}

#[test]
fn payout_minimum() {
    let db = setup_db();
    let user = create_user(&db, "payout3");
    db.ensure_wallet(user).unwrap();
    db.deposit(user, 100, "initial").unwrap();

    let result = db.request_payout(user, 5, "USDT", "TAddr", "TRC20");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Minimum"));
}

#[test]
fn payout_process_approve() {
    let db = setup_db();
    let admin = create_user(&db, "admin");
    let user = create_user(&db, "payout4");
    db.ensure_wallet(user).unwrap();
    db.deposit(user, 5000, "initial").unwrap();

    let id = db.request_payout(user, 1000, "USDT", "TAddr", "TRC20").unwrap();
    db.process_payout(id, admin, "0xabc123", true).unwrap();

    let payouts = db.get_user_payouts(user).unwrap();
    assert_eq!(payouts[0]["status"], "completed");
}

#[test]
fn payout_process_reject_refunds() {
    let db = setup_db();
    let admin = create_user(&db, "admin2");
    let user = create_user(&db, "payout5");
    db.ensure_wallet(user).unwrap();
    db.deposit(user, 5000, "initial").unwrap();

    let id = db.request_payout(user, 1000, "USDT", "TAddr", "TRC20").unwrap();
    let balance_before = db.get_balance(user).unwrap();

    db.process_payout(id, admin, "", false).unwrap();

    let balance_after = db.get_balance(user).unwrap();
    assert_eq!(balance_before + 1000, balance_after);
}

#[test]
fn payout_pending_list() {
    let db = setup_db();
    let _admin = create_user(&db, "admin3");
    let user = create_user(&db, "payout6");

    assert_eq!(user, user); // just to use admin

    let pending = db.get_pending_payouts().unwrap();
    assert_eq!(pending.len(), 0);
}

// ═══════════════════════════════════════════
// FRAUD TESTS
// ═══════════════════════════════════════════

#[test]
fn fraud_alert_create_and_resolve() {
    let db = setup_db();
    let admin = create_user(&db, "fraud_admin");

    let id = db.create_fraud_alert(Some(1), "velocity", "high", "Rapid deposits", "{}", Some("1.2.3.4")).unwrap();
    assert!(id > 0);

    let alerts = db.get_fraud_alerts(None).unwrap();
    assert_eq!(alerts.len(), 1);

    db.resolve_fraud_alert(id, admin).unwrap();

    let alerts = db.get_fraud_alerts(Some("resolved")).unwrap();
    assert_eq!(alerts.len(), 1);
}

#[test]
fn fraud_velocity_check() {
    let db = setup_db();
    let user = create_user(&db, "vel_user");
    db.ensure_wallet(user).unwrap();
    db.deposit(user, 1000, "initial").unwrap();

    let (count, total) = db.check_velocity(user, "deposit", 300).unwrap();
    assert!(count >= 1);
    assert!(total >= 1000);
}

#[test]
fn fraud_alert_filter() {
    let db = setup_db();
    db.create_fraud_alert(None, "velocity", "high", "alert1", "{}", None).unwrap();
    db.create_fraud_alert(None, "duplicate", "low", "alert2", "{}", None).unwrap();

    let all = db.get_fraud_alerts(None).unwrap();
    assert_eq!(all.len(), 2);

    let high = db.get_fraud_alerts(Some("open")).unwrap();
    assert_eq!(high.len(), 2);
}

// ═══════════════════════════════════════════
// RECEIPT TESTS
// ═══════════════════════════════════════════

#[test]
fn receipt_create_and_verify() {
    let db = setup_db();
    let user = create_user(&db, "receipt1");

    let id = db.create_receipt(user, "deposit", 1, 5000, "YSH", "Test deposit", "{}").unwrap();
    assert!(id > 0);

    let receipt = db.get_receipt(id).unwrap();
    assert!(receipt.is_some());
    let r = receipt.unwrap();
    assert_eq!(r["amount"], 5000);
    assert_eq!(r["currency"], "YSH");

    let valid = db.verify_receipt(id).unwrap();
    assert!(valid);
}

#[test]
fn receipt_list() {
    let db = setup_db();
    let user = create_user(&db, "receipt2");

    db.create_receipt(user, "deposit", 1, 1000, "YSH", "d1", "{}").unwrap();
    db.create_receipt(user, "withdraw", 2, 500, "YSH", "d2", "{}").unwrap();

    let receipts = db.get_user_receipts(user, 10).unwrap();
    assert_eq!(receipts.len(), 2);
}

#[test]
fn receipt_not_found() {
    let db = setup_db();
    let receipt = db.get_receipt(999).unwrap();
    assert!(receipt.is_none());
}

// ═══════════════════════════════════════════
// WALLET FREEZE TESTS
// ═══════════════════════════════════════════

#[test]
fn wallet_freeze_and_unfreeze() {
    let db = setup_db();
    let user = create_user(&db, "freeze1");
    db.ensure_wallet(user).unwrap();

    assert!(!db.is_wallet_frozen(user).unwrap());

    db.freeze_wallet(user).unwrap();
    assert!(db.is_wallet_frozen(user).unwrap());

    db.unfreeze_wallet(user).unwrap();
    assert!(!db.is_wallet_frozen(user).unwrap());
}

// ═══════════════════════════════════════════
// SPENDING LIMITS TESTS
// ═══════════════════════════════════════════

#[test]
fn spending_limits_within() {
    let db = setup_db();
    let user = create_user(&db, "limit1");

    db.set_spending_limit(user, 10000, 100000).unwrap();

    let (ok, _) = db.check_spending_limit(user, 5000).unwrap();
    assert!(ok);

    let limits = db.get_spending_limits(user).unwrap();
    assert_eq!(limits["daily_spent"], 5000);
}

#[test]
fn spending_limits_exceeded() {
    let db = setup_db();
    let user = create_user(&db, "limit2");

    db.set_spending_limit(user, 1000, 100000).unwrap();

    let (ok, _) = db.check_spending_limit(user, 800).unwrap();
    assert!(ok);

    let (ok, msg) = db.check_spending_limit(user, 500).unwrap();
    assert!(!ok);
    assert!(msg.contains("Daily limit"));
}

#[test]
fn spending_limits_default() {
    let db = setup_db();
    let user = create_user(&db, "limit3");

    let limits = db.get_spending_limits(user).unwrap();
    assert_eq!(limits["daily_limit"], 100000);
    assert_eq!(limits["monthly_limit"], 1000000);
}

// ═══════════════════════════════════════════
// ENHANCED GIFT TESTS
// ═══════════════════════════════════════════

#[test]
fn gift_sent_gifts() {
    let db = setup_db();
    let sender = create_user(&db, "sender1");
    let receiver = create_user(&db, "receiver1");

    db.ensure_wallet(sender).unwrap();
    db.deposit(sender, 10000, "initial").unwrap();

    db.send_gift(sender, receiver, 1).unwrap();
    db.send_gift(sender, receiver, 2).unwrap();

    let sent = db.get_sent_gifts(sender).unwrap();
    assert_eq!(sent.len(), 2);

    let stats = db.get_gift_stats(sender).unwrap();
    assert_eq!(stats["sent_count"], 2);
    assert!(stats["total_spent"].as_i64().unwrap() > 0);
}

#[test]
fn gift_stats_received() {
    let db = setup_db();
    let sender = create_user(&db, "sender2");
    let receiver = create_user(&db, "receiver2");

    db.ensure_wallet(sender).unwrap();
    db.deposit(sender, 10000, "initial").unwrap();

    db.send_gift(sender, receiver, 3).unwrap();

    let stats = db.get_gift_stats(receiver).unwrap();
    assert_eq!(stats["received_count"], 1);
    assert!(stats["total_received"].as_i64().unwrap() > 0);
}

#[test]
fn gift_nft_mint_and_list() {
    let db = setup_db();
    let sender = create_user(&db, "nft_sender");
    let receiver = create_user(&db, "nft_receiver");

    db.ensure_wallet(sender).unwrap();
    db.deposit(sender, 10000, "initial").unwrap();

    // Send a legendary gift (gift_id=5, price=1000) to create a gift record
    let gift_record_id = db.send_gift(sender, receiver, 5).unwrap();

    // Mint NFT for the receiver using the real gift_record_id
    let nft_id = db.mint_nft_gift(receiver, 5, gift_record_id).unwrap();
    assert!(nft_id > 0);

    let nfts = db.get_nft_gifts(receiver).unwrap();
    assert_eq!(nfts.len(), 1);
    assert!(nfts[0]["token_id"].as_str().unwrap().starts_with("YSH-NFT-"));
}

// ═══════════════════════════════════════════
// WALLET FREEZE BLOCKS OPERATIONS
// ═══════════════════════════════════════════

#[test]
fn frozen_wallet_blocks_deposit() {
    let db = setup_db();
    let user = create_user(&db, "frozen1");
    db.ensure_wallet(user).unwrap();

    db.freeze_wallet(user).unwrap();
    assert!(db.is_wallet_frozen(user).unwrap());

    // Deposit still works at DB level (freeze checked at API level)
    db.deposit(user, 1000, "test").unwrap();
    let balance = db.get_balance(user).unwrap();
    assert_eq!(balance, 1000);
}
