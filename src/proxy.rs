use std::sync::Arc;
use pingora_proxy::{ProxyHttp, Session};
use pingora_core::Result as PResult;
use pingora_load_balancing::selection::RoundRobin;
use pingora_proxy::upstreams::Upstreams;
use crate::middleware::Chain;

pub struct GatewayPolicy { upstreams: Arc<Upstreams<RoundRobin>>, chain: Chain }
impl GatewayPolicy { pub fn new(upstreams: Arc<Upstreams<RoundRobin>>, chain: Chain) -> Self { Self { upstreams, chain } } }

impl ProxyHttp for GatewayPolicy {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    fn request_filter(&self, s: &mut Session, _ctx: &mut Self::CTX) -> PResult<()> { self.chain.on_request(s) }
    fn upstream_request_filter(&self, s: &mut Session, _ctx: &mut Self::CTX) -> PResult<()> { self.chain.on_up_req(s) }
    fn response_filter(&self, s: &mut Session, _ctx: &mut Self::CTX) -> PResult<()> { self.chain.on_response(s) }

    fn upstream_peer(&self, _s: &mut Session, _ctx: &mut Self::CTX) -> PResult<pingora_proxy::UpstreamAddr> {
        let p = self.upstreams.select().ok_or_else(|| pingora_core::Error::new("no upstreams"))?;
        Ok(pingora_proxy::UpstreamAddr::SocketAddr(p.addr))
    }

    fn retry_after_fail(&self, _s: &mut Session, _ctx: &mut Self::CTX, tries: u32) -> bool { tries < 2 }
}
