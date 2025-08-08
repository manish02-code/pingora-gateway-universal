use pingora_proxy::Session;
use std::sync::Arc;

pub type MwResult = pingora_core::Result<()>;

pub trait Middleware: Send + Sync {
    fn on_request(&self, _s: &mut Session) -> MwResult { Ok(()) }
    fn on_upstream_request(&self, _s: &mut Session) -> MwResult { Ok(()) }
    fn on_response(&self, _s: &mut Session) -> MwResult { Ok(()) }
}

#[derive(Clone)]
pub struct Chain { inner: Arc<Vec<Arc<dyn Middleware>>> }
impl Chain {
    pub fn new(list: Vec<Arc<dyn Middleware>>) -> Self { Self { inner: Arc::new(list) } }
    pub fn on_request(&self, s: &mut Session) -> MwResult { for m in self.inner.iter() { m.on_request(s)?; } Ok(()) }
    pub fn on_up_req(&self, s: &mut Session) -> MwResult { for m in self.inner.iter() { m.on_upstream_request(s)?; } Ok(()) }
    pub fn on_response(&self, s: &mut Session) -> MwResult { for m in self.inner.iter() { m.on_response(s)?; } Ok(()) }
}

// Re-exports
pub mod acl; pub mod auth; pub mod rate_limit; pub mod cors; pub mod headers;
pub use acl::AclMw; pub use auth::AuthMw; pub use rate_limit::RateLimitMw; pub use cors::CorsMw; pub use headers::HeadersMw;
