fn test_db() -> ysh::db::Database {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    ysh::db::Database::new(tmp.path().to_str().unwrap()).unwrap()
}

fn create_test_user(db: &ysh::db::Database, name: &str) -> i64 {
    let user = db
        .create_user(name, &format!("{}@test.com", name), "hash123")
        .unwrap();
    user.id
}

#[cfg(test)]
mod migration_tests {
    use super::*;

    #[test]
    fn db_creates_with_tables() {
        let db = test_db();
        assert!(db.health_check().is_ok());
    }

    #[test]
    fn user_count_starts_at_zero() {
        let db = test_db();
        assert_eq!(db.user_count().unwrap(), 0);
    }
}

#[cfg(test)]
mod user_tests {
    use super::*;

    #[test]
    fn create_and_find_user() {
        let db = test_db();
        let user = db.create_user("alice", "alice@test.com", "hash").unwrap();
        assert_eq!(user.username, "alice");
        assert_eq!(user.email, "alice@test.com");
        assert_eq!(user.role, "user");

        let found = db.find_user_by_username("alice").unwrap().unwrap();
        assert_eq!(found.id, user.id);
    }

    #[test]
    fn find_user_by_id() {
        let db = test_db();
        let user = db.create_user("bob", "bob@test.com", "hash").unwrap();
        let found = db.find_user_by_id(user.id).unwrap().unwrap();
        assert_eq!(found.username, "bob");
    }

    #[test]
    fn user_exists() {
        let db = test_db();
        db.create_user("charlie", "c@test.com", "hash").unwrap();
        assert!(db.user_exists("charlie", "other@test.com").unwrap());
        assert!(db.user_exists("other", "c@test.com").unwrap());
        assert!(!db.user_exists("nobody", "nobody@test.com").unwrap());
    }

    #[test]
    fn user_count_increments() {
        let db = test_db();
        assert_eq!(db.user_count().unwrap(), 0);
        create_test_user(&db, "u1");
        assert_eq!(db.user_count().unwrap(), 1);
        create_test_user(&db, "u2");
        assert_eq!(db.user_count().unwrap(), 2);
    }

    #[test]
    fn ban_and_unban_user() {
        let db = test_db();
        let id = create_test_user(&db, "trouble");
        db.ban_user(id).unwrap();
        let user = db.find_user_by_id(id).unwrap().unwrap();
        assert!(user.locked_until.is_some());

        db.unban_user(id).unwrap();
        let user = db.find_user_by_id(id).unwrap().unwrap();
        assert!(user.locked_until.is_none());
    }

    #[test]
    fn list_users() {
        let db = test_db();
        for i in 0..5 {
            create_test_user(&db, &format!("user_{}", i));
        }
        let users = db.list_users(0, 3).unwrap();
        assert_eq!(users.len(), 3);
    }
}

#[cfg(test)]
mod totp_2fa_tests {
    use super::*;

    #[test]
    fn set_and_get_totp_secret() {
        let db = test_db();
        let id = create_test_user(&db, "tfa_user");
        db.set_totp_secret(id, "JBSWY3DPEHPK3PXP").unwrap();
        let secret = db.get_totp_secret(id).unwrap().unwrap();
        assert_eq!(secret, "JBSWY3DPEHPK3PXP");
    }

    #[test]
    fn enable_disable_totp() {
        let db = test_db();
        let id = create_test_user(&db, "tfa_user2");
        db.set_totp_secret(id, "SECRET123").unwrap();
        db.enable_totp(id).unwrap();
        let user = db.find_user_by_id(id).unwrap().unwrap();
        assert!(user.totp_enabled);

        db.disable_totp(id).unwrap();
        let user = db.find_user_by_id(id).unwrap().unwrap();
        assert!(!user.totp_enabled);
        assert!(db.get_totp_secret(id).unwrap().is_none());
    }

    #[test]
    fn recovery_codes_workflow() {
        let db = test_db();
        let id = create_test_user(&db, "recovery_user");
        let codes = vec![
            ("hash1".to_string(), false),
            ("hash2".to_string(), false),
            ("hash3".to_string(), false),
        ];
        db.store_recovery_codes(id, &codes).unwrap();

        let stored = db.get_recovery_codes(id).unwrap();
        assert_eq!(stored.len(), 3);

        db.mark_recovery_code_used(id, stored[0].id).unwrap();
        let stored = db.get_recovery_codes(id).unwrap();
        assert!(stored[0].used);
        assert!(!stored[1].used);

        db.delete_recovery_codes(id).unwrap();
        assert!(db.get_recovery_codes(id).unwrap().is_empty());
    }
}

