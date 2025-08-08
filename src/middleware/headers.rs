use pingora_proxy::Session;
use std::sync::Arc;
use uuid::Uuid;
use crate::middleware::{Middleware, MwResult};

pub struct HeadersMw { security: bool }
impl HeadersMw { pub fn new(security: bool) -> Arc<Self> { Arc::new(Self { security }) } }
impl Middleware for HeadersMw {
    fn on_request(&self, s: &mut Session) -> MwResult { s.req_header.headers.insert("x-request-id".into(), Uuid::new_v4().to_string().into()); Ok(()) }
    fn on_upstream_request(&self, s: &mut Session) -> MwResult {
        let h = &mut s.req_header.headers; h.remove("connection"); h.remove("keep-alive"); h.insert("x-forwarded-for".into(), s.client_addr().ip().to_string().into()); h.insert("x-forwarded-proto".into(), if s.tls_info().is_some() { "https".into() } else { "http".into() }); Ok(())
    }
    fn on_response(&self, s: &mut Session) -> MwResult { if self.security { let rh = &mut s.resp_header_mut().headers; rh.insert("x-content-type-options".into(), "nosniff".into()); rh.insert("x-frame-options".into(), "DENY".into()); rh.insert("referrer-policy".into(), "no-referrer".into()); } Ok(()) }
}
