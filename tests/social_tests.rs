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
// USER BLOCKS
// ═══════════════════════════════════════════

#[test]
fn block_unblock_and_checks() {
    let db = setup_db();
    let alice = create_user(&db, "alice");
    let bob = create_user(&db, "bob");

    assert!(!db.is_blocked(alice, bob).unwrap());

    db.block_user(alice, bob).unwrap();
    assert!(db.is_blocked(alice, bob).unwrap());

    let blocked = db.get_blocked_users(alice).unwrap();
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0].blocked_user_id, bob);

    assert!(db.unblock_user(alice, bob).unwrap());
    assert!(!db.is_blocked(alice, bob).unwrap());
}

#[test]
fn block_self_is_rejected() {
    let db = setup_db();
    let alice = create_user(&db, "alice");
    let result = db.block_user(alice, alice);
    assert!(result.is_err());
}

#[test]
fn block_unknown_user_is_rejected() {
    let db = setup_db();
    let alice = create_user(&db, "alice");
    let result = db.block_user(alice, 9999);
    assert!(result.is_err());
}

#[test]
fn feed_hides_blocked_authors() {
    let db = setup_db();
    let author = create_user(&db, "author");
    let viewer = create_user(&db, "viewer");

    let mid = db.create_moment(author, "Hello world", "", "text").unwrap();
    let feed_before = db.get_moment_feed(viewer, 0, 10).unwrap();
    assert!(
        feed_before
            .iter()
            .any(|m| m["id"] == serde_json::json!(mid))
    );

    db.block_user(viewer, author).unwrap();
    let feed_after = db.get_moment_feed(viewer, 0, 10).unwrap();
    assert!(!feed_after.iter().any(|m| m["id"] == serde_json::json!(mid)));
}

// ═══════════════════════════════════════════
// RATINGS + REPUTATION
// ═══════════════════════════════════════════

#[test]
fn rates_and_recomputes_reputation() {
    let db = setup_db();
    let sam = create_user(&db, "sam");
    let r1 = create_user(&db, "rater1");
    let r2 = create_user(&db, "rater2");

    db.rate_user(r1, sam, 5.0).unwrap();
    db.rate_user(r2, sam, 3.0).unwrap();

    let rep = db.get_reputation(sam).unwrap();
    assert_eq!(rep.rating_count, 2);
    assert_eq!(rep.rating_avg, 4.0);

    // re-rate overwrites
    db.rate_user(r1, sam, 1.0).unwrap();
    let rep = db.get_reputation(sam).unwrap();
    assert_eq!(rep.rating_count, 2);
    assert_eq!(rep.rating_avg, 2.0);
}

#[test]
fn rating_validation() {
    let db = setup_db();
    let sam = create_user(&db, "sam");
    let rater = create_user(&db, "rater");

    assert!(db.rate_user(sam, sam, 5.0).is_err());
    assert!(db.rate_user(rater, sam, 6.0).is_err());
    assert!(db.rate_user(rater, sam, 0.0).is_err());
}

// ═══════════════════════════════════════════
// VERIFICATION BADGES
// ═══════════════════════════════════════════

#[test]
fn grant_revoke_badges() {
    let db = setup_db();
    let user = create_user(&db, "badge_user");

    let badge_id = db.grant_badge(user, "email_verified").unwrap();
    assert!(badge_id > 0);
    assert!(db.has_badge(user, "email_verified").unwrap());

    let badges = db.get_user_badges(user).unwrap();
    assert_eq!(badges.len(), 1);
    assert_eq!(badges[0].badge_type, "email_verified");

    // duplicate rejected
    assert!(db.grant_badge(user, "email_verified").is_err());

    assert!(db.revoke_badge(user, "email_verified").unwrap());
    assert!(!db.has_badge(user, "email_verified").unwrap());
}

// ═══════════════════════════════════════════
// USER REPORTS
// ═══════════════════════════════════════════

