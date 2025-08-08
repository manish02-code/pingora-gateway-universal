use cidr::IpInet;
use pingora_proxy::Session;
use std::net::IpAddr;
use std::sync::Arc;
use crate::middleware::{Middleware, MwResult};
use crate::metrics::Metrics;

pub struct AclMw { allow: Vec<IpInet>, deny: Vec<IpInet>, metrics: Metrics }
impl AclMw {
    pub fn new(allow: Vec<IpInet>, deny: Vec<IpInet>, metrics: Metrics) -> Arc<Self> { Arc::new(Self { allow, deny, metrics }) }
    fn hit(list: &Vec<IpInet>, ip: IpAddr) -> bool {
        list.iter().any(|n| match (n, ip) { (IpInet::V4(nv4), IpAddr::V4(i)) => nv4.contains(&i), (IpInet::V6(nv6), IpAddr::V6(i)) => nv6.contains(&i), _ => false })
    }
}
impl Middleware for AclMw {
    fn on_request(&self, s: &mut Session) -> MwResult {
        let ip = s.client_addr().ip();
        if !self.allow.is_empty() && !Self::hit(&self.allow, ip) { self.metrics.requests.with_label_values(&["in","block"]).inc(); return Err(pingora_core::Error::explain("forbidden", 403)); }
        if Self::hit(&self.deny, ip) { self.metrics.requests.with_label_values(&["in","block"]).inc(); return Err(pingora_core::Error::explain("forbidden", 403)); }
        Ok(())
    }
}
