use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;

pub type GlobalRateLimiter =
    Arc<RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::QuantaClock>>;

pub fn create_rate_limiter(requests_per_second: u32, burst_size: u32) -> GlobalRateLimiter {
    let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap())
        .allow_burst(NonZeroU32::new(burst_size).unwrap());
    Arc::new(RateLimiter::direct(quota))
}