#[test]
fn reports_flow_and_counting() {
    let db = setup_db();
    let reporter = create_user(&db, "rpt_reporter");
    let reported = create_user(&db, "rpt_target");

    let rid = db
        .create_report(reporter, "user", reported, "scam", "Scam attempt")
        .unwrap();
    assert!(rid > 0);

    assert_eq!(db.count_open_reports_for("user", reported).unwrap(), 1);
    assert_eq!(db.distinct_reporters_for("user", reported).unwrap(), 1);

    let pending = db.get_reports(Some("pending")).unwrap();
    assert_eq!(pending.len(), 1);

    db.resolve_report(rid, reporter, "actioned").unwrap();
    assert_eq!(db.get_reports(Some("pending")).unwrap().len(), 0);
    assert_eq!(db.get_reports(Some("actioned")).unwrap().len(), 1);
    assert_eq!(db.count_open_reports_for("user", reported).unwrap(), 1);
}

#[test]
fn reports_feed_moderation_queue() {
    let db = setup_db();
    let reporter = create_user(&db, "q_reporter");
    let reported = create_user(&db, "q_target");

    db.create_report(reporter, "user", reported, "fraud", "Fraud")
        .unwrap();

    let queue = db.get_moderation_queue(Some("pending")).unwrap();
    assert!(!queue.is_empty());
    assert!(queue.iter().any(|i| i.item_type == "report"));
}

// ═══════════════════════════════════════════
// CONTENT FLAGS
// ═══════════════════════════════════════════

#[test]
fn content_flags_auto_and_manual() {
    let db = setup_db();
    let user = create_user(&db, "flag_user");

    let fid = db
        .flag_content("spam", "auto", "moment", 1, 0.9, "auto detected")
        .unwrap();

    let flags = db.get_content_flags(None).unwrap();
    assert_eq!(flags.len(), 1);
    assert_eq!(flags[0].source, "auto");

    assert!(!db.is_content_blocked("moment", 1).unwrap());

    db.resolve_content_flag(fid, user, "actioned").unwrap();
    assert!(db.is_content_blocked("moment", 1).unwrap());
}

// ═══════════════════════════════════════════
// SHADOW BANS
// ═══════════════════════════════════════════

#[test]
fn shadow_ban_lifecycle() {
    let db = setup_db();
    let user = create_user(&db, "shadow_user");

    assert!(!db.is_shadow_banned(user).unwrap());

    db.shadow_ban_user(user, "test", Some(3600)).unwrap();
    assert!(db.is_shadow_banned(user).unwrap());

    let ids = db.active_shadow_ban_ids().unwrap();
    assert!(ids.contains(&user));

    assert!(db.unshadow_ban_user(user).unwrap());
    assert!(!db.is_shadow_banned(user).unwrap());
}

#[test]
fn shadow_banned_posts_hidden_from_feed_and_search() {
    let db = setup_db();
    let author = create_user(&db, "ghost_author");
    let viewer = create_user(&db, "ghost_viewer");

    db.create_moment(author, "Ghosted content", "", "text")
        .unwrap();

    let feed_before = db.get_moment_feed(viewer, 0, 10).unwrap();
    assert!(
        feed_before
            .iter()
            .any(|m| m["content"] == "Ghosted content")
    );

    let search_before = db.search_users("ghost_author", 10).unwrap();
    assert_eq!(search_before.len(), 1);

    db.shadow_ban_user(author, "policy", None).unwrap();

    let feed_after = db.get_moment_feed(viewer, 0, 10).unwrap();
    assert!(!feed_after.iter().any(|m| m["content"] == "Ghosted content"));

    let search_after = db.search_users("ghost_author", 10).unwrap();
    assert_eq!(search_after.len(), 0);
}

// ═══════════════════════════════════════════
// MODERATION QUEUE
// ═══════════════════════════════════════════

