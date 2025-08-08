use pingora_proxy::Session;
use std::sync::Arc;
use dashmap::DashMap;
use crate::middleware::{Middleware, MwResult};
use crate::metrics::Metrics;

pub struct AuthMw { keys: Arc<DashMap<String, ()>>, jwt_enabled: bool, pub_key_pem: Option<String>, metrics: Metrics }
impl AuthMw {
    pub fn new(keys: Vec<String>, jwt_enabled: bool, pub_key_pem: Option<String>, metrics: Metrics) -> Arc<Self> {
        let map = DashMap::new(); keys.into_iter().for_each(|k| { map.insert(k, ()); });
        Arc::new(Self { keys: Arc::new(map), jwt_enabled, pub_key_pem, metrics })
    }
}
impl Middleware for AuthMw {
    fn on_request(&self, s: &mut Session) -> MwResult {
        let h = &s.req_header.headers;
        if let Some(k) = h.get("x-api-key") { if self.keys.contains_key(k.as_str()) { return Ok(()); } }
        if self.jwt_enabled {
            if let Some(a) = h.get("authorization") { if let Some(_t) = a.as_str().strip_prefix("Bearer ") { /* TODO: verify with jsonwebtoken */ return Ok(()); } }
        }
        if self.keys.is_empty() && !self.jwt_enabled { return Ok(()); }
        self.metrics.requests.with_label_values(&["in","unauth"]).inc();
        Err(pingora_core::Error::explain("unauthorized", 401))
    }
}
