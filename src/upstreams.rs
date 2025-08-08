use std::time::Duration;
use pingora_load_balancing::{LoadBalancer, selection::RoundRobin};
use pingora_proxy::upstreams::Upstreams;
use pingora::prelude::*;

pub fn build_upstreams(urls: &[String], hc_secs: u64) -> anyhow::Result<(Upstreams<RoundRobin>, BackgroundServiceHandle<Upstreams<RoundRobin>>)> {
    let mut lb = LoadBalancer::<RoundRobin>::try_new()?;
    for u in urls { lb.add(u.parse()?); }
    let mut ups = Upstreams::from(lb);
    ups.set_health_check(health_check::TcpHealthCheck::new());
    ups.health_check_frequency = Some(Duration::from_secs(hc_secs));
    let bg = background_service("health-check", ups);
    Ok((bg.task(), bg))
}