#[test]
fn mod_queue_sorted_by_severity() {
    let db = setup_db();
    db.enqueue_moderation_item("user", 1, 0.4, "low").unwrap();
    db.enqueue_moderation_item("user", 2, 0.9, "high").unwrap();

    let queue = db.get_moderation_queue(Some("pending")).unwrap();
    assert_eq!(queue.len(), 2);
    assert_eq!(queue[0].severity, 0.9);
    assert_eq!(queue[1].severity, 0.4);

    db.resolve_moderation_item(queue[0].id, "reviewed").unwrap();
    let pending = db.get_moderation_queue(Some("pending")).unwrap();
    assert_eq!(pending.len(), 1);
}

// ═══════════════════════════════════════════
// APPEALS
// ═══════════════════════════════════════════

#[test]
fn appeal_approval_lifts_shadow_ban() {
    let db = setup_db();
    let user = create_user(&db, "appeal_user");
    let admin = create_user(&db, "moderator");

    db.shadow_ban_user(user, "temporary", Some(86400)).unwrap();
    assert!(db.is_shadow_banned(user).unwrap());

    let appeal_id = db
        .create_appeal(user, "shadow_ban", user, "I was wrongfully punished")
        .unwrap();
    assert!(appeal_id > 0);

    let appeals = db.get_user_appeals(user).unwrap();
    assert_eq!(appeals.len(), 1);
    assert_eq!(appeals[0].status, "open");

    db.resolve_appeal(appeal_id, admin, true, "Situation resolved")
        .unwrap();

    let resolved = db.get_appeals(Some("approved")).unwrap();
    assert_eq!(resolved.len(), 1);
    assert!(!db.is_shadow_banned(user).unwrap());

    let appeals = db.get_user_appeals(user).unwrap();
    assert_eq!(appeals[0].status, "approved");
}

// ═══════════════════════════════════════════
// TRUST SCORE
// ═══════════════════════════════════════════

#[test]
fn trust_score_penalizes_bad_behavior() {
    let db = setup_db();
    let clean = create_user(&db, "trust_clean");
    let bad = create_user(&db, "trust_bad");
    let reporter = create_user(&db, "trust_reporter");

    let clean_score = db.compute_trust_score(clean).unwrap();
    assert_eq!(clean_score, 60.0);

    let _ = db.create_report(reporter, "user", bad, "scam", "scammer");
    db.shadow_ban_user(bad, "fraud", None).unwrap();

    let bad_score = db.compute_trust_score(bad).unwrap();
    assert!(bad_score < 60.0);

    // banned user scores lower still
    db.ban_user(bad).unwrap();
    let banned_score = db.compute_trust_score(bad).unwrap();
    assert!(banned_score < bad_score);

    assert!((0.0..=100.0).contains(&banned_score));
}

#[test]
fn trust_score_badge_bonus() {
    let db = setup_db();
    let user = create_user(&db, "trust_badge");

    let base = db.compute_trust_score(user).unwrap();
    assert_eq!(base, 60.0);

    db.grant_badge(user, "identity_verified").unwrap();
    db.grant_badge(user, "host").unwrap();

    let boosted = db.compute_trust_score(user).unwrap();
    assert!(boosted > base);

    let trust = db.get_trust_score(user).unwrap();
    assert!(trust["score"].as_f64().unwrap() > base);
    assert!(trust["level"].is_string());
}

#[test]
fn trust_score_clamps_to_bounds() {
    let db = setup_db();
    let user = create_user(&db, "trust_clamp");
    let reporter = create_user(&db, "trust_reporter2");

    for _ in 0..20 {
        db.create_report(reporter, "user", user, "scam", "spam")
            .unwrap();
    }
    db.shadow_ban_user(user, "spam", None).unwrap();
    db.ban_user(user).unwrap();

    let score = db.compute_trust_score(user).unwrap();
    assert!(score >= 0.0);

    db.unban_user(user).unwrap();
    db.unshadow_ban_user(user).unwrap();
    db.grant_badge(user, "identity_verified").unwrap();
    db.grant_badge(user, "agency").unwrap();
    db.grant_badge(user, "host").unwrap();
    db.grant_badge(user, "staff").unwrap();
    db.grant_badge(user, "email_verified").unwrap();
    let score = db.compute_trust_score(user).unwrap();
    assert!(score <= 100.0);
}
