//! The only way FrameLy AI reaches the network: model downloads and (optional) external
//! providers. Every request goes through `agent`, which refuses when local-only mode is on.

use crate::core::errors::{AppError, AppResult};
use std::time::Duration;

/// HTTPS client using the operating system's TLS stack and certificate store.
pub fn agent(local_only: bool, timeout: Option<Duration>) -> AppResult<ureq::Agent> {
    if local_only {
        return Err(AppError::LocalOnly);
    }
    Ok(build(timeout))
}

fn build(timeout: Option<Duration>) -> ureq::Agent {
    let tls = ureq::tls::TlsConfig::builder()
        .provider(ureq::tls::TlsProvider::NativeTls)
        .root_certs(ureq::tls::RootCerts::PlatformVerifier)
        .build();
    ureq::Agent::config_builder()
        .tls_config(tls)
        .timeout_connect(Some(Duration::from_secs(20)))
        .timeout_global(timeout)
        .http_status_as_error(false)
        .user_agent(concat!("FrameLy/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

pub fn net_err(e: ureq::Error) -> AppError {
    AppError::Network(e.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn local_only_blocks_every_request() {
        assert_eq!(super::agent(true, None).unwrap_err().code(), "local_only");
        assert!(super::agent(false, None).is_ok());
    }
}