#[cfg(test)]
mod chat_tests {
    use super::*;

    #[test]
    fn create_chat_session() {
        let db = test_db();
        let u1 = create_test_user(&db, "chatter1");
        let u2 = create_test_user(&db, "chatter2");
        let session_id = db.create_chat_session("direct", &[u1, u2]).unwrap();
        assert!(session_id > 0);
    }

    #[test]
    fn find_direct_session() {
        let db = test_db();
        let u1 = create_test_user(&db, "dm_user1");
        let u2 = create_test_user(&db, "dm_user2");
        let sid = db.create_chat_session("direct", &[u1, u2]).unwrap();

        let found = db.find_direct_session(u1, u2).unwrap();
        assert_eq!(found, Some(sid));

        let not_found = db.find_direct_session(u1, 9999).unwrap();
        assert_eq!(not_found, None);
    }

    #[test]
    fn send_and_get_messages() {
        let db = test_db();
        let u1 = create_test_user(&db, "sender");
        let u2 = create_test_user(&db, "receiver");
        let sid = db.create_chat_session("direct", &[u1, u2]).unwrap();

        let msg_id = db.send_message(sid, u1, "Hello!", "text", false).unwrap();
        assert!(msg_id > 0);
        db.send_message(sid, u2, "Hi there!", "text", false)
            .unwrap();

        let messages = db.get_messages(sid, 50, None).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["content"], "Hi there!");
        assert_eq!(messages[1]["content"], "Hello!");
    }

    #[test]
    fn mark_messages_read() {
        let db = test_db();
        let u1 = create_test_user(&db, "reader1");
        let u2 = create_test_user(&db, "reader2");
        let sid = db.create_chat_session("direct", &[u1, u2]).unwrap();

        db.send_message(sid, u1, "msg1", "text", false).unwrap();
        db.send_message(sid, u1, "msg2", "text", false).unwrap();
        db.send_message(sid, u2, "msg3", "text", false).unwrap();

        let count = db.mark_messages_read(sid, u2).unwrap();
        assert_eq!(count, 2, "Should mark 2 messages from u1 as read");
    }

    #[test]
    fn unread_message_count() {
        let db = test_db();
        let u1 = create_test_user(&db, "unread1");
        let u2 = create_test_user(&db, "unread2");
        let sid = db.create_chat_session("direct", &[u1, u2]).unwrap();

        assert_eq!(db.get_unread_message_count(u2).unwrap(), 0);
        db.send_message(sid, u1, "unread!", "text", false).unwrap();
        assert_eq!(db.get_unread_message_count(u2).unwrap(), 1);
    }

    #[test]
    fn get_user_sessions() {
        let db = test_db();
        let u1 = create_test_user(&db, "session_user1");
        let u2 = create_test_user(&db, "session_user2");
        let u3 = create_test_user(&db, "session_user3");

        db.create_chat_session("direct", &[u1, u2]).unwrap();
        db.create_chat_session("direct", &[u1, u3]).unwrap();

        let sessions = db.get_user_sessions(u1).unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn get_session_participants() {
        let db = test_db();
        let u1 = create_test_user(&db, "part1");
        let u2 = create_test_user(&db, "part2");
        let sid = db.create_chat_session("direct", &[u1, u2]).unwrap();

        let participants = db.get_session_participants(sid).unwrap();
        assert_eq!(participants.len(), 2);
    }

    #[test]
    fn encrypted_message() {
        let db = test_db();
        let u1 = create_test_user(&db, "enc_sender");
        let u2 = create_test_user(&db, "enc_receiver");
        let sid = db.create_chat_session("direct", &[u1, u2]).unwrap();

        db.send_message(sid, u1, "encrypted_content", "text", true)
            .unwrap();
        let messages = db.get_messages(sid, 50, None).unwrap();
        assert_eq!(messages[0]["encrypted"], true);
    }

    #[test]
    fn messages_before_id_pagination() {
        let db = test_db();
        let u1 = create_test_user(&db, "page1");
        let u2 = create_test_user(&db, "page2");
        let sid = db.create_chat_session("direct", &[u1, u2]).unwrap();

        let mut last_id = 0;
        for i in 0..10 {
            last_id = db
                .send_message(sid, u1, &format!("msg_{}", i), "text", false)
                .unwrap();
        }

        let messages = db.get_messages(sid, 3, None).unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["content"], "msg_9");

        let older = db.get_messages(sid, 3, Some(last_id)).unwrap();
        assert_eq!(older.len(), 3);
    }
}

