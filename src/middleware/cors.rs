use pingora_proxy::Session;
use std::sync::Arc;
use crate::middleware::{Middleware, MwResult};

pub struct CorsMw { origin: String, headers: String, methods: String }
impl CorsMw { pub fn new(o: String, h: String, m: String) -> Arc<Self> { Arc::new(Self { origin: o, headers: h, methods: m }) } }
impl Middleware for CorsMw {
    fn on_request(&self, s: &mut Session) -> MwResult {
        if s.req_header.method == "OPTIONS" { let rh = s.resp_header_mut(); rh.status = 204; rh.headers.insert("access-control-allow-origin".into(), self.origin.clone().into()); rh.headers.insert("access-control-allow-headers".into(), self.headers.clone().into()); rh.headers.insert("access-control-allow-methods".into(), self.methods.clone().into()); return Err(pingora_core::Error::explain("preflight", 204)); }
        Ok(())
    }
    fn on_response(&self, s: &mut Session) -> MwResult {
        let rh = &mut s.resp_header_mut().headers; rh.insert("access-control-allow-origin".into(), self.origin.clone().into()); Ok(())
    }
}
