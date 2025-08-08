mod config; mod errors; mod metrics; mod upstreams; mod middleware; mod proxy;

use std::net::SocketAddr;
use std::sync::Arc;
use pingora::prelude::*;
#[cfg(feature = "tls")]
use pingora_core::listeners::tls::TlsSettings;
use tracing::Level;
use axum::{Router, routing::get};
use prometheus::{Registry, Encoder, TextEncoder};

use crate::config::{load_yaml, watch, CfgHandle};
use crate::metrics::Metrics;
use crate::upstreams::build_upstreams;
use crate::middleware::{Chain, AclMw, AuthMw, RateLimitMw, CorsMw, HeadersMw};
use crate::proxy::GatewayPolicy;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info,pingora=info").with_target(false).with_max_level(Level::INFO).init();

    // Load + watch config
    let cfg_path = std::env::var("GATEWAY_CONFIG").unwrap_or_else(|_| "config/gateway.yaml".into());
    let cfg = load_yaml(&cfg_path)?; let cfg_handle = CfgHandle::new(cfg.clone());
    let _watcher = watch(std::path::PathBuf::from(&cfg_path), cfg_handle.clone())?;

    // Metrics
    let registry = Registry::new(); let metrics = Metrics::new(&registry);

    // Upstreams + health checks
    let (ups, hc_bg) = build_upstreams(&cfg.upstreams, cfg.health_check.interval_secs)?;
    let ups = Arc::new(ups);

    // Middleware chain from config
    let acl = AclMw::new(
        cfg.middleware.acl.allow.iter().filter_map(|s| s.parse().ok()).collect(),
        cfg.middleware.acl.deny.iter().filter_map(|s| s.parse().ok()).collect(),
        metrics.clone(),
    );
    let auth = AuthMw::new(
        cfg.middleware.auth.api_keys.clone(),
        cfg.middleware.auth.jwt.enabled,
        if cfg.middleware.auth.jwt.pub_key_pem.is_empty() { None } else { Some(cfg.middleware.auth.jwt.pub_key_pem.clone()) },
        metrics.clone(),
    );
    let rl = RateLimitMw::new(cfg.middleware.rate_limit.req_per_sec, cfg.middleware.rate_limit.burst, metrics.clone());
    let cors = CorsMw::new(cfg.middleware.cors.allow_origin.clone(), cfg.middleware.cors.allow_headers.clone(), cfg.middleware.cors.allow_methods.clone());
    let headers = HeadersMw::new(cfg.middleware.headers.security);
    let chain = Chain::new(vec![acl, auth, rl, cors, headers]);

    // Pingora services
    let mut server = Server::new(None)?; server.bootstrap();

    let policy = GatewayPolicy::new(ups.clone(), chain);
    let mut http_srv = pingora_proxy::http_proxy_service(&server.configuration, policy);
    http_srv.add_tcp(cfg.server.http_listen.as_str());

    // TLS listener is optional and only compiled if --features tls is used
    #[cfg(feature = "tls")]
    {
        let mut tls = TlsSettings::intermediate(&cfg.server.cert, &cfg.server.key)?;
        if cfg.server.enable_h2 { tls.enable_h2(); }
        http_srv.add_tls_with_settings(cfg.server.https_listen.as_str(), None, tls);
    }

    // Metrics server
    let app = Router::new().route("/metrics", get({
        let reg = registry.clone(); move || async move {
            let enc = TextEncoder::new(); let mf = reg.gather(); let mut buf = Vec::new(); enc.encode(&mf, &mut buf).unwrap(); String::from_utf8(buf).unwrap()
        }
    }));
    let metrics_addr: SocketAddr = cfg.server.metrics_addr.parse()?;
    tokio::spawn(async move { axum::Server::bind(&metrics_addr).serve(app.into_make_service()).await.unwrap(); });

    server.add_service(http_srv); server.add_service(hc_bg);
    tracing::info!("gateway up: http={}, metrics={}", cfg.server.http_listen, cfg.server.metrics_addr);
    #[cfg(feature = "tls")]
    tracing::info!("https (TLS) listening at {}", cfg.server.https_listen);

    server.run_forever();
    Ok(())
}