#[cfg(test)]
mod matching_tests {
    use super::*;

    #[test]
    fn enqueue_and_dequeue() {
        let db = test_db();
        let u1 = create_test_user(&db, "matcher1");
        let qid = db.enqueue_match(u1, "random", "{}").unwrap();
        assert!(qid > 0);

        let removed = db.dequeue_match(u1).unwrap();
        assert!(removed);
    }

    #[test]
    fn find_random_match() {
        let db = test_db();
        let u1 = create_test_user(&db, "rand1");
        let u2 = create_test_user(&db, "rand2");
        db.enqueue_match(u1, "random", "{}").unwrap();
        db.enqueue_match(u2, "random", "{}").unwrap();

        let found = db.find_random_match(u1).unwrap();
        assert_eq!(found, Some(u2));
    }

    #[test]
    fn find_fifo_match() {
        let db = test_db();
        let u1 = create_test_user(&db, "fifo1");
        let u2 = create_test_user(&db, "fifo2");
        db.enqueue_match(u1, "random", "{}").unwrap();
        db.enqueue_match(u2, "random", "{}").unwrap();

        let found = db.find_match(u1, "random").unwrap();
        assert_eq!(found, Some(u2));
    }

    #[test]
    fn no_self_match() {
        let db = test_db();
        let u1 = create_test_user(&db, "self_match");
        db.enqueue_match(u1, "random", "{}").unwrap();

        let found = db.find_random_match(u1).unwrap();
        assert_eq!(found, None);
    }

    #[test]
    fn queue_size() {
        let db = test_db();
        let u1 = create_test_user(&db, "qsize1");
        let u2 = create_test_user(&db, "qsize2");
        assert_eq!(db.get_queue_size().unwrap(), 0);
        db.enqueue_match(u1, "random", "{}").unwrap();
        assert_eq!(db.get_queue_size().unwrap(), 1);
        db.enqueue_match(u2, "random", "{}").unwrap();
        assert_eq!(db.get_queue_size().unwrap(), 2);
        db.dequeue_match(u1).unwrap();
        assert_eq!(db.get_queue_size().unwrap(), 1);
    }

    #[test]
    fn complete_match() {
        let db = test_db();
        let u1 = create_test_user(&db, "complete1");
        let qid = db.enqueue_match(u1, "random", "{}").unwrap();
        db.complete_match(qid).unwrap();
    }
}

#[cfg(test)]
mod wallet_tests {
    use super::*;

    #[test]
    fn deposit_and_balance() {
        let db = test_db();
        let id = create_test_user(&db, "wallet_user");
        db.ensure_wallet(id).unwrap();
        let balance = db.deposit(id, 1000, "initial").unwrap();
        assert_eq!(balance, 1000);
        assert_eq!(db.get_balance(id).unwrap(), 1000);
    }

    #[test]
    fn withdraw_sufficient_funds() {
        let db = test_db();
        let id = create_test_user(&db, "withdraw_user");
        db.ensure_wallet(id).unwrap();
        db.deposit(id, 500, "").unwrap();
        let balance = db.withdraw(id, 200, "test").unwrap();
        assert_eq!(balance, 300);
    }

    #[test]
    fn withdraw_insufficient_funds() {
        let db = test_db();
        let id = create_test_user(&db, "poor_user");
        db.ensure_wallet(id).unwrap();
        db.deposit(id, 100, "").unwrap();
        let result = db.withdraw(id, 200, "test");
        assert!(result.is_err());
    }

    #[test]
    fn transfer_between_users() {
        let db = test_db();
        let alice = create_test_user(&db, "transfer_alice");
        let bob = create_test_user(&db, "transfer_bob");
        db.ensure_wallet(alice).unwrap();
        db.ensure_wallet(bob).unwrap();
        db.deposit(alice, 1000, "").unwrap();

        db.transfer(alice, bob, 300, "gift").unwrap();
        assert_eq!(db.get_balance(alice).unwrap(), 700);
        assert_eq!(db.get_balance(bob).unwrap(), 300);
    }

    #[test]
    fn transaction_history() {
        let db = test_db();
        let id = create_test_user(&db, "tx_user");
        db.ensure_wallet(id).unwrap();
        db.deposit(id, 100, "dep1").unwrap();
        db.deposit(id, 200, "dep2").unwrap();

        let txs = db.get_transactions(id, 10).unwrap();
        assert_eq!(txs.len(), 2);
    }
}

