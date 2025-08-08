use prometheus::{Registry, IntCounterVec, register_int_counter_vec};

#[derive(Clone)]
pub struct Metrics {
    pub requests: IntCounterVec,    // { stage, decision }
    pub upstreams: IntCounterVec,   // { result }
}

impl Metrics {
    pub fn new(reg: &Registry) -> Self {
        let requests = register_int_counter_vec!(
            "proxy_requests_total", "Requests seen by middleware", &["stage","decision"]
        ).unwrap();
        let upstreams = register_int_counter_vec!(
            "proxy_upstream_total", "Upstream results", &["result"]
        ).unwrap();
        reg.register(Box::new(requests.clone())).ok();
        reg.register(Box::new(upstreams.clone())).ok();
        Self { requests, upstreams }
    }
}
