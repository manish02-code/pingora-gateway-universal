use governor::{Quota, RateLimiter, clock::DefaultClock, state::InMemoryState};
use nonzero_ext::nonzero;
use pingora_proxy::Session;
use std::{net::IpAddr, sync::Arc};
use dashmap::DashMap;
use crate::middleware::{Middleware, MwResult};
use crate::metrics::Metrics;

pub struct RateLimitMw { buckets: Arc<DashMap<IpAddr, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>, qps: u32, burst: u32, metrics: Metrics }
impl RateLimitMw {
    pub fn new(qps: u32, burst: u32, metrics: Metrics) -> Arc<Self> { Arc::new(Self { buckets: Arc::new(DashMap::new()), qps, burst, metrics }) }
}
impl Middleware for RateLimitMw {
    fn on_request(&self, s: &mut Session) -> MwResult {
        let ip = s.client_addr().ip();
        let rl = self.buckets.entry(ip).or_insert_with(|| {
            Arc::new(RateLimiter::direct(Quota::per_second(nonzero!(self.qps)) * self.burst))
        }).clone();
        if rl.check().is_err() { self.metrics.requests.with_label_values(&["in","ratelimited"]).inc(); return Err(pingora_core::Error::explain("too many", 429)); }
        Ok(())
    }
}
