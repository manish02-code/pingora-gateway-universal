use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use parking_lot::RwLock;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, Config as NConfig};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCfg {
    pub http_listen: String,
    pub https_listen: String,
    pub cert: String,
    pub key: String,
    pub enable_h2: bool,
    pub metrics_addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCfg { pub interval_secs: u64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclCfg { pub allow: Vec<String>, pub deny: Vec<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtCfg { pub enabled: bool, pub pub_key_pem: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCfg { pub api_keys: Vec<String>, pub jwt: JwtCfg }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateCfg { pub req_per_sec: u32, pub burst: u32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsCfg { pub allow_origin: String, pub allow_headers: String, pub allow_methods: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadersCfg { pub security: bool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MwCfg {
    pub acl: AclCfg,
    pub auth: AuthCfg,
    pub rate_limit: RateCfg,
    pub cors: CorsCfg,
    pub headers: HeadersCfg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayCfg {
    pub server: ServerCfg,
    pub upstreams: Vec<String>,
    pub health_check: HealthCfg,
    pub middleware: MwCfg,
}

#[derive(Clone)]
pub struct CfgHandle(Arc<RwLock<GatewayCfg>>);
impl CfgHandle {
    pub fn new(cfg: GatewayCfg) -> Self { Self(Arc::new(RwLock::new(cfg))) }
    pub fn get(&self) -> GatewayCfg { self.0.read().clone() }
    pub fn replace(&self, cfg: GatewayCfg) { *self.0.write() = cfg; }
}

pub fn load_yaml(path: &str) -> anyhow::Result<GatewayCfg> {
    let s = std::fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&s)?)
}

pub fn watch(path: PathBuf, handle: CfgHandle) -> notify::Result<RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        match res { Ok(_) => {
            match load_yaml(path.to_string_lossy().as_ref()) {
                Ok(new_cfg) => { handle.replace(new_cfg); info!("config reloaded"); }
                Err(e) => error!(?e, "failed to reload config"),
            }
        }, Err(e) => error!(?e, "watch error"), }
    })?;
    watcher.configure(NConfig::PreciseEvents(true))?;
    watcher.watch(&path, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}
