use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use anyhow::Result;
use std::sync::Arc;

pub fn build_tls_config(cert_path: &str, key_path: &str) -> Result<Arc<ServerConfig>> {
    let cert_chain = CertificateDer::pem_file_iter(cert_path)
        .map_err(|e| anyhow::anyhow!("Failed to read cert: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("Failed to parse cert: {}", e))?;

    let key_der = PrivateKeyDer::pem_file_iter(key_path)
        .map_err(|e| anyhow::anyhow!("Failed to read key: {}", e))?
        .next()
        .ok_or_else(|| anyhow::anyhow!("No private key found"))?
        .map_err(|e| anyhow::anyhow!("Failed to parse key: {}", e))?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key_der)
        .map_err(|e| anyhow::anyhow!("TLS config error: {}", e))?;

    Ok(Arc::new(config))
}