#[cfg(test)]
mod notification_tests {
    use super::*;

    #[test]
    fn create_and_list_notifications() {
        let db = test_db();
        let id = create_test_user(&db, "notif_user");
        db.create_notification(id, "test", "Title", "Body", "{}", "in_app")
            .unwrap();
        db.create_notification(id, "gift", "Gift!", "You got a gift", "{}", "in_app")
            .unwrap();

        let notifs = db.get_notifications(id, 10).unwrap();
        assert_eq!(notifs.len(), 2);
    }

    #[test]
    fn mark_notification_read() {
        let db = test_db();
        let id = create_test_user(&db, "read_user");
        let nid = db
            .create_notification(id, "test", "T", "B", "{}", "in_app")
            .unwrap();

        db.mark_notification_read(id, nid).unwrap();
        let notifs = db.get_notifications(id, 10).unwrap();
        assert_eq!(notifs[0]["read"], true);
    }

    #[test]
    fn mark_all_read() {
        let db = test_db();
        let id = create_test_user(&db, "readall_user");
        db.create_notification(id, "test", "T1", "B1", "{}", "in_app")
            .unwrap();
        db.create_notification(id, "test", "T2", "B2", "{}", "in_app")
            .unwrap();

        let count = db.mark_all_read(id).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn notification_preferences() {
        let db = test_db();
        let id = create_test_user(&db, "pref_user");
        let prefs = db.get_notification_preference(id).unwrap();
        assert_eq!(prefs["email_enabled"], true);
        assert_eq!(prefs["email_marketing"], false);

        db.update_notification_preference(id, "email_marketing", true)
            .unwrap();
        let prefs = db.get_notification_preference(id).unwrap();
        assert_eq!(prefs["email_marketing"], true);
    }

    #[test]
    fn quiet_hours() {
        let db = test_db();
        let id = create_test_user(&db, "quiet_user");
        db.update_quiet_hours(id, "22:00", "08:00").unwrap();
        let prefs = db.get_notification_preference(id).unwrap();
        assert_eq!(prefs["quiet_hours_start"], "22:00");
        assert_eq!(prefs["quiet_hours_end"], "08:00");
    }

    #[test]
    fn push_tokens() {
        let db = test_db();
        let id = create_test_user(&db, "push_user");
        db.register_push_token(id, "token_abc", "android").unwrap();
        db.register_push_token(id, "token_def", "ios").unwrap();

        let tokens = db.get_push_tokens(id).unwrap();
        assert_eq!(tokens.len(), 2);

        db.deactivate_push_token(id, "token_abc").unwrap();
        let tokens = db.get_push_tokens(id).unwrap();
        assert_eq!(tokens.len(), 1);
    }
}

#[cfg(test)]
mod gdpr_tests {
    use super::*;

    #[test]
    fn record_and_get_consent() {
        let db = test_db();
        let id = create_test_user(&db, "consent_user");
        db.record_consent(id, "analytics", true).unwrap();
        db.record_consent(id, "marketing", false).unwrap();

        let history = db.get_consent_history(id).unwrap();
        assert_eq!(history.len(), 2);
        assert!(history[0].granted);
        assert!(!history[1].granted);
    }

    #[test]
    fn export_user_data() {
        let db = test_db();
        let id = create_test_user(&db, "export_user");
        let data = db.get_user_data(id).unwrap();
        assert_eq!(data["user"]["username"], "export_user");
    }

    #[test]
    fn delete_user_data() {
        let db = test_db();
        let id = create_test_user(&db, "delete_user");
        db.record_consent(id, "test", true).unwrap();
        db.delete_user_data(id).unwrap();
        assert!(db.find_user_by_id(id).unwrap().is_none());
    }

    #[test]
    fn do_not_sell() {
        let db = test_db();
        let id = create_test_user(&db, "dns_user");
        db.set_do_not_sell(id, true).unwrap();
        let user = db.find_user_by_id(id).unwrap().unwrap();
        assert!(user.do_not_sell);
    }
}

#[cfg(test)]
mod admin_tests {
    use super::*;

    #[test]
    fn platform_stats() {
        let db = test_db();
        create_test_user(&db, "stats_user");
        let stats = db.platform_stats().unwrap();
        assert_eq!(stats["users"], 1);
    }
}

#[cfg(test)]
mod moment_tests {
    use super::*;

