#[cfg(test)]
mod circuit_breaker_tests {
    use std::sync::Arc;
    use std::time::Duration;
    use ysh::middleware::circuit_breaker::CircuitBreaker;

    #[test]
    fn starts_available() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1));
        assert!(cb.is_available());
    }

    #[test]
    fn stays_closed_below_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1));
        cb.record_failure();
        cb.record_failure();
        assert!(cb.is_available(), "Should still be available at 2 failures (threshold 3)");
    }

    #[test]
    fn opens_at_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(100));
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert!(!cb.is_available(), "Should open at exactly 3 failures");
    }

    #[test]
    fn recovers_after_timeout() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        cb.record_failure();
        cb.record_failure();
        assert!(!cb.is_available());

        std::thread::sleep(Duration::from_millis(100));
        assert!(cb.is_available(), "Should recover after timeout");
    }

    #[test]
    fn success_resets_count() {
        let cb = CircuitBreaker::new(5, Duration::from_secs(1));
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert!(cb.is_available());
        cb.record_failure();
        cb.record_failure();
        assert!(cb.is_available(), "Should still be available after reset");
    }

    #[test]
    fn clone_shares_state() {
        let cb1 = CircuitBreaker::new(2, Duration::from_secs(1));
        let cb2 = cb1.clone();
        cb1.record_failure();
        cb1.record_failure();
        assert!(!cb2.is_available(), "Clone should share state");
    }

    #[test]
    fn concurrent_failures() {
        let cb = Arc::new(CircuitBreaker::new(100, Duration::from_secs(10)));
        let mut handles = vec![];
        for _ in 0..50 {
            let cb = cb.clone();
            handles.push(std::thread::spawn(move || {
                cb.record_failure();
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert!(cb.is_available(), "50 failures < 100 threshold");
    }
}

#[cfg(test)]
mod rate_limit_tests {
    use ysh::config::settings::DdosRateLimit;
    use ysh::middleware::ip_blocklist::IpBlocklist;
    use ysh::middleware::rate_limit::PerIpRateLimiter;

    fn make_limiter() -> std::sync::Arc<PerIpRateLimiter> {
        let bl = IpBlocklist::new(100, 60, 300, 10000);
        let rl = DdosRateLimit {
            auth_max_per_minute: 5,
            api_max_per_minute: 60,
            ws_max_per_minute: 30,
            admin_max_per_minute: 120,
        };
        PerIpRateLimiter::new(rl, bl)
    }

    #[test]
    fn creates_rate_limiter() {
        let _limiter = make_limiter();
    }

    #[test]
    fn allows_within_limit() {
        let limiter = make_limiter();
        for _ in 0..55 {
            let result = limiter.check("127.0.0.1", "/api/v1/profile");
            assert!(result.allowed, "Should allow within api limit");
        }
    }

    #[test]
    fn auth_limit_stricter() {
        let limiter = make_limiter();
        for _ in 0..5 {
            let result = limiter.check("10.0.0.1", "/api/v1/login");
            assert!(result.allowed, "Should allow first 5 auth requests");
        }
        let result = limiter.check("10.0.0.1", "/api/v1/login");
        assert!(!result.allowed, "Should block 6th auth request");
    }

    #[test]
    fn separate_ips_independent() {
        let limiter = make_limiter();
        for _ in 0..5 {
            limiter.check("1.1.1.1", "/api/v1/register");
        }
        let result = limiter.check("1.1.1.1", "/api/v1/register");
        assert!(!result.allowed);

        let result2 = limiter.check("2.2.2.2", "/api/v1/register");
        assert!(result2.allowed, "Different IP should not be affected");
    }
}

#[cfg(test)]
mod security_headers_tests {
    #[test]
    fn headers_module_exists() {
        let _ = ysh::middleware::security_headers::security_headers_middleware;
    }
}

#[cfg(test)]
mod ip_blocklist_tests {
    use ysh::middleware::ip_blocklist::IpBlocklist;

    #[test]
    fn blocks_after_threshold() {
        let bl = IpBlocklist::new(3, 60, 300, 10000);
        assert!(!bl.is_blocked("1.2.3.4"));
        bl.record_error("1.2.3.4");
        bl.record_error("1.2.3.4");
        assert!(!bl.is_blocked("1.2.3.4"));
        bl.record_error("1.2.3.4");
        assert!(bl.is_blocked("1.2.3.4"));
    }

    #[test]
    fn manual_block_unblock() {
        let bl = IpBlocklist::new(100, 60, 300, 10000);
        bl.block_ip("5.5.5.5", std::time::Duration::from_secs(60));
        assert!(bl.is_blocked("5.5.5.5"));
        assert!(bl.unblock_ip("5.5.5.5"));
        assert!(!bl.is_blocked("5.5.5.5"));
    }

    #[test]
    fn blocked_count() {
        let bl = IpBlocklist::new(1, 60, 300, 10000);
        assert_eq!(bl.blocked_count(), 0);
        bl.block_ip("1.1.1.1", std::time::Duration::from_secs(60));
        assert_eq!(bl.blocked_count(), 1);
    }
}

#[cfg(test)]
mod ws_guard_tests {
    use ysh::config::settings::DdosWs;
    use ysh::middleware::ip_blocklist::IpBlocklist;
    use ysh::middleware::ws_guard::WsGuard;

    fn make_guard() -> std::sync::Arc<WsGuard> {
        let bl = IpBlocklist::new(100, 60, 300, 10000);
        let cfg = DdosWs {
            max_connections_per_user: 3,
            max_message_size_bytes: 65536,
            heartbeat_timeout_secs: 60,
            message_rate_per_second: 10,
        };
        WsGuard::new(cfg, bl)
    }

    #[test]
    fn connection_limit() {
        let g = make_guard();
        assert!(g.can_connect(1));
        g.on_connect(1);
        assert!(g.can_connect(1));
        g.on_connect(1);
        assert!(g.can_connect(1));
        g.on_connect(1);
        assert!(!g.can_connect(1));
    }

    #[test]
    fn disconnect_frees_slot() {
        let g = make_guard();
        g.on_connect(1);
        g.on_connect(1);
        g.on_disconnect(1);
        assert!(g.can_connect(1));
    }

    #[test]
    fn message_rate_limit() {
        let g = make_guard();
        for _ in 0..10 {
            assert!(g.check_message_rate(1));
        }
        assert!(!g.check_message_rate(1));
    }

    #[test]
    fn different_users_independent() {
        let g = make_guard();
        g.on_connect(1);
        g.on_connect(1);
        g.on_connect(1);
        assert!(!g.can_connect(1));
        assert!(g.can_connect(2));
    }
}
