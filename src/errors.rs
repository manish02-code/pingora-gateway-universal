use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("Config error: {0}")]
    Config(String),
    #[error("Upstream selection failed")]
    NoUpstream,
}