    #[test]
    fn create_moment() {
        let db = test_db();
        let id = create_test_user(&db, "moment_user");
        let mid = db.create_moment(id, "Hello world!", "", "text").unwrap();
        assert!(mid > 0);
    }

    #[test]
    fn like_unlike_moment() {
        let db = test_db();
        let id = create_test_user(&db, "like_user");
        let mid = db.create_moment(id, "Like me!", "", "text").unwrap();

        db.like_moment(id, mid).unwrap();
        let feed = db.get_moment_feed(id, 0, 10).unwrap();
        assert_eq!(feed[0]["likes"], 1);
        assert_eq!(feed[0]["liked"], true);

        db.unlike_moment(id, mid).unwrap();
        let feed = db.get_moment_feed(id, 0, 10).unwrap();
        assert_eq!(feed[0]["likes"], 0);
        assert_eq!(feed[0]["liked"], false);
    }

    #[test]
    fn comment_on_moment() {
        let db = test_db();
        let id = create_test_user(&db, "comment_user");
        let mid = db.create_moment(id, "Comment on me!", "", "text").unwrap();

        db.comment_on_moment(id, mid, "Great post!").unwrap();
        let comments = db.get_moment_comments(mid).unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0]["content"], "Great post!");
    }

    #[test]
    fn delete_moment() {
        let db = test_db();
        let id = create_test_user(&db, "del_moment_user");
        let mid = db.create_moment(id, "Delete me", "", "text").unwrap();

        let deleted = db.delete_moment(id, mid).unwrap();
        assert!(deleted);
        let feed = db.get_moment_feed(id, 0, 10).unwrap();
        assert!(feed.is_empty());
    }
}

#[cfg(test)]
mod gift_tests {
    use super::*;

    #[test]
    fn gift_catalog() {
        let db = test_db();
        let catalog = db.get_gift_catalog().unwrap();
        assert_eq!(catalog.len(), 6);
        assert_eq!(catalog[0]["name"], "Rose");
    }

    #[test]
    fn send_gift() {
        let db = test_db();
        let alice = create_test_user(&db, "gift_alice");
        let bob = create_test_user(&db, "gift_bob");
        db.ensure_wallet(alice).unwrap();
        db.ensure_wallet(bob).unwrap();
        db.deposit(alice, 1000, "").unwrap();

        let gift_id = db.send_gift(alice, bob, 1).unwrap();
        assert!(gift_id > 0);
        assert_eq!(db.get_balance(alice).unwrap(), 990);
        assert_eq!(db.get_balance(bob).unwrap(), 10);
    }

    #[test]
    fn received_gifts() {
        let db = test_db();
        let alice = create_test_user(&db, "rg_alice");
        let bob = create_test_user(&db, "rg_bob");
        db.ensure_wallet(alice).unwrap();
        db.ensure_wallet(bob).unwrap();
        db.deposit(alice, 500, "").unwrap();

        db.send_gift(alice, bob, 1).unwrap();
        let gifts = db.get_received_gifts(bob).unwrap();
        assert_eq!(gifts.len(), 1);
        assert_eq!(gifts[0]["from_user"], "rg_alice");
    }
}

#[cfg(test)]
mod i18n_override_tests {
    use super::*;

    #[test]
    fn i18n_override_roundtrip_and_delete() {
        let db = test_db();
        // Initially empty.
        assert!(db.get_i18n_override("es", "nav-wallet").unwrap().is_none());
        assert!(db.list_i18n_overrides().unwrap().is_empty());

        db.set_i18n_override("es", "nav-wallet", "Mi Cartera")
            .unwrap();
        assert_eq!(
            db.get_i18n_override("es", "nav-wallet").unwrap().unwrap(),
            "Mi Cartera"
        );
        // A different locale is unaffected.
        assert!(db.get_i18n_override("en", "nav-wallet").unwrap().is_none());

        let all = db.list_i18n_overrides().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].0, "es::nav-wallet");
        assert_eq!(all[0].1, "Mi Cartera");

        // Overwrite existing.
        db.set_i18n_override("es", "nav-wallet", "Cartera 2")
            .unwrap();
        assert_eq!(
            db.get_i18n_override("es", "nav-wallet").unwrap().unwrap(),
            "Cartera 2"
        );

        // Delete.
        assert!(db.delete_i18n_override("es", "nav-wallet").unwrap());
        assert!(db.get_i18n_override("es", "nav-wallet").unwrap().is_none());
        // Deleting a missing key returns false.
        assert!(!db.delete_i18n_override("es", "nav-wallet").unwrap());
    }
}
